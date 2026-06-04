//! End-to-end Apple pass tests (PRD §15.2). Uses the fake signer so no cert is
//! needed; the bundle structure, manifest hashes, strip image, zip, and QR URL
//! are all exercised against the real pipeline.

use std::io::{Cursor, Read};

use cardclaws_wallet::apple::signer::FakePassSigner;
use cardclaws_wallet::apple::{build_pkpass, BrandAssets, PassInput};
use cardclaws_wallet::strip_renderer;

fn input() -> PassInput {
    PassInput {
        serial_number: "card-abc-123".into(),
        pass_type_id: "pass.com.cardclaws.card".into(),
        team_id: "TEAM123".into(),
        organization_name: "CardClaws".into(),
        holder_name: "Omar Sobh".into(),
        title: Some("Founder & CEO".into()),
        company: Some("RedClaw Systems".into()),
        email: Some("omar@redclaw.dev".into()),
        phone: Some("+15551234567".into()),
        website: Some("https://redclaw.dev".into()),
        profile_url: "https://cardclaws.com/omar".into(),
        background_hex: "#101014".into(),
    }
}

fn brand() -> BrandAssets {
    BrandAssets {
        icon_png: strip_renderer::render_solid(58, 58, "#ff3b30").unwrap(),
        logo_png: strip_renderer::render_solid(160, 50, "#ffffff").unwrap(),
    }
}

fn build() -> zip::ZipArchive<Cursor<Vec<u8>>> {
    let bytes = build_pkpass(&input(), &brand(), &FakePassSigner).unwrap();
    zip::ZipArchive::new(Cursor::new(bytes)).unwrap()
}

fn read_entry(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>, name: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();
    buf
}

#[test]
fn test_packager_produces_valid_zip() {
    let mut archive = build();
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "icon.png",
            "logo.png",
            "manifest.json",
            "pass.json",
            "signature",
            "strip.png",
        ]
    );
}

#[test]
fn test_pass_json_structure_valid() {
    let mut archive = build();
    let pass: serde_json::Value =
        serde_json::from_slice(&read_entry(&mut archive, "pass.json")).unwrap();
    for key in [
        "formatVersion",
        "passTypeIdentifier",
        "teamIdentifier",
        "serialNumber",
        "generic",
        "barcode",
    ] {
        assert!(pass.get(key).is_some(), "pass.json missing {key}");
    }
    assert_eq!(pass["generic"]["headerFields"][0]["value"], "Omar Sobh");
    assert_eq!(
        pass["generic"]["primaryFields"][0]["value"],
        "Founder & CEO"
    );
}

#[test]
fn test_manifest_sha1_correct() {
    let mut archive = build();
    let manifest: serde_json::Value =
        serde_json::from_slice(&read_entry(&mut archive, "manifest.json")).unwrap();

    // Every bundled file (including strip.png) must have a manifest entry whose
    // hash matches the actual bytes in the zip.
    for name in ["pass.json", "icon.png", "logo.png", "strip.png"] {
        let bytes = read_entry(&mut archive, name);
        let expected = sha1_hex(&bytes);
        assert_eq!(
            manifest[name].as_str().unwrap(),
            expected,
            "manifest hash mismatch for {name}"
        );
    }
}

#[test]
fn test_strip_image_is_bundled_not_a_url() {
    // The plan's A1 correction: strip image is a real bundled PNG referenced in
    // the manifest, and pass.json carries NO stripImage URL field.
    let mut archive = build();
    let strip = read_entry(&mut archive, "strip.png");
    assert_eq!(&strip[0..4], &[0x89, b'P', b'N', b'G']);

    let pass: serde_json::Value =
        serde_json::from_slice(&read_entry(&mut archive, "pass.json")).unwrap();
    assert!(pass.get("stripImage").is_none());
}

#[test]
fn test_signature_present_and_nonempty() {
    let mut archive = build();
    let sig = read_entry(&mut archive, "signature");
    assert!(!sig.is_empty());
}

#[test]
fn test_pass_qr_url_matches_profile() {
    let mut archive = build();
    let pass: serde_json::Value =
        serde_json::from_slice(&read_entry(&mut archive, "pass.json")).unwrap();
    assert_eq!(pass["barcode"]["message"], "https://cardclaws.com/omar");
}

fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    Sha1::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
