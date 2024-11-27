use std::{path::Path, process::Stdio};

use codec::common::{
    eyre::{bail, Result},
    serde_json,
    tokio::{io::AsyncWriteExt, process::Command},
};
use pandoc_types::definition::Pandoc;

/// The semver requirement for Pandoc.
///
/// Note that this is a semver *requirement*, so higher versions of Pandoc
/// that meet this, should still work.
///
/// This is mostly based on compatibility with the `pandoc_types` crate.
/// Some recent changes to the pandoc-types versions used by Pandoc (from https://pandoc.org/releases.html):
///
///   pandoc 3.5 (?upgrade may have been in earlier version) : pandoc-types 1.23.1
///   pandoc 2.11 (2020-10-11) : pandoc-types 1.22
///   pandoc 2.10 (2020-06-29) : pandoc-types 1.21
///
/// If/when there are future changes the `pandoc-types` version used in Pandoc itself
/// then this semver requirement will need to be updated (i.e. be given an upper bound
/// or `pandoc_types` crate updated and the lower bound raised)

/// Call Pandoc binary to convert some input content to Pandoc JSON.
pub async fn pandoc_from_format(
    input: &str,
    path: Option<&Path>,
    format: &str,
    args: Vec<String>,
) -> Result<Pandoc> {
    let json = if format == "pandoc" {
        input.to_string()
    } else {
        let mut command = Command::new("pandoc");
        command
            .args(["--from", format, "--to", "json"])
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = if let Some(path) = path {
            if !path.exists() {
                bail!("File does not exists: {}", path.to_string_lossy())
            }
            command.arg(path).spawn()?
        } else {
            let mut child = command.stdin(Stdio::piped()).spawn()?;
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
pub async fn pandoc_to_format(
    pandoc: &Pandoc,
    path: Option<&Path>,
    format: &str,
    args: Vec<String>,
) -> Result<String> {
    let json = serde_json::to_string(&pandoc)?;

    if format == "pandoc" {
        return Ok(json.to_string());
    }

    let mut command = Command::new("pandoc");
    command.args(["--from", "json", "--to", format]);
    command.args(args);
    if let Some(path) = &path {
        command.args(["--output", &path.to_string_lossy()]);
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()?;
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
