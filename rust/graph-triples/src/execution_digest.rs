use std::fs::read_to_string;
use std::path::Path;

use hash_utils::str_seahash;
use stencila_schema::ExecutionDigest;

/// Create a new `ExecutionDigest` from a string of content
///
/// Before generating the hash of strings remove carriage returns from strings to avoid
/// cross platform differences in generated digests.
pub fn execution_digest_from_content(content: &str) -> ExecutionDigest {
    let digest = str_seahash(&strip_chars(content)).unwrap_or_default();
    ExecutionDigest {
        content_digest: digest,
        semantic_digest: digest,
        ..Default::default()
    }
}

/// Create a new `ExecutionDigest` from strings for content and semantics.
///
/// Before generating the hash of strings remove carriage returns from strings to avoid
/// cross platform differences in generated digests.
pub fn execution_digest_from_content_semantics(content: &str, semantics: &str) -> ExecutionDigest {
    ExecutionDigest {
        content_digest: str_seahash(&strip_chars(content)).unwrap_or_default(),
        semantic_digest: str_seahash(&strip_chars(semantics)).unwrap_or_default(),
        ..Default::default()
    }
}

/// Create a new `ExecutionDigest` from a file
///
/// If there is an error when hashing the file, a default (empty) digest is returned.
pub fn execution_digest_from_path(path: &Path, media_type: Option<&str>) -> ExecutionDigest {
    match read_to_string(path) {
        Ok(content) => {
            if let Some(semantics) = media_type.map(|mt| [&content, mt].concat()) {
                execution_digest_from_content_semantics(&content, &semantics)
            } else {
                execution_digest_from_content(&content)
            }
        }
        Err(..) => ExecutionDigest::default(),
    }
}

/// Strip carriage returns from strings
///
/// Because the use of carriage returns differs between *nix and Windows, we
/// strip them so that content digest does not change between platforms.
fn strip_chars(bytes: &str) -> String {
    bytes.replace('\r', "")
}
