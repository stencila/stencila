use super::{decode::decode, read::read};
use crate::nodes::Node;
use eyre::Result;

/// Import a node from a URL (including `file://` or `string://` URL)
///
/// # Arguments
///
/// - `input`: The URL to import
/// - `format`: The format to import the node from.
///             Defaults to the URL's file extension or media type. Falling back to JSON.
pub fn import(input: &str, format: Option<String>) -> Result<Node> {
    let (content, format_read) = read(input)?;
    let format = format.unwrap_or_else(|| format_read.unwrap_or_else(|| "json".to_string()));
    let decoded = decode(content, &format)?;
    Ok(decoded)
}
