use std::path::{Path, PathBuf};

use crate::error::{ProviderDetails, SdkError, SdkResult};

/// Resolve a local file path from an image URL-like string.
///
/// Recognizes the spec-defined local path forms:
/// - absolute paths (`/tmp/image.png`)
/// - relative dot paths (`./image.png`)
/// - home paths (`~/image.png`)
#[must_use]
pub(crate) fn local_image_path(url: &str) -> Option<PathBuf> {
    if url.starts_with('/') || url.starts_with("./") {
        return Some(PathBuf::from(url));
    }

    if url == "~" || url.starts_with("~/") {
        let home = std::env::var_os("HOME")?;
        let mut path = PathBuf::from(home);
        if url != "~" {
            path.push(&url[2..]);
        }
        return Some(path);
    }

    None
}

/// Infer an image MIME type from the file extension.
#[must_use]
pub(crate) fn infer_media_type_from_path(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "heic" => "image/heic",
        "heif" => "image/heif",
        _ => "image/png",
    }
}

/// Read local image bytes when `url` points to a local path.
///
/// Returns `Ok(None)` for non-local URLs.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` if the local file cannot be read.
pub(crate) fn read_local_image_from_url(
    url: &str,
    explicit_media_type: Option<&str>,
    provider: &str,
) -> SdkResult<Option<(Vec<u8>, String)>> {
    let Some(path) = local_image_path(url) else {
        return Ok(None);
    };

    let data = std::fs::read(&path).map_err(|e| SdkError::InvalidRequest {
        message: format!("failed to read local image path '{}': {e}", path.display()),
        details: ProviderDetails {
            provider: Some(provider.to_string()),
            ..ProviderDetails::default()
        },
    })?;

    let media_type = explicit_media_type.map_or_else(
        || infer_media_type_from_path(&path).to_string(),
        ToString::to_string,
    );

    Ok(Some((data, media_type)))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{infer_media_type_from_path, local_image_path, read_local_image_from_url};
    use crate::error::SdkError;

    #[test]
    fn local_image_path_detects_supported_forms() {
        assert_eq!(
            local_image_path("/tmp/example.png"),
            Some(Path::new("/tmp/example.png").to_path_buf())
        );
        assert_eq!(
            local_image_path("./example.png"),
            Some(Path::new("./example.png").to_path_buf())
        );
        assert!(local_image_path("~/example.png").is_some());
        assert!(local_image_path("https://example.com/cat.png").is_none());
    }

    #[test]
    fn infer_media_type_from_extension() {
        assert_eq!(
            infer_media_type_from_path(Path::new("/tmp/a.jpg")),
            "image/jpeg"
        );
        assert_eq!(
            infer_media_type_from_path(Path::new("/tmp/a.webp")),
            "image/webp"
        );
        assert_eq!(
            infer_media_type_from_path(Path::new("/tmp/a.unknown")),
            "image/png"
        );
    }

    #[test]
    fn read_local_image_from_url_reads_bytes_and_infers_type()
    -> Result<(), Box<dyn std::error::Error>> {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("models3-local-image-{unique}.gif"));
        std::fs::write(&path, [1_u8, 2, 3, 4])?;

        let loaded = read_local_image_from_url(&path.to_string_lossy(), None, "test-provider")?;
        let _ = std::fs::remove_file(&path);

        let (bytes, media_type) = loaded.ok_or("expected local image to be read")?;
        assert_eq!(bytes, vec![1, 2, 3, 4]);
        assert_eq!(media_type, "image/gif");
        Ok(())
    }

    #[test]
    fn read_local_image_from_url_missing_file_is_error() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("models3-missing-image-{unique}.png"));
        let _ = std::fs::remove_file(&path);

        let err = read_local_image_from_url(&path.to_string_lossy(), None, "openai")
            .expect_err("expected missing path to be an error");
        assert!(matches!(err, SdkError::InvalidRequest { .. }));
    }
}
