use std::{convert::TryInto, fs::File, io, path::Path};

use hmac::{Hmac, Mac};
use sha2::{digest::Output, Digest, Sha256};

use common::eyre::Result;

// Re-exports for consumers of this crate
pub use hmac;
pub use sha2;

/// Get a SHA-256 digest of a string as a hex string
pub fn str_sha256_hex(str: &str) -> String {
    format!("{:x}", str_sha256(str))
}

/// Get a SHA-256 digest of a string as 32-bytes
pub fn str_sha256_bytes(str: &str) -> [u8; 32] {
    str_sha256(str)
        .as_slice()
        .try_into()
        .expect("Should convert to array")
}

/// Get a SHA-256 digest of a file as a hex string
pub fn file_sha256_hex<P: AsRef<Path>>(path: P) -> Result<String> {
    Ok(format!("{:x}", file_sha256(path)?))
}

/// Get a SHA-256 digest of a file as 32-bytes
pub fn file_sha256_bytes<P: AsRef<Path>>(path: P) -> Result<[u8; 32]> {
    Ok(file_sha256(path)?.as_slice().try_into()?)
}

/// Get a SHA-256 digest of a string
pub fn str_sha256(str: &str) -> Output<Sha256> {
    let mut sha256 = Sha256::new();
    sha256.update(str);
    sha256.finalize()
}

/// Get a SHA-256 digest of a file
pub fn file_sha256<P: AsRef<Path>>(path: P) -> Result<Output<Sha256>> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    Ok(sha256.finalize())
}

/// Get a HMAC-SHA256 digest of bytes as a hex string
pub fn bytes_hmac_sha256_hex(key: &str, bytes: &[u8]) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())?;
    mac.update(bytes);
    let result = mac.finalize();
    Ok(format!("{:x}", result.into_bytes()))
}
