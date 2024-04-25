use std::path::{Path, PathBuf};

use codec::{
    common::{
        eyre::{bail, eyre, Result},
        reqwest::Client,
        tracing,
    },
    schema::Node,
};
pub use codec::{
    format::Format, Codec, CodecDirection, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo,
    EncodeOptions, Losses, LossesResponse, Mapping, MappingEntry,
};
use node_id::NodeUid;
use node_strip::{StripNode, StripTargets};

/// Get a list of all codecs
pub fn list() -> Vec<Box<dyn Codec>> {
    vec![
        Box::new(codec_cbor::CborCodec),
        Box::new(codec_debug::DebugCodec),
        Box::new(codec_dom::DomCodec),
        Box::new(codec_directory::DirectoryCodec),
        Box::new(codec_html::HtmlCodec),
        Box::new(codec_jats::JatsCodec),
        Box::new(codec_json::JsonCodec),
        Box::new(codec_json5::Json5Codec),
        Box::new(codec_jsonld::JsonLdCodec),
        Box::new(codec_markdown::MarkdownCodec),
        Box::new(codec_text::TextCodec),
        Box::new(codec_yaml::YamlCodec),
    ]
}

/// Resolve whether an optional string is a codec
pub fn codec_maybe(name: &str) -> Option<String> {
    list()
        .iter()
        .any(|codec| codec.name() == name)
        .then(|| name.to_string())
}

/// Get the codec for a given format
pub fn get(
    name: Option<&String>,
    format: Option<&Format>,
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

/// Decode a Stencila Schema node from a string
#[tracing::instrument]
pub async fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
    let (node, DecodeInfo { losses, .. }) = from_str_with_info(str, options.clone()).await?;
    if !losses.is_empty() {
        let options = options.unwrap_or_default();
        let format = options
            .format
            .map(|format| format!("{format} ", format = format.name()))
            .unwrap_or_default();
        losses.respond(
            format!("While decoding from {format}string"),
            options.losses,
        )?;
    }

    Ok(node)
}

/// Decode a Stencila Schema node from a string with decoding losses
#[tracing::instrument]
pub async fn from_str_with_info(
    str: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = options
        .as_ref()
        .and_then(|options| options.format.clone())
        .unwrap_or(Format::Json);

    let codec = get(codec, Some(&format), Some(CodecDirection::Decode))?;

    NodeUid::testing_only_reset();

    codec
        .from_str(
            str,
            Some(DecodeOptions {
                format: Some(format),
                ..options.unwrap_or_default()
            }),
        )
        .await
}

/// Decode a Stencila Schema node from a file system path
#[tracing::instrument]
pub async fn from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
    let (node, DecodeInfo { losses, .. }) = from_path_with_info(path, options.clone()).await?;
    if !losses.is_empty() {
        let options = options.unwrap_or_default();
        losses.respond(
            format!("While decoding from path `{path}`", path = path.display()),
            options.losses,
        )?;
    }

    Ok(node)
}

/// Decode a Stencila Schema node from a URL (http://, https://, or file://)
#[tracing::instrument]
pub async fn from_url(url: &str, options: Option<DecodeOptions>) -> Result<Node> {
    if url.starts_with("https://") || url.starts_with("http://") {
        // TODO: If a format or media type is specified in options than
        // use that, otherwise use the `Content-Type` header, otherwise
        // (or maybe if plain text / octet stream) then use path.
        // This is just a temporary hack
        let options = Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..options.unwrap_or_default()
        });

        // TODO: Enable HTTP caching to avoid unnecessary requests
        let response = Client::new().get(url).send().await?;
        if let Err(error) = response.error_for_status_ref() {
            let message = response.text().await?;
            bail!("{error}: {message}")
        }

        let text = response.text().await?;
        from_str(&text, options).await
    } else if let Some(path) = url.strip_prefix("file://") {
        from_path(&PathBuf::from(path), options).await
    } else {
        bail!("unknown URL protocol: {url}")
    }
}

/// Decode a Stencila Schema node from a file system path with decoding losses
#[tracing::instrument]
pub async fn from_path_with_info(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = match options.as_ref().and_then(|options| options.format.clone()) {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    let codec = get(codec, Some(&format), Some(CodecDirection::Decode))?;

    NodeUid::testing_only_reset();

    codec
        .from_path(
            path,
            Some(DecodeOptions {
                format: Some(format),
                ..options.unwrap_or_default()
            }),
        )
        .await
}

/// Decode a Stencila Schema node from `stdin`
#[tracing::instrument]
pub async fn from_stdin(options: Option<DecodeOptions>) -> Result<Node> {
    use std::io::{self, BufRead};

    let stdin = io::stdin();
    let mut content = String::new();
    for line in stdin.lock().lines() {
        content += &line?;
        content.push('\n'); // Need to add the newline back on (e.g for Markdown)
    }

    from_str(&content, options).await
}

/// Encode a Stencila Schema node to a string
#[tracing::instrument(skip(node))]
pub async fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
    let (content, EncodeInfo { losses, .. }) = to_string_with_info(node, options.clone()).await?;
    if !losses.is_empty() {
        let options = options.unwrap_or_default();
        let format = options
            .format
            .map(|format| format!("{format} ", format = format.name()))
            .unwrap_or_default();
        losses.respond(format!("While encoding to {format}string"), options.losses)?;
    }

    Ok(content)
}

/// Encode a Stencila Schema node to a string with encoding losses
#[tracing::instrument(skip(node))]
pub async fn to_string_with_info(
    node: &Node,
    options: Option<EncodeOptions>,
) -> Result<(String, EncodeInfo)> {
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = options
        .as_ref()
        .and_then(|options| options.format.clone())
        .unwrap_or(Format::Json);

    let codec = get(codec, Some(&format), Some(CodecDirection::Encode))?;

    let options = Some(EncodeOptions {
        format: Some(format),
        ..options.unwrap_or_default()
    });

    if let Some(EncodeOptions {
        strip_scopes,
        strip_types,
        strip_props,
        ..
    }) = options.clone()
    {
        if !(strip_scopes.is_empty() && strip_types.is_empty() && strip_props.is_empty()) {
            let mut node = node.clone();
            node.strip(&StripTargets::new(strip_scopes, strip_types, strip_props));
            return codec.to_string(&node, options).await;
        }
    }

    codec.to_string(node, options).await
}

/// Encode a Stencila Schema node to a file system path
#[tracing::instrument(skip(node))]
pub async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
    let EncodeInfo { losses, .. } = to_path_with_info(node, path, options.clone()).await?;
    if !losses.is_empty() {
        losses.respond(
            format!("While encoding to `{path}`", path = path.display()),
            options.unwrap_or_default().losses,
        )?;
    }

    Ok(())
}

/// Encode a Stencila Schema node to a file system path with losses
#[tracing::instrument(skip(node))]
pub async fn to_path_with_info(
    node: &Node,
    path: &Path,
    options: Option<EncodeOptions>,
) -> Result<EncodeInfo> {
    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = match options.as_ref().and_then(|options| options.format.clone()) {
        Some(format) => format,
        None => Format::from_path(path)?,
    };

    let codec = get(codec, Some(&format), Some(CodecDirection::Encode))?;

    let options = Some(EncodeOptions {
        format: Some(format),
        ..options.unwrap_or_default()
    });

    if let Some(EncodeOptions {
        strip_scopes,
        strip_types,
        strip_props,
        ..
    }) = options.clone()
    {
        if !(strip_scopes.is_empty() && strip_types.is_empty() && strip_props.is_empty()) {
            let mut node = node.clone();
            node.strip(&StripTargets::new(strip_scopes, strip_types, strip_props));
            return codec.to_path(&node, path, options).await;
        }
    }

    codec.to_path(node, path, options).await
}

/// Convert a document from one format to another
#[tracing::instrument]
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
