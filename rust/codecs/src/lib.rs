use std::path::Path;

use codec::{Codec, DecodeOptions, EncodeOptions};
use common::eyre::{bail, Result};
use format::Format;
use schema::Node;

/// Get the codec for a given format
fn get_codec(format: Format) -> Result<Box<dyn Codec>> {
    match format {
        Format::Json => Ok(Box::new(codec_json::JsonCodec)),
        Format::Json5 => Ok(Box::new(codec_json5::Json5Codec)),
        Format::Yaml => Ok(Box::new(codec_yaml::YamlCodec)),
        _ => bail!("No codec available for format `{format}`"),
    }
}

/// Decode a Stencila Schema node from a string
pub async fn from_str(str: &str, format: Format, options: Option<DecodeOptions>) -> Result<Node> {
    get_codec(format)?.from_str(str, options).await
}

/// Decode a Stencila Schema node from a file system path
pub async fn from_path(
    path: &Path,
    format: Option<Format>,
    options: Option<DecodeOptions>,
) -> Result<Node> {
    let format = match format {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    get_codec(format)?.from_path(path, options).await
}

/// Encode a Stencila Schema node to a string
pub async fn to_string(
    node: &Node,
    format: Format,
    options: Option<EncodeOptions>,
) -> Result<String> {
    get_codec(format)?.to_string(node, options).await
}

/// Encode a Stencila Schema node to a file system path
pub async fn to_path(
    node: &Node,
    path: &Path,
    format: Option<Format>,
    options: Option<EncodeOptions>,
) -> Result<()> {
    let format = match format {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    get_codec(format)?.to_path(node, path, options).await
}
