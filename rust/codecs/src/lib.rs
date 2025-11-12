use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use chrono::Local;
use futures::StreamExt;
use reqwest::Client;
use tempfile::tempdir;
use tokio::{
    fs::{File, read_to_string, write},
    io::AsyncWriteExt,
};
use url::Url;
use walkdir::WalkDir;

use stencila_ask::{Answer, AskLevel, AskOptions, ask_with};
use stencila_cli_utils::{Code, ToStdout};
use stencila_codec::stencila_schema::{
    Article, Block, IncludeBlock, Node, VisitorAsync, WalkControl, WalkNode,
};
pub use stencila_codec::{
    CitationStyle, Codec, CodecDirection, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo,
    EncodeOptions, Losses, LossesResponse, Mapping, MappingEntry, Message, MessageLevel, Messages,
    PageSelector, PoshMap, Position8, Position16, Positions, Range8, Range16, StructuringOperation,
    StructuringOptions,
    eyre::{Context, OptionExt, Result, bail, eyre},
    stencila_format::Format,
};
use stencila_codec_utils::rebase_edits;
use stencila_node_strip::{StripNode, StripTargets};
use stencila_node_structuring::structuring;

use stencila_codec_arxiv::ArxivCodec;
use stencila_codec_biblio::decode::text_to_reference;
use stencila_codec_cbor::CborCodec;
use stencila_codec_cff::CffCodec;
use stencila_codec_crossref::CrossrefCodec;
use stencila_codec_csl::CslCodec;
use stencila_codec_csv::CsvCodec;
use stencila_codec_debug::DebugCodec;
use stencila_codec_directory::DirectoryCodec;
use stencila_codec_docx::DocxCodec;
use stencila_codec_doi::DoiCodec;
use stencila_codec_dom::DomCodec;
use stencila_codec_github::GithubCodec;
use stencila_codec_html::HtmlCodec;
use stencila_codec_ipynb::IpynbCodec;
use stencila_codec_jats::JatsCodec;
use stencila_codec_json::JsonCodec;
use stencila_codec_json5::Json5Codec;
use stencila_codec_jsonld::JsonLdCodec;
use stencila_codec_latex::LatexCodec;
use stencila_codec_lexical::LexicalCodec;
use stencila_codec_markdown::MarkdownCodec;
use stencila_codec_meca::MecaCodec;
use stencila_codec_odt::OdtCodec;
use stencila_codec_openalex::OpenAlexCodec;
use stencila_codec_openrxiv::OpenRxivCodec;
use stencila_codec_pandoc::PandocCodec;
use stencila_codec_pdf::PdfCodec;
use stencila_codec_pmc::PmcCodec;
use stencila_codec_png::PngCodec;
use stencila_codec_rnw::RnwCodec;
use stencila_codec_swb::SwbCodec;
use stencila_codec_text::TextCodec;
use stencila_codec_xlsx::XlsxCodec;
use stencila_codec_yaml::YamlCodec;
use stencila_codec_zenodo::ZenodoCodec;

//#[cfg(feature = "stencila-codec-polars")]
//use stencila_codec_polars::PolarsCodec;

pub mod cli;
pub mod remotes;

/// Get a list of all codecs
pub fn list() -> Vec<Box<dyn Codec>> {
    vec![
        Box::new(CborCodec) as Box<dyn Codec>,
        Box::new(CffCodec),
        Box::new(CslCodec),
        Box::new(CsvCodec),
        Box::new(DebugCodec),
        Box::new(DocxCodec),
        // DomCodec supports to HTML and because listed here before HtmlCodec
        // will be selected when encoding to HTML
        Box::new(DomCodec),
        Box::new(DirectoryCodec),
        Box::new(GithubCodec),
        Box::new(HtmlCodec),
        Box::new(IpynbCodec),
        Box::new(JatsCodec),
        Box::new(JsonCodec),
        Box::new(Json5Codec),
        Box::new(JsonLdCodec),
        Box::new(LatexCodec),
        Box::new(LexicalCodec),
        Box::new(MarkdownCodec),
        Box::new(MecaCodec),
        //#[cfg(feature = "stencila-codec-polars")]
        //Box::new(PolarsCodec),
        Box::new(RnwCodec),
        Box::new(OdtCodec),
        Box::new(PandocCodec),
        Box::new(PmcCodec),
        Box::new(PdfCodec),
        Box::new(PngCodec),
        Box::<SwbCodec>::default(),
        Box::new(TextCodec),
        Box::new(XlsxCodec),
        Box::new(YamlCodec),
        // arXiv codec supports from HTML but because after all others supporting HTML
        // will need to be explicitly chosen
        Box::new(ArxivCodec),
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
    if let Some(name) = name
        && let Some(codec) = list()
            .into_iter()
            .find_map(|codec| (codec.name() == name).then_some(codec))
    {
        return Ok(codec);
    }

    if let Some(format) = format
        && let Some(codec) = list().into_iter().find_map(|codec| {
            match direction {
                Some(CodecDirection::Decode) => codec.supports_from_format(format).is_supported(),
                Some(CodecDirection::Encode) => codec.supports_to_format(format).is_supported(),
                None => {
                    codec.supports_from_format(format).is_supported()
                        || codec.supports_to_format(format).is_supported()
                }
            }
            .then_some(codec)
        })
    {
        return Ok(codec);
    }

    let dir = match direction {
        Some(CodecDirection::Decode) => "decoding from ",
        Some(CodecDirection::Encode) => "encoding from ",
        None => "",
    };

    match (name, format) {
        (Some(name), Some(format)) => {
            bail!("Unable to find a codec with name `{name}` or supporting {dir}format `{format}`")
        }
        (Some(name), None) => bail!("Unable to find a codec with name `{name}`"),
        (None, Some(format)) => bail!("Unable to find a codec supporting format `{format}`"),
        (None, None) => bail!("At least one of `name` or `format` must be supplied"),
    }
}

/// Determine whether [`from_path`] is supported for a path
pub fn from_path_is_supported(path: &Path) -> bool {
    let format = Format::from_path(path);
    get(None, Some(&format), Some(CodecDirection::Decode)).is_ok()
}

/// Determine whether [`to_path`] is supported for a path
pub fn to_path_is_supported(path: &Path) -> bool {
    let format = Format::from_path(path);
    get(None, Some(&format), Some(CodecDirection::Encode)).is_ok()
}

/// Decode a Stencila Schema node from a string
#[tracing::instrument(skip(str))]
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
#[tracing::instrument(skip(str))]
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

/// Get the codec that supports a given identifier
pub fn codec_for_identifier(identifier: &str) -> Option<Box<dyn Codec>> {
    let path = PathBuf::from(identifier);
    if path.exists() {
        let format = Format::from_path(&path);
        get(None, Some(&format), Some(CodecDirection::Decode)).ok()
    } else if ArxivCodec::supports_identifier(identifier) {
        Some(Box::new(ArxivCodec))
    } else if OpenRxivCodec::supports_identifier(identifier) {
        Some(Box::new(OpenRxivCodec))
    } else if PmcCodec::supports_identifier(identifier) {
        Some(Box::new(PmcCodec))
    } else if GithubCodec::supports_identifier(identifier) {
        Some(Box::new(GithubCodec))
    } else {
        None
    }
}

/// Decode a Stencila Schema node from an identifier
#[tracing::instrument]
pub async fn from_identifier(identifier: &str, options: Option<DecodeOptions>) -> Result<Node> {
    // Read from stdin
    if identifier == "-" {
        return from_stdin(options).await;
    }

    // Read from an existing path
    let path = PathBuf::from(identifier);
    if path.exists() {
        return from_path(&path, options).await;
    }

    // Try codecs that supports identifiers (including specific URLs)
    if let Some((mut node, .., codec_structuring_options)) =
        if ArxivCodec::supports_identifier(identifier) {
            Some(ArxivCodec::from_identifier(identifier, options.clone()).await?)
        } else if OpenRxivCodec::supports_identifier(identifier) {
            Some(OpenRxivCodec::from_identifier(identifier, options.clone()).await?)
        } else if PmcCodec::supports_identifier(identifier) {
            Some(PmcCodec::from_identifier(identifier, options.clone()).await?)
        } else if GithubCodec::supports_identifier(identifier) {
            Some(GithubCodec::from_identifier(identifier, options.clone()).await?)
        } else if ZenodoCodec::supports_identifier(identifier) {
            Some(ZenodoCodec::from_identifier(identifier, options.clone()).await?)
        } else {
            None
        }
    {
        // Merge any user supplied structuring options with the codec's defaults
        // for the format resolved from the identifier
        let mut structuring_options = options
            .map(|opts| opts.structuring_options)
            .unwrap_or_default();
        structuring_options.merge(codec_structuring_options);

        // Perform any structuring
        if structuring_options.should_perform_any() {
            structuring(&mut node, structuring_options).await?;
        }

        return Ok(node);
    }

    // Read from a generic URL (ignoring DOI URLs which are handled below)
    if ((identifier.starts_with("https://") || identifier.starts_with("http://"))
        && !identifier.contains("doi.org/"))
        || identifier.starts_with("file://")
    {
        return from_url(identifier, options).await;
    }

    // Try codecs that provide metadata from a reference (could be a citation, or a DOI)
    let reference = text_to_reference(identifier);
    if let Ok(node) = OpenAlexCodec::from_reference(&reference).await {
        return Ok(node);
    }
    if let Ok(node) = CrossrefCodec::from_reference(&reference).await {
        return Ok(node);
    }
    if let Ok(node) = DoiCodec::from_reference(&reference).await {
        return Ok(node);
    }

    bail!("Unable to decode identifier into a node: {identifier}")
}

/// Decode a Stencila Schema node from a file system path
#[tracing::instrument]
pub async fn from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
    let (node, .., DecodeInfo { losses, .. }) = from_path_with_info(path, options.clone()).await?;
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
pub async fn from_url(input: &str, options: Option<DecodeOptions>) -> Result<Node> {
    if let Some(path) = input.strip_prefix("file://") {
        // URL:parse will remove any leading `../` etc so avoid that and do it
        // this way
        return from_path(&PathBuf::from(path), options).await;
    }

    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let url = Url::parse(input)?;
    match url.scheme() {
        "http" | "https" => {
            // TODO: Enable HTTP caching to avoid unnecessary requests
            tracing::info!("Fetching {url}");
            let response = Client::new().get(url.as_str()).send().await?;
            if let Err(error) = response.error_for_status_ref() {
                let message = response.text().await?;
                bail!("{error}: {message}")
            }

            // Determine format based on options, Content-Type header, or URL path
            let format = options
                .as_ref()
                .and_then(|opts| opts.format.clone())
                .or_else(|| {
                    // Try to determine format from Content-Type header
                    response
                        .headers()
                        .get("content-type")
                        .and_then(|ct| ct.to_str().ok())
                        .and_then(|ct| Format::from_content_type(ct).ok())
                })
                .unwrap_or_else(|| {
                    // Fall back to determining format from URL path
                    Format::from_path(&PathBuf::from(url.path()))
                });

            // Check there is a codec that supports the format
            let codec = get(codec, Some(&format), Some(CodecDirection::Decode))?;

            let options = Some(DecodeOptions {
                format: Some(format.clone()),
                ..options.unwrap_or_default()
            });

            // If the content is small and the codec supports `from_str` then no need to
            // touch the filesystem. Otherwise, download to a temporary file and decode that.
            let content_length = response.content_length();
            const MAX_IN_MEMORY_SIZE: u64 = 10 * 1024 * 1024; // 10MB

            if codec.supports_from_string()
                && content_length.is_some_and(|len| len <= MAX_IN_MEMORY_SIZE)
            {
                let text = response.text().await?;
                from_str(&text, options).await
            } else {
                let temp_dir = tempdir()?;
                let temp_file = temp_dir
                    .path()
                    .join(format!("download.{}", format.extension()));

                let mut file = File::create(&temp_file).await?;
                let mut stream = response.bytes_stream();
                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    file.write_all(&chunk).await?;
                }
                file.flush().await?;
                drop(file);

                from_path(&temp_file, options).await
            }
        }
        scheme => {
            bail!("Unsupported URL scheme: {scheme}")
        }
    }
}

/// Decode a Stencila Schema node from a file system path with decoding losses
#[tracing::instrument]
pub async fn from_path_with_info(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    if !path.exists() {
        bail!("Path does not exist: {}", path.display());
    }

    let codec = options.as_ref().and_then(|options| options.codec.as_ref());

    let format = match options.as_ref().and_then(|options| options.format.clone()) {
        Some(format) => format,
        None => Format::from_path(path),
    };

    let codec = get(codec, Some(&format), Some(CodecDirection::Decode))?;

    let (mut node, other, info) = codec
        .from_path(
            path,
            Some(DecodeOptions {
                format: Some(format.clone()),
                ..options.clone().unwrap_or_default()
            }),
        )
        .await?;

    // Merge any user supplied structuring options with the codec's default for
    // the format
    let mut structuring_options = options
        .map(|opts| opts.structuring_options)
        .unwrap_or_default();
    structuring_options.merge(codec.structuring_options(&format));

    // Perform any structuring
    if structuring_options.should_perform_any() {
        structuring(&mut node, structuring_options).await?;
    }

    Ok((node, other, info))
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

/// Encode a Stencila Schema node to `stdout`
#[tracing::instrument]
pub async fn to_stdout(node: &Node, options: Option<EncodeOptions>) -> Result<()> {
    let format = options
        .as_ref()
        .and_then(|opts| opts.format.clone())
        .unwrap_or_default();

    let content = to_string(node, options).await?;
    Code::new(format, &content).to_stdout();

    Ok(())
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
        losses.respond(
            format!("Losses when encoding to {format}string"),
            options.losses,
        )?;
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
        && !(strip_scopes.is_empty() && strip_types.is_empty() && strip_props.is_empty())
    {
        let mut node = node.clone();
        node.strip(&StripTargets::new(strip_scopes, strip_types, strip_props));
        return codec.to_string(&node, options).await;
    }

    codec.to_string(node, options).await
}

/// Encode a Stencila Schema node to a file system path
///
/// Returns `Ok(true)` if the file was encoded successfully and `Ok(false)` if
/// the user answered No to a prompt asking if they wanted to continue.
#[tracing::instrument(skip(node))]
pub async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<bool> {
    if options
        .as_ref()
        .and_then(|opts| opts.reproducible)
        .unwrap_or_default()
        && let Node::Article(Article { options, .. }) = &node
        && !check_git_for_to_path(&options.path, &options.commit).await?
    {
        return Ok(false);
    }

    let EncodeInfo { losses, .. } = to_path_with_info(node, path, options.clone()).await?;

    if !losses.is_empty() {
        losses.respond(
            format!("Losses when encoding to `{path}`", path = path.display()),
            options.clone().unwrap_or_default().losses,
        )?;
    }

    if options
        .as_ref()
        .and_then(|opts| opts.recurse)
        .unwrap_or_default()
    {
        let mut recurser = Recurser {
            path: path.to_path_buf(),
            options,
        };
        node.clone().walk_async(&mut recurser).await?;
    }

    Ok(true)
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
        None => Format::from_path(path),
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
        && !(strip_scopes.is_empty() && strip_types.is_empty() && strip_props.is_empty())
    {
        let mut node = node.clone();
        node.strip(&StripTargets::new(strip_scopes, strip_types, strip_props));
        return codec.to_path(&node, path, options).await;
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
        Some(input) => {
            if input == PathBuf::from("-") {
                from_stdin(decode_options).await?
            } else {
                from_path(input, decode_options).await?
            }
        }
        None => from_stdin(decode_options).await?,
    };

    match output {
        Some(output) => {
            if output == PathBuf::from("-") {
                to_string(&node, encode_options).await
            } else {
                to_path(&node, output, encode_options).await?;
                Ok(String::new())
            }
        }
        None => to_string(&node, encode_options).await,
    }
}

/// Merge changes from an edited document into the original document
///
/// Usually the edited document is in a different format to the
/// original.
///
/// Returns a vector of paths that were modified during the merge or
/// `None` is the merge was cancelled.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument]
pub async fn merge(
    edited: &Path,
    original: Option<&Path>,
    unedited: Option<&Path>,
    commit: Option<&str>,
    rebase: bool,
    decode_options: DecodeOptions,
    mut encode_options: EncodeOptions,
    workdir: Option<PathBuf>,
) -> Result<Option<Vec<PathBuf>>> {
    // Create, or use specified, working directory
    let tempdir = tempdir()?;
    let workdir = if let Some(workdir) = &workdir {
        workdir
    } else {
        tempdir.path()
    };

    let unedited_provided = unedited.is_some();

    // Decode the edited file because it may contain information on the
    // original source & commit not supplied as an argument
    let (edited_node, unedited_node, ..) =
        from_path_with_info(edited, Some(decode_options.clone())).await?;

    let mut original = original.map(|path| path.to_path_buf());
    if original.is_none()
        && let Node::Article(Article { options, .. }) = &edited_node
        && let Some(source) = &options.path
    {
        original = Some(PathBuf::from(source));
    }
    let Some(original) = original else {
        bail!(
            "Relative path of original source file not specified and not available from edited document"
        )
    };

    let mut commit = commit.map(String::from);
    if commit.is_none()
        && let Node::Article(Article { options, .. }) = &edited_node
        && let Some(value) = &options.commit
    {
        commit = Some(value.to_string());
    }

    // If a commit is available then check the status of the file relative to the path
    if let Some(commit) = commit
        && commit != "untracked"
        && commit != "dirty"
    {
        let should_continue = check_git_for_merge(&original, &commit, edited, false).await?;
        if !should_continue {
            tracing::debug!("Merge cancelled");
            return Ok(None);
        }
    }

    // Get the dir and file name of the original for intermediate files
    let original_dir = original
        .parent()
        .ok_or_eyre("original file has no parent")?;
    let original_file = original
        .file_name()
        .ok_or_eyre("original file has no name")?;

    // Override decoding and encoding options
    // TODO: Warn user if their settings have been ignored
    encode_options.recurse = Some(true);
    encode_options.render = Some(false);

    // Track modified files
    let mut modified_files = Vec::new();

    // Convert the edited node into the original format
    let edited_dir = workdir.join("edited");
    to_path(
        &edited_node,
        &edited_dir.join(original_file),
        Some(encode_options.clone()),
    )
    .await?;

    let unedited_dir = workdir.join("unedited");
    let unedited_file = unedited_dir.join(original_file);
    if let Some(unedited) = unedited {
        // If an unedited file was provided, convert into the original format
        convert(
            Some(unedited),
            Some(&unedited_file),
            Some(decode_options),
            Some(encode_options),
        )
        .await?;
    } else if let Some(unedited_node) = unedited_node {
        // If un unedited node was embedded in the edited file, encode it to the
        // original format
        to_path(&unedited_node, &unedited_file, Some(encode_options)).await?;
    }

    // Apply edits for each file in the `edited` directory.
    for entry in WalkDir::new(&edited_dir)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let edited_path = entry.path();
        if !edited_path.is_file() {
            continue;
        }

        let edited_format = Format::from_path(edited_path);
        if edited_format.is_audio() || edited_format.is_image() || edited_format.is_video() {
            continue;
        }

        let edited_string = match read_to_string(edited_path).await {
            Ok(content) => content,
            Err(error) => {
                tracing::debug!("Unable to read `{}`: {error}", edited_path.display());
                continue;
            }
        };

        let relative_path = edited_path
            .strip_prefix(&edited_dir)
            .expect("not in edited dir");
        let unedited_path = unedited_dir.join(relative_path);
        let original_path = original_dir.join(relative_path);

        // If a file exists in the edited dir but not in the unedited then just
        // write it to the original dir (no rebasing to do)
        if !rebase || !unedited_path.exists() {
            write(&original_path, edited_string).await?;
            modified_files.push(original_path);
            continue;
        }

        let unedited_string = read_to_string(unedited_path).await?;
        if edited_string == unedited_string {
            tracing::trace!("No changes, skipping `{}`", relative_path.display());
            continue;
        }

        let original_string = read_to_string(&original_path).await.wrap_err_with(|| {
            eyre!(
                "Could not find original file `{}` to merge into",
                original_path.display()
            )
        })?;

        tracing::debug!(
            "Rebasing edits for `{}` using unedited version {}",
            relative_path.display(),
            if unedited_provided {
                "provided".to_string()
            } else {
                format!("embedded in `{}`", edited.display())
            }
        );
        let rebased = rebase_edits(&original_string, &unedited_string, &edited_string);

        write(&original_path, rebased).await?;
        modified_files.push(original_path);
    }

    Ok(Some(modified_files))
}

/// Generate warnings if a node is being encoded with the `--reproducible` option but
/// does not have necessary metadata
///
/// # Returns
///
/// * `Ok(true)` - User answered yes to continuing
/// * `Ok(false)` - User answered no to continuing
/// * `Err(_)` - User input could not be read
pub async fn check_git_for_to_path(
    source: &Option<String>,
    commit: &Option<String>,
) -> Result<bool> {
    let message = if let Some(source) = source {
        if commit.is_none() {
            format!(
                "file `{source}` whose Git commit is unknown. You may need to specify the commit when merging changes later"
            )
        } else if matches!(commit.as_deref(), Some("untracked")) {
            format!(
                "file `{source}` which is in a Git repository but is not tracked. Consider committing this file first to be able to merge changes correctly later"
            )
        } else if matches!(commit.as_deref(), Some("dirty")) {
            format!(
                "file `{source}` which has uncommitted changes. Consider committing these changes first to be able to merge changes correctly later"
            )
        } else {
            return Ok(true);
        }
    } else {
        "a file that does not appear to be within a Git repository. This may make it harder to resolve conflicts when merging changes".into()
    };

    let answer = ask_with(
        &format!("Creating reproducible document from {message}. Do you want to continue?"),
        AskOptions {
            level: AskLevel::Warning,
            default: Some(Answer::Yes),
            title: Some("Reproducible document creation".into()),
            yes_text: Some("Yes".into()),
            no_text: Some("No".into()),
            ..Default::default()
        },
    )
    .await?;

    Ok(answer.is_yes())
}

/// Check if a file has changed since a specific commit and optionally create a branch
///
/// This function compares a file's current state against a specified commit. If the file
/// has changed, it offers to create a new branch at that commit point, handling any
/// conflicting uncommitted changes by stashing them first.
///
/// # Workflow
///
/// 1. Check if the file has any changes since the specified commit
/// 2. If no changes, exit early with an info message
/// 3. If changes exist, prompt user (unless `force` is true) to create a branch
/// 4. If file has uncommitted changes that would conflict:
///    - Prompt user to stash changes or exit to commit manually (unless `force` is true)
///    - Stash changes if user agrees or if `force` is true
/// 5. Create a new branch with format `reverse-{path-kebab}-{commit-short}`
/// 6. Switch to the new branch at the specified commit
///
/// # Error Handling
///
/// - Git command failures are wrapped with context
/// - If branch creation fails after stashing, attempts to restore stashed changes
/// - User input errors are propagated with context
///
/// # Arguments
///
/// * `path` - Path to the file relative to the repository root
/// * `commit` - The commit hash or reference to compare against and branch from
/// * `force` - Skip all user prompts and automatically stash/create branch
///
/// # Returns
///
/// * `Ok(true)` - Completed successfully
/// * `Ok(false)` - Completed successfully but user cancelled the calling operation
/// * `Err(_)` - Git operations failed or user input could not be read
#[tracing::instrument]
#[must_use = "return boolean indicated if user cancelled the operation"]
async fn check_git_for_merge(path: &Path, commit: &str, other: &Path, force: bool) -> Result<bool> {
    let path_ = path.display();
    let file = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| path.to_string_lossy());

    let commit_short = &commit[..8.min(commit.len())];

    let other_file = other
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| other.to_string_lossy());

    // Check if file has changed since the specified commit
    tracing::debug!("Checking git diff for {path_} against commit {commit}");
    let diff_output = Command::new("git")
        .args(["diff", commit, "--", path.to_str().unwrap_or("")])
        .output()?;
    if !diff_output.status.success() {
        let error = String::from_utf8_lossy(&diff_output.stderr);
        bail!("Unable to check for changes in {path_} since commit {commit}: {error}");
    }

    if diff_output.stdout.is_empty() {
        tracing::debug!("No changes detected in {path_} since commit {commit}");
        return Ok(true);
    } else {
        tracing::debug!("File {path_} has changed since commit {commit}");
    }

    // Determine if we should create a branch
    tracing::debug!("File has changes, determining whether to create branch");
    if force {
        tracing::debug!("Force mode enabled, will create branch");
    } else {
        match ask_with(
            &format!("Source file `{file}` has changed since `{other_file}` was generated from it. Would you like to create a new branch at commit `{commit_short}` so edits can be applied correctly?"),
            AskOptions {
                level: AskLevel::Warning,
                default: Some(Answer::Yes),
                title: Some("Source has changed".into()),
                yes_text: Some("Yes, create a Git branch".into()),
                no_text: Some("No, just continue".into()),
                cancel_allowed: true
            }
        ).await? {
            Answer::Yes => {}, // continue below
            Answer::No => return Ok(true),
            Answer::Cancel => return Ok(false)
        }
    };

    // Check if the file has uncommitted changes
    tracing::debug!("Checking if file {path_} has uncommitted changes");
    let file_status_output = Command::new("git")
        .args(["status", "--porcelain", "--", path.to_str().unwrap_or("")])
        .output()?;
    if !file_status_output.status.success() {
        let error = String::from_utf8_lossy(&file_status_output.stderr);
        bail!("Failed to get file status: {error}");
    }

    let file_has_uncommitted_changes = !file_status_output.stdout.is_empty();
    let mut stashed = false;

    // Handle uncommitted changes if they exist
    if file_has_uncommitted_changes {
        tracing::debug!("File {path_} has uncommitted changes that conflict with target commit");

        if !force {
            match ask_with(
                &format!("Source file `{file}` has uncommitted changes. Would you like to stash changes before creating branch?"),
                AskOptions {
                    level: AskLevel::Warning,
                    default: Some(Answer::Yes),
                    title: Some("Uncommitted changes to source".into()),
                    yes_text: Some("Yes, create a Git stash".into()),
                    no_text: Some("No, I'll deal with conflicts".into()),
                    cancel_allowed: true
                }
            ).await? {
                Answer::Yes => {} // continue below
                Answer::No => return Ok(true),
                Answer::Cancel => return Ok(false)
            }
        }

        // Stash the changes
        tracing::debug!("Stashing uncommitted changes");
        let stash_output = Command::new("git")
            .args([
                "stash",
                "push",
                "-m",
                "WIP: auto-stash before branch creation",
            ])
            .output()?;
        if !stash_output.status.success() {
            let error = String::from_utf8_lossy(&stash_output.stderr);
            bail!("Failed to stash changes: {error}");
        }

        stashed = true;
    }

    // Generate a unique branch name (in case this function is run twice)
    let datetime = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    let branch_name = format!("merge-{other_file}-{datetime}");

    // Create and checkout the new branch at the specified commit
    tracing::debug!("Executing git checkout -b {} {}", branch_name, commit);
    let branch_result = Command::new("git")
        .args(["checkout", "-b", &branch_name, commit])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .wrap_err("Failed to execute git checkout")?;

    if !branch_result.success() {
        // If we stashed changes, try to restore them
        if stashed {
            tracing::debug!("Branch creation failed, attempting to restore stashed changes");
            let _ = Command::new("git")
                .args(["stash", "pop"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        bail!("Failed to create branch. The commit may not exist");
    }

    tracing::info!(
        "Successfully created and switched to branch '{}'",
        branch_name
    );
    Ok(true)
}

/// Pull a document from a remote service
///
/// Downloads the document from the remote service and saves it to the specified path.
#[tracing::instrument(skip(node))]
pub async fn push(
    service: &remotes::RemoteService,
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    url: Option<&Url>,
    doc_path: Option<&Path>,
) -> Result<Url> {
    service.push(node, path, title, url).await
}

/// Pull a document from a remote service and update a local file
///
/// If `merge` is true, merges the pulled version with the local file.
/// If false, replaces the local file with the pulled version.
///
/// Returns `Some(paths)` with modified file paths, or `None` if merge was cancelled.
#[tracing::instrument]
pub async fn pull(
    service: &remotes::RemoteService,
    url: &Url,
    dest: &Path,
    merge: bool,
    decode_options: DecodeOptions,
    encode_options: EncodeOptions,
) -> Result<Option<Vec<PathBuf>>> {
    // Create temp directory for the pulled version
    let temp_dir = tempdir()?;
    let format = service.pull_format();
    let pulled_path = temp_dir
        .path()
        .join("pulled")
        .with_extension(format.extension());

    // Download from remote service
    service.pull(url, &pulled_path).await?;

    if merge {
        // Merge the pulled version with the local file
        self::merge(
            &pulled_path,
            Some(dest),
            None,
            None,
            false,
            decode_options,
            encode_options,
            None,
        )
        .await
    } else {
        // Replace local file with pulled version
        self::convert(
            Some(&pulled_path),
            Some(dest),
            Some(decode_options),
            Some(encode_options),
        )
        .await?;
        Ok(Some(vec![dest.to_path_buf()]))
    }
}

/// A visitor that implements the `--recurse` encoding option by walking over
/// the a node and encoding any `IncludeBlock` nodes having `content` to their
/// `source` file.
struct Recurser {
    /// The path of the main file being encoded
    path: PathBuf,

    /// Encoding options
    options: Option<EncodeOptions>,
}

impl VisitorAsync for Recurser {
    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        if let Block::IncludeBlock(IncludeBlock {
            source,
            content: Some(content),
            ..
        }) = block
        {
            let path = self
                .path
                .canonicalize()?
                .parent()
                .ok_or_eyre("no parent")?
                .join(source);

            let node = Node::Article(Article {
                content: content.clone(),
                ..Default::default()
            });

            let format = Format::from_path(&path);

            let options = EncodeOptions {
                standalone: Some(false),
                format: Some(format),
                ..self.options.clone().unwrap_or_default()
            };

            to_path(&node, &path, Some(options)).await?;
        }

        Ok(WalkControl::Continue)
    }
}
