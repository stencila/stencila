use eyre::Result;
use std::{fs, path::Path};

/// Read content from a URL.
///
/// # Arguments
///
/// - `input` The URL to read from (including `file://` and `stdio://` URLs)
pub fn read(input: &str) -> Result<(String, Option<String>)> {
    let content = fs::read_to_string(input)?;
    let format = Path::new(input)
        .extension()
        .map(|ext| ext.to_string_lossy().into());
    Ok((content, format))
}
