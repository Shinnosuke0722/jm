use jm_core::error::{JmError, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Verify a file's SHA-256 checksum.
pub fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(path)?;
    std::io::copy(&mut file, &mut hasher)?;
    let actual = format!("{:x}", hasher.finalize());

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(JmError::ChecksumMismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn verify_valid_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"hello world").unwrap();

        // SHA-256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        verify_sha256(&path, expected).unwrap();
    }

    #[test]
    fn verify_invalid_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"hello world").unwrap();

        let result = verify_sha256(&path, "0000000000000000");
        assert!(result.is_err());
    }
}
