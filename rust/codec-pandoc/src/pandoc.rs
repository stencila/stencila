//! The semver requirement for Pandoc.
//!
//! Note that this is a semver *requirement*, so higher versions of Pandoc
//! that meet this, should still work.
//!
//! This is mostly based on compatibility with the `pandoc_types` crate.
//! Some recent changes to the pandoc-types versions used by Pandoc (from https://pandoc.org/releases.html):
//!
//!   pandoc 3.5 (?upgrade may have been in earlier version) : pandoc-types 1.23.1
//!   pandoc 2.11 (2020-10-11) : pandoc-types 1.22
//!   pandoc 2.10 (2020-06-29) : pandoc-types 1.21
//!
//! If/when there are future changes the `pandoc-types` version used in Pandoc itself
//! then this semver requirement will need to be updated (i.e. be given an upper bound
//! or `pandoc_types` crate updated and the lower bound raised)

use std::path::Path;

use pandoc_types::definition::Pandoc;

use codec::{
    DecodeOptions, EncodeOptions,
    eyre::{Result, bail},
    format::Format,
};
use tokio::io::AsyncWriteExt;
use tools::{Pandoc as PandocTool, Tool, ToolStdio};

/// Call Pandoc binary to convert some input content to Pandoc JSON.
#[tracing::instrument(skip(input))]
pub async fn pandoc_from_format(
    input: &str,
    path: Option<&Path>,
    format: &str,
    options: &Option<DecodeOptions>,
) -> Result<Pandoc> {
    let json = if format == "pandoc" {
        input.to_string()
    } else {
        tracing::debug!("Spawning pandoc to parse `{format}`");

        let mut args = options
            .as_ref()
            .map(|options| options.tool_args.clone())
            .unwrap_or_default();

        // Some codecs use the `--pandoc` to indicate that pandoc should be used
        // instead of the default decoding so remove that.
        args.retain(|arg| arg != "--pandoc");

        let mut command = PandocTool.async_command();
        command
            .args(["--from", format, "--to", "json"])
            .args(args)
            .stdout(ToolStdio::Piped)
            .stderr(ToolStdio::Piped);

        let child = if let Some(path) = path {
            if !path.exists() {
                bail!("File does not exists: {}", path.to_string_lossy())
            }
            command.arg(path).spawn().await?
        } else {
            let mut child = command.stdin(ToolStdio::Piped).spawn().await?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input.as_ref()).await?;
            }
            child
        };

        let result = child.wait_with_output().await?;

        if !result.status.success() {
            let error = String::from_utf8(result.stderr)?;
            bail!("While importing from format `{format}` using Pandoc: {error}")
        }

        String::from_utf8(result.stdout)?
    };

    let pandoc = serde_json::from_str(&json)?;
    Ok(pandoc)
}

/// Call Pandoc binary to convert Pandoc JSON to some output format
#[tracing::instrument(skip(pandoc))]
pub async fn pandoc_to_format(
    pandoc: &Pandoc,
    path: Option<&Path>,
    format: &str,
    options: &Option<EncodeOptions>,
) -> Result<String> {
    let json = serde_json::to_string(&pandoc)?;

    if format == "pandoc" {
        return Ok(json.to_string());
    }

    tracing::debug!("Spawning pandoc to generate `{format}`");

    let options = options.clone().unwrap_or_default();
    let mut args = options.tool_args.clone();

    // Some codecs use the `--pandoc` to indicate that pandoc should be used
    // instead of the default encoding so remove that.
    args.retain(|arg| arg != "--pandoc");

    // Translate `template` option to Pandoc argument
    if let Some(template) = options.template {
        args.push(format!("--reference-doc={}", template.to_string_lossy()));
    }
    let mut command = PandocTool.async_command();
    command.args(args);
    command.args(["--from", "json", "--to", format]);
    if let Some(path) = &path {
        command.args(["--output", &path.to_string_lossy()]);
    }

    let mut child = command
        .stdout(ToolStdio::Piped)
        .stderr(ToolStdio::Piped)
        .stdin(ToolStdio::Piped)
        .spawn()
        .await?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json.as_ref()).await?;
    }

    let result = child.wait_with_output().await?;

    if !result.status.success() {
        let error = String::from_utf8(result.stderr)?;
        bail!("While exporting to format `{format}` using Pandoc: {error}")
    }

    if let Some(path) = path {
        Ok(path.to_string_lossy().to_string())
    } else {
        let stdout = String::from_utf8(result.stdout)?;
        Ok(stdout)
    }
}

/// Encode content to a path
#[tracing::instrument(skip(content))]
pub(crate) async fn format_to_path(
    from: &Format,
    to: &Format,
    content: &str,
    path: &Path,
    options: &Option<EncodeOptions>,
) -> Result<()> {
    tracing::debug!("Spawning pandoc to create {}", path.display());

    let options = options.clone().unwrap_or_default();
    let mut args = options.tool_args.clone();

    // Translate `template` option to Pandoc argument
    if let Some(template) = options.template {
        args.push(format!("--reference-doc={}", template.to_string_lossy()));
    }

    let mut command = PandocTool.async_command();
    command.args(args);
    command.args([
        "--from",
        &from.to_string(),
        "--to",
        &to.to_string(),
        "--output",
        &path.to_string_lossy(),
    ]);

    let mut child = command
        .stdout(ToolStdio::Piped)
        .stderr(ToolStdio::Piped)
        .stdin(ToolStdio::Piped)
        .spawn()
        .await?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_ref()).await?;
    }

    let result = child.wait_with_output().await?;

    if !result.status.success() {
        let error = String::from_utf8(result.stderr)?;
        bail!("Pandoc error: {error}")
    }

    Ok(())
}
