use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use codec::common::eyre::Context;
use codec::common::inflector::Inflector;
use codec::{
    common::{
        eyre::{bail, eyre, OptionExt, Result},
        reqwest::Client,
        tempfile::tempdir,
        tokio::fs::{read_to_string, write},
        tracing,
    },
    schema::{Article, Block, IncludeBlock, Node, VisitorAsync, WalkControl, WalkNode},
};
pub use codec::{
    format::Format, Codec, CodecDirection, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo,
    EncodeOptions, Losses, LossesResponse, Mapping, MappingEntry, Message, MessageLevel, Messages,
    PoshMap, Position16, Position8, Positions, Range16, Range8,
};
use codec_utils::lift_edits;
use node_strip::{StripNode, StripTargets};
use walkdir::WalkDir;

pub mod cli;

/// Get a list of all codecs
pub fn list() -> Vec<Box<dyn Codec>> {
    let codecs = vec![
        Box::new(codec_cbor::CborCodec) as Box<dyn Codec>,
        Box::new(codec_debug::DebugCodec),
        Box::new(codec_docx::DocxCodec),
        // DomCodec supports to HTML and because listed here before HtmlCodec
        // will be selected when encoding to HTML
        Box::new(codec_dom::DomCodec),
        Box::new(codec_directory::DirectoryCodec),
        Box::new(codec_gdocx::GDocxCodec),
        Box::new(codec_html::HtmlCodec),
        Box::new(codec_ipynb::IpynbCodec),
        Box::new(codec_jats::JatsCodec),
        Box::new(codec_json::JsonCodec),
        Box::new(codec_json5::Json5Codec),
        Box::new(codec_jsonld::JsonLdCodec),
        Box::new(codec_latex::LatexCodec),
        Box::new(codec_lexical::LexicalCodec),
        Box::new(codec_markdown::MarkdownCodec),
        Box::new(codec_rnw::RnwCodec),
        Box::new(codec_odt::OdtCodec),
        Box::new(codec_pandoc::PandocCodec),
        Box::new(codec_pmcoap::PmcOapCodec),
        Box::new(codec_pdf::PdfCodec),
        Box::new(codec_png::PngCodec),
        Box::<codec_swb::SwbCodec>::default(),
        Box::new(codec_text::TextCodec),
        Box::new(codec_yaml::YamlCodec),
    ];

    // TODO: make plugins a dependency and append codecs to list
    //let provided_by_plugins = &mut plugins::codecs::list();
    //codecs.append(provided_by_plugins);

    codecs
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

/// Reverse changes from an edited document (in a different format) into the original
/// document (in a different format)
#[tracing::instrument]
pub async fn reverse(
    edited: &Path,
    original: Option<&Path>,
    unedited: Option<&Path>,
    commit: Option<&str>,
    decode_options: DecodeOptions,
    mut encode_options: EncodeOptions,
    workdir: Option<PathBuf>,
) -> Result<()> {
    // Create, or use specified, working directory
    let tempdir = tempdir()?;
    let workdir = if let Some(workdir) = &workdir {
        workdir
    } else {
        tempdir.path()
    };

    // Decode the edited file because it may contain information on the
    // original source & commit not supplied as an argument
    let (edited_node, unedited_node, ..) =
        from_path_with_info(edited, Some(decode_options.clone())).await?;

    let mut original = original.map(|path| path.to_path_buf());
    if original.is_none() {
        if let Node::Article(Article { options, .. }) = &edited_node {
            if let Some(source) = &options.source {
                original = Some(PathBuf::from(source));
            }
        }
    }
    let Some(original) = original else {
        bail!("Relative path of original source file not specified and not available from edited document")
    };

    let mut commit = commit.map(String::from);
    if commit.is_none() {
        if let Node::Article(Article { options, .. }) = &edited_node {
            if let Some(value) = &options.commit {
                commit = Some(value.to_string());
            }
        }
    }

    // If a commit is available then check the status of the file relative to the path
    if let Some(commit) = commit {
        if commit != "untracked" && commit != "dirty" {
            check_git_status(&original, &commit, false)?;
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
        to_path(&unedited_node, &unedited_file, Some(encode_options)).await?
    }

    // Rebase edits for each file in the `edited` directory.
    for entry in WalkDir::new(&edited_dir)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let edited_path = entry.path();
        if !edited_path.is_file() {
            continue;
        }

        let edited = read_to_string(edited_path).await?;

        let relative_path = edited_path
            .strip_prefix(&edited_dir)
            .expect("not in edited dir");
        let unedited_path = unedited_dir.join(relative_path);
        let original_path = original_dir.join(relative_path);

        // If a file exists in the edited dir but not in the unedited then just
        // write it to the original dir.
        if !unedited_path.exists() {
            write(original_path, edited).await?;
            continue;
        }

        let unedited = read_to_string(unedited_path).await?;
        if edited == unedited {
            tracing::trace!("No changes, skipping `{}`", relative_path.display());
            continue;
        }

        let original = read_to_string(&original_path).await?;

        tracing::debug!("Merging `{}`", relative_path.display());
        let merged = lift_edits(&original, &unedited, &edited);

        write(original_path, merged).await?;
    }

    Ok(())
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
/// * `Ok(())` - Operation completed successfully (including early exits)
/// * `Err(_)` - Git operations failed or user input could not be read
#[allow(clippy::print_stdout)]
#[tracing::instrument]
fn check_git_status(path: &Path, commit: &str, force: bool) -> Result<()> {
    let path_ = path.display();
    let commit_short = &commit[..8.min(commit.len())];

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
        return Ok(());
    } else {
        tracing::debug!("File {path_} has changed since commit {commit}");
    }

    // Determine if we should create a branch
    tracing::debug!("File has changes, determining whether to create branch");
    let should_create_branch = if force {
        tracing::debug!("Force mode enabled, will create branch");
        true
    } else {
        println!("Original source file `{path_}` has changed since the edited file was generated from it.");
        print!("Would you like to create a new branch at commit `{commit_short}` so edits can be applied correctly? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .wrap_err("Failed to read user input")?;

        let input = input.trim().to_lowercase();
        let result = input == "y" || input == "yes";
        tracing::debug!("User input: '{}', will create branch: {}", input, result);
        result
    };

    if !should_create_branch {
        tracing::info!("Branch creation cancelled");
        return Ok(());
    }

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
            println!("File `{path_}` has uncommitted changes.");
            print!("Would you like to stash changes before creating branch? (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .wrap_err("Failed to read user input")?;

            let input = input.trim().to_lowercase();
            let should_stash = input == "y" || input == "yes";
            if !should_stash {
                println!("Please commit your changes first, then run this command again.");
                return Ok(());
            }
        }

        // Stash the changes
        tracing::info!("Stashing uncommitted changes");
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

    // Generate a branch name based on the path and the commit hash
    let path_kebab = path.to_string_lossy().to_kebab_case();
    let branch_name = format!("reverse-{path_kebab}-{commit_short}");

    // Create and checkout the new branch at the specified commit
    tracing::debug!("Executing git checkout -b {} {}", branch_name, commit);
    tracing::info!("Creating branch '{}' at commit {}", branch_name, commit);
    let branch_result = Command::new("git")
        .args(["checkout", "-b", &branch_name, commit])
        .status()
        .wrap_err("Failed to execute git checkout")?;

    if !branch_result.success() {
        // If we stashed changes, try to restore them
        if stashed {
            tracing::debug!("Branch creation failed, attempting to restore stashed changes");
            let _ = Command::new("git").args(["stash", "pop"]).status();
        }
        bail!("Failed to create branch. The commit may not exist");
    }

    tracing::info!(
        "Successfully created and switched to branch '{}'",
        branch_name
    );
    Ok(())
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
