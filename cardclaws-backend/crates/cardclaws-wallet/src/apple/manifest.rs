//! `manifest.json`: a map of every bundled filename to the SHA-1 hash of its
//! contents (PassKit spec). The strip image is included here — that is the whole
//! point of the plan's A1 correction.

use std::collections::BTreeMap;

use sha1::{Digest, Sha1};

/// Build the manifest JSON string from the file set. Keys are sorted (BTreeMap)
/// so the output is deterministic.
pub fn build_manifest(files: &BTreeMap<String, Vec<u8>>) -> String {
    let entries: BTreeMap<&String, String> = files
        .iter()
        .map(|(name, bytes)| (name, sha1_hex(bytes)))
        .collect();
    // Hand-serialize to guarantee stable key ordering regardless of serde_json
    // map feature flags.
    let body = entries
        .iter()
        .map(|(name, hash)| format!("\"{name}\":\"{hash}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{body}}}")
}

fn sha1_hex(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_hashes_match_file_contents() {
        let mut files = BTreeMap::new();
        files.insert("pass.json".to_string(), b"{}".to_vec());
        files.insert("strip.png".to_string(), b"\x89PNG".to_vec());

        let manifest = build_manifest(&files);

        // Known SHA-1 of "{}" is bf21a9e8fbc5a3846fb05b4fa0859e0917b2202f.
        assert!(manifest.contains("\"pass.json\":\"bf21a9e8fbc5a3846fb05b4fa0859e0917b2202f\""));
        // Strip image MUST appear in the manifest (the A1 correction).
        assert!(manifest.contains("\"strip.png\":\""));
        let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert!(parsed.get("strip.png").is_some());
    }
}
