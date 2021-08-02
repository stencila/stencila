use eyre::Result;
use sha2::{Digest, Sha256};
use std::{fs::File, io, path::Path};

/// Return a SHA-256 hash of a string
pub fn str_to_sha256(str: &str) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(str);
    format!("{:x}", sha256.finalize())
}

/// Return a SHA-256 hash of a file
pub fn file_to_sha256<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    Ok(format!("{:x}", sha256.finalize()))
}
