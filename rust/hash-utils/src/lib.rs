use std::{convert::TryInto, fs::File, hash::Hasher, io, path::Path};

use hmac::{Hmac, Mac};
use seahash::SeaHasher;
use sha2::{digest::Output, Digest, Sha256};

use common::eyre::Result;

// Re-exports for consumers of this crate
pub use hmac;
pub use sha2;
pub use seahash;

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

/// Get a SeaHash digest of a string
pub fn str_seahash(str: &str) -> Result<u64> {
    Ok(seahash::hash(str.as_bytes()))
}

/// Get a SHA-256 digest of a file
pub fn file_sha256<P: AsRef<Path>>(path: P) -> Result<Output<Sha256>> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    Ok(sha256.finalize())
}

/// Get a SeaHash digest of a file
///
/// Based on https://github.com/jRimbault/yadf/blob/04205a57882ffa7d6a9ca05016e18214a38079b6/src/fs/hash.rs#L29
pub fn file_seahash<P: AsRef<Path>>(path: P) -> Result<u64> {
    struct HashWriter<H>(H);
    impl<H: Hasher> io::Write for HashWriter<H> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut hasher = HashWriter(SeaHasher::default());
    io::copy(&mut std::fs::File::open(path)?, &mut hasher)?;
    Ok(hasher.0.finish())
}

/// Get a HMAC-SHA256 digest of bytes as a hex string
pub fn bytes_hmac_sha256_hex(key: &str, bytes: &[u8]) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())?;
    mac.update(bytes);
    let result = mac.finalize();
    Ok(format!("{:x}", result.into_bytes()))
}
