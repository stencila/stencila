use std::path::Path;

use codec::{
    common::{
        eyre::{bail, eyre, Result},
        itertools::Itertools,
    },
    schema::Node,
    CodecSpec,
};
pub use codec::{
    format::Format, Codec, CodecDirection, CodecSupport, DecodeOptions, EncodeOptions, Losses,
    LossesResponse,
};
use node_strip::StripNode;

/// Get a list of all codecs
pub fn list() -> Vec<Box<dyn Codec>> {
    vec![
        Box::new(codec_debug::DebugCodec),
        Box::new(codec_html::HtmlCodec),
        Box::new(codec_jats::JatsCodec),
        Box::new(codec_json::JsonCodec),
        Box::new(codec_json5::Json5Codec),
        Box::new(codec_markdown::MarkdownCodec),
        Box::new(codec_text::TextCodec),
        Box::new(codec_yaml::YamlCodec),
    ]
}

/// Get a list of all codec specifications
pub fn specs() -> Vec<CodecSpec> {
    list().iter().map(|codec| codec.spec()).collect_vec()
}

/// Resolve whether an optional string is a format or codec name
///
/// If the string matches the name of a format then assume it is a format, otherwise assume it is a codec name
pub fn format_or_codec(format_or_codec: Option<String>) -> (Option<Format>, Option<String>) {
    match format_or_codec {
        Some(format_or_codec) => match Format::from_name(&format_or_codec.to_lowercase()) {
            Ok(format) => (Some(format), None),
            Err(..) => (None, Some(format_or_codec)),
        },
        None => (None, None),
    }
}

/// Get the codec for a given format
pub fn get(
    name: Option<&String>,
    format: Option<Format>,
    direction: Option<CodecDirection>,
) -> Result<Box<dyn Codec>> {
    if let Some(name) = name {
        list()
            .into_iter()
            .find_map(|codec| (codec.name() == name).then_some(codec))
            .ok_or_else(|| eyre!("Unable to find a codec with name `{name}`"))
    } else if let Some(format) = format {
        list()
            .into_iter()
            .find_map(|codec| {
                match direction {
                    Some(CodecDirection::Decode) => {
                        codec.supports_from_format(format).is_supported()
                    }
                    Some(CodecDirection::Encode) => codec.supports_to_format(format).is_supported(),
                    None => {
                        codec.supports_from_format(format).is_supported()
                            || codec.supports_to_format(format).is_supported()
                    }
                }
                .then_some(codec)
            })
            .ok_or_else(|| eyre!("Unable to find a codec supporting format `{format}`"))
    } else {
        bail!("One of `name` or `format` must be supplied")
    }
}

/// Get the specification for a codec
pub fn spec(name: &str) -> Result<CodecSpec> {
    Ok(get(Some(&name.to_string()), None, None)?.spec())
}

/// Decode a Stencila Schema node from a string
pub async fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = options
        .as_ref()
        .and_then(|options| options.format)
        .or(Some(Format::Json));

    let (node, losses) = get(codec, format, Some(CodecDirection::Decode))?
        .from_str(str, options.clone())
        .await?;
    losses.respond(options.unwrap_or_default().losses)?;

    Ok(node)
}

/// Decode a Stencila Schema node from a file system path
pub async fn from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = match options.as_ref().and_then(|options| options.format) {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    let (node, losses) = get(codec, Some(format), Some(CodecDirection::Decode))?
        .from_path(path, options.clone())
        .await?;
    losses.respond(options.unwrap_or_default().losses)?;

    Ok(node)
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
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = options
        .as_ref()
        .and_then(|options| options.format)
        .or(Some(Format::Json));

    let codec = get(codec, format, Some(CodecDirection::Encode))?;

    if let Some(EncodeOptions {
        strip_id: id,
        strip_code: code,
        strip_execution: execution,
        strip_output: output,
        strip_types: types,
        ..
    }) = options.clone()
    {
        if id || code || execution || output {
            let mut node = node.clone();
            node.strip(&node_strip::Targets {
                id,
                code,
                execution,
                output,
                types,
            });

            let (content, losses) = codec.to_string(&node, options.clone()).await?;
            losses.respond(options.unwrap_or_default().losses)?;

            return Ok(content);
        }
    }

    let (content, losses) = codec.to_string(node, options.clone()).await?;
    losses.respond(options.unwrap_or_default().losses)?;

    Ok(content)
}

/// Encode a Stencila Schema node to a file system path
pub async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = match options.as_ref().and_then(|options| options.format) {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    let codec = get(codec, Some(format), Some(CodecDirection::Encode))?;

    if let Some(EncodeOptions {
        strip_id: id,
        strip_code: code,
        strip_execution: execution,
        strip_output: output,
        strip_types: types,
        ..
    }) = options.clone()
    {
        if id || code || execution || output {
            let mut node = node.clone();
            node.strip(&node_strip::Targets {
                id,
                code,
                execution,
                output,
                types,
            });

            let losses = codec.to_path(&node, path, options.clone()).await?;
            losses.respond(options.unwrap_or_default().losses)?;

            return Ok(());
        }
    }

    let losses = codec.to_path(node, path, options.clone()).await?;
    losses.respond(options.unwrap_or_default().losses)?;

    Ok(())
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

    match output {
        Some(output) => {
            to_path(&node, output, encode_options).await?;
            Ok(String::new())
        }
        None => to_string(&node, encode_options).await,
    }
}
