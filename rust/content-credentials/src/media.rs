//! Media-type and asset-byte helpers.

use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use c2pa::Reader;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

fn lower_hex(bytes: &[u8]) -> String {
    const CHARS: &[u8; 16] = b"0123456789abcdef";

    let mut hex = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        hex.push(char::from(CHARS[usize::from(byte >> 4)]));
        hex.push(char::from(CHARS[usize::from(byte & 0x0f)]));
    }
    hex
}

/// Whether Stencila signs assets with an embedded manifest for the given media type.
///
/// In MVP we restrict this to the four image formats the design names as
/// initial targets. Everything else (including `application/pdf`) goes via a
/// sidecar `.c2pa` file.
#[must_use]
pub fn embed_supported(media_type: &str) -> bool {
    matches!(
        media_type,
        "image/png" | "image/jpeg" | "image/webp" | "image/svg+xml"
    )
}

/// Whether sidecar signing needs a pre-hashed manifest instead of the SDK's
/// stream writer.
///
/// The c2pa SDK can read embedded PDF manifests, but does not yet implement
/// PDF write or strip support. Pre-hashing keeps sidecar signing working while
/// still allowing verification to detect embedded PDF manifests when present.
#[must_use]
pub fn sidecar_requires_prehashed_manifest(media_type: &str) -> bool {
    matches!(media_type, "application/pdf")
}

/// Whether the c2pa SDK can read an embedded manifest from the given media type.
///
/// This is intentionally broader than [`embed_supported`]: Stencila may choose
/// to emit a sidecar for a format that the SDK can still read embedded
/// manifests from. Verification should prefer embedded manifests in those
/// cases so a stale sidecar cannot shadow the asset's own credentials.
#[must_use]
pub fn could_have_embedded(media_type: &str) -> bool {
    Reader::supported_mime_types()
        .iter()
        .any(|supported| supported == media_type)
}

/// Compute a `sha256:<hex>` digest of a file's contents.
///
/// # Errors
///
/// Returns an error if the file does not exist or cannot be read.
pub fn sha256_file(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => Error::InputNotFound(path.to_path_buf()),
        _ => Error::Io(err),
    })?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let digest = hasher.finalize();
    Ok(format!("sha256:{}", lower_hex(&digest)))
}

/// Compute a `sha256:<hex>` digest of in-memory bytes.
#[must_use]
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", lower_hex(&digest))
}

/// Best-effort media type lookup by file extension.
///
/// # Errors
///
/// Returns an error when no media type can be inferred from the path.
pub fn guess_media_type(path: &Path) -> Result<String> {
    if let Some(media_type) = stencila_media_type(path) {
        return Ok(media_type.to_string());
    }

    mime_guess::from_path(path)
        .first()
        .map(|m| m.essence_str().to_string())
        .ok_or_else(|| Error::UnknownMediaType(path.to_path_buf()))
}

fn stencila_media_type(path: &Path) -> Option<&'static str> {
    let path_string = path.to_string_lossy().to_lowercase();

    for (suffix, media_type) in [
        (".cbor.zstd", "application/cbor+zstd"),
        (".dom.html", "text/dom"),
        (".email.html", "text/html"),
        (".jats.xml", "text/jats+xml"),
        (".json.zip", "application/json+zip"),
    ] {
        if path_string.ends_with(suffix) {
            return Some(media_type);
        }
    }

    match Path::new(path_string.as_str())
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("llmd") => Some("text/llmd"),
        Some("myst") => Some("text/myst"),
        Some("qmd") => Some("text/qmd"),
        Some("smd") => Some("text/smd"),
        _ => None,
    }
}

/// Sidecar filename convention: replace the asset extension with `.c2pa`.
#[must_use]
pub fn sidecar_path(asset_path: &Path) -> PathBuf {
    asset_path.with_extension("c2pa")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Documents which media types use embedded manifests versus sidecar-only manifests.
    #[test]
    fn embed_matrix() {
        assert!(embed_supported("image/png"));
        assert!(embed_supported("image/jpeg"));
        assert!(embed_supported("image/webp"));
        assert!(embed_supported("image/svg+xml"));
        assert!(!embed_supported("application/pdf"));
        assert!(!embed_supported("image/gif"));
        assert!(!embed_supported("text/plain"));
        assert!(sidecar_requires_prehashed_manifest("application/pdf"));
        assert!(!sidecar_requires_prehashed_manifest("image/gif"));
    }

    /// Documents that verification checks SDK read support, not Stencila's signing policy.
    #[test]
    fn embedded_read_matrix() {
        assert!(could_have_embedded("image/png"));
        assert!(could_have_embedded("application/pdf"));
        assert!(could_have_embedded("image/gif"));
        assert!(!could_have_embedded("text/plain"));
    }

    /// Checks file hashing against the well-known SHA-256 digest for `abc`.
    #[test]
    fn sha256_golden() {
        let mut f = tempfile::NamedTempFile::new().expect("tempfile");
        f.write_all(b"abc").expect("write");
        f.flush().expect("flush");
        let digest = sha256_file(f.path()).expect("digest");
        // Well-known sha256("abc")
        assert_eq!(
            digest,
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(sha256_bytes(b"abc"), digest);
    }

    /// Ensures sidecar paths replace the asset extension with `.c2pa`.
    #[test]
    fn sidecar_path_uses_c2pa_extension() {
        let p = sidecar_path(Path::new("/tmp/figures/cell-4.png"));
        assert_eq!(p, Path::new("/tmp/figures/cell-4.c2pa"));
    }

    /// Ensures media type guessing succeeds for known extensions and fails for unknown ones.
    #[test]
    fn guess_media_type_known_and_unknown() {
        assert_eq!(
            guess_media_type(Path::new("foo.png")).expect("png"),
            "image/png"
        );
        assert!(guess_media_type(Path::new("foo.unknownext")).is_err());
    }

    /// Ensures Stencila-specific extensions override generic MIME guesses.
    #[test]
    fn guess_media_type_stencila_paths() {
        assert_eq!(
            guess_media_type(Path::new("foo.smd")).expect("smd"),
            "text/smd"
        );
        assert_eq!(
            guess_media_type(Path::new("foo.dom.html")).expect("dom"),
            "text/dom"
        );
        assert_eq!(
            guess_media_type(Path::new("foo.json.zip")).expect("json zip"),
            "application/json+zip"
        );
    }
}
