use std::path::Path;

pub use codec::{Codec, DecodeOptions, EncodeOptions};
use common::eyre::{bail, Result};
use format::Format;
use node_strip::Strip;
use schema::Node;

/// Get the codec for a given format
fn get_codec(format: Format) -> Result<Box<dyn Codec>> {
    match format {
        Format::Debug => Ok(Box::new(codec_debug::DebugCodec)),
        Format::Json => Ok(Box::new(codec_json::JsonCodec)),
        Format::Json5 => Ok(Box::new(codec_json5::Json5Codec)),
        Format::Html => Ok(Box::new(codec_html::HtmlCodec)),
        Format::Yaml => Ok(Box::new(codec_yaml::YamlCodec)),
        _ => bail!("No codec available for format `{format}`"),
    }
}

/// Decode a Stencila Schema node from a string
pub async fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
    let format = match options.as_ref().and_then(|options| options.format) {
        Some(format) => format,
        None => Format::Json,
    };

    get_codec(format)?.from_str(str, options).await
}

/// Decode a Stencila Schema node from a file system path
pub async fn from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
    let format = match options.as_ref().and_then(|options| options.format) {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    get_codec(format)?.from_path(path, options).await
}

/// Decode a Stencila Schema node from a file system path with main options as arguments
pub async fn from_path_with(path: &Path, format: Option<Format>) -> Result<Node> {
    from_path(path, Some(DecodeOptions { format })).await
}

/// Decode a Stencila Schema node from `stdin`
pub async fn from_stdin(options: Option<DecodeOptions>) -> Result<Node> {
    use std::io::{self, BufRead};

    let stdin = io::stdin();
    let mut content = String::new();
    for line in stdin.lock().lines() {
        content += &line?;
    }

    from_str(&content, options).await
}

/// Encode a Stencila Schema node to a string
pub async fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
    let format = match options.as_ref().and_then(|options| options.format) {
        Some(format) => format,
        None => Format::Json,
    };

    let codec = get_codec(format)?;

    if let Some(EncodeOptions {
        strip_id: id,
        strip_code: code,
        strip_derived: derived,
        strip_outputs: outputs,
        ..
    }) = options
    {
        if id || code || outputs {
            let mut node = node.clone();
            node.strip(&node_strip::Targets {
                id,
                code,
                derived,
                outputs,
            });

            return codec.to_string(&node, options).await;
        }
    }

    codec.to_string(node, options).await
}

/// Encode a Stencila Schema node to a file system path
pub async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
    let format = match options.as_ref().and_then(|options| options.format) {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    let codec = get_codec(format)?;

    if let Some(EncodeOptions {
        strip_id: id,
        strip_code: code,
        strip_derived: derived,
        strip_outputs: outputs,
        ..
    }) = options
    {
        if id || code || outputs {
            let mut node = node.clone();
            node.strip(&node_strip::Targets {
                id,
                code,
                derived,
                outputs,
            });

            return codec.to_path(&node, path, options).await;
        }
    }

    codec.to_path(node, path, options).await
}

/// Encode a Stencila Schema node to a file system path with main options as arguments
pub async fn to_path_with(node: &Node, path: &Path, format: Option<Format>) -> Result<()> {
    to_path(
        node,
        path,
        Some(EncodeOptions {
            format,
            ..Default::default()
        }),
    )
    .await
}

/// Convert a document from one format to another
pub async fn convert(
    input: Option<&Path>,
    output: Option<&Path>,
    decode_options: Option<DecodeOptions>,
    encode_options: Option<EncodeOptions>,
) -> Result<String> {
    let node = match input {
        Some(input) => from_path(input, decode_options).await?,
        None => from_stdin(decode_options).await?,
    };

    println!("{:?}", node);

    match output {
        Some(output) => {
            to_path(&node, output, encode_options).await?;
            Ok(String::new())
        }
        None => to_string(&node, encode_options).await,
    }
}
