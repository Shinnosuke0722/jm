use jm_core::error::{JmError, Result};
use sha2::{Digest, Sha256};
use std::{fmt::Write as _, io::Read, path::Path};

const HASH_BUFFER_SIZE: usize = 64 * 1024;

/// Verify a file's SHA-256 checksum.
pub fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(path)?;
    let mut buffer = [0_u8; HASH_BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let digest = hasher.finalize();
    let mut actual = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut actual, "{byte:02x}").expect("writing to a String cannot fail");
    }

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

    #[test]
    fn verify_checksum_across_multiple_read_buffers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large.bin");
        std::fs::write(&path, vec![b'a'; 1_000_000]).unwrap();

        // SHA-256 of one million ASCII 'a' bytes.
        let expected = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";
        verify_sha256(&path, expected).unwrap();
    }
}
