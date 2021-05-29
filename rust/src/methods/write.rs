use eyre::Result;
use std::fs;

/// Write content to a URL.
///
/// # Arguments
///
/// - `content` The content to write
/// - `output` The URL to write to (including `file://` and `stdio://` URLs)
pub fn write(content: &str, output: &str) -> Result<()> {
    fs::write(output, content)?;
    Ok(())
}
