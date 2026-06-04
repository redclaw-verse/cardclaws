//! Zips the bundle into a `.pkpass`. Contents: every payload/brand file, plus
//! `manifest.json` and the detached `signature`.

use std::collections::BTreeMap;
use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::WalletError;

pub fn package(
    files: &BTreeMap<String, Vec<u8>>,
    manifest_json: &str,
    signature: &[u8],
) -> Result<Vec<u8>, WalletError> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut write = |name: &str, bytes: &[u8]| -> Result<(), WalletError> {
        zip.start_file(name, opts)
            .map_err(|e| WalletError::Packaging(e.to_string()))?;
        zip.write_all(bytes)
            .map_err(|e| WalletError::Packaging(e.to_string()))?;
        Ok(())
    };

    for (name, bytes) in files {
        write(name, bytes)?;
    }
    write("manifest.json", manifest_json.as_bytes())?;
    write("signature", signature)?;

    let cursor = zip
        .finish()
        .map_err(|e| WalletError::Packaging(e.to_string()))?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn produces_openable_zip_with_required_entries() {
        let mut files = BTreeMap::new();
        files.insert("pass.json".to_string(), b"{}".to_vec());
        files.insert("strip.png".to_string(), b"\x89PNG".to_vec());

        let zip_bytes = package(&files, "{\"pass.json\":\"abc\"}", b"sig").unwrap();

        let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes)).unwrap();
        let mut names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec!["manifest.json", "pass.json", "signature", "strip.png"]
        );

        let mut manifest = String::new();
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest)
            .unwrap();
        assert!(manifest.contains("pass.json"));
    }
}
