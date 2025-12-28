use stencila_images::ImageResizeOptions;

/// HTML escape special characters
pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Process an image URL for email, resizing and embedding as needed
///
/// - Data URIs are resized to email dimensions (600px max width)
/// - HTTP/HTTPS URLs are passed through unchanged
/// - File paths are loaded, resized, and converted to data URIs
pub(crate) fn process_image_url(url: &str) -> String {
    if url.starts_with("data:") {
        // Resize existing data URI
        let options = ImageResizeOptions::for_email();
        match stencila_images::resize_data_uri(url, &options) {
            Ok(resized) => resized,
            Err(e) => {
                tracing::warn!("Failed to resize image data URI: {e}");
                url.to_string()
            }
        }
    } else if url.starts_with("http://") || url.starts_with("https://") {
        // HTTP URLs pass through unchanged
        url.to_string()
    } else {
        // File path - load, resize, and convert to data URI
        let path = std::path::Path::new(url.strip_prefix("file://").unwrap_or(url));
        let options = ImageResizeOptions::for_email();
        match stencila_images::resize_file_to_data_uri(path, &options) {
            Ok(data_uri) => data_uri,
            Err(e) => {
                tracing::warn!("Failed to load image file '{}': {e}", path.display());
                url.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }
}
