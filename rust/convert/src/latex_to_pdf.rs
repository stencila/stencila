use std::path::{Path, PathBuf};

use eyre::{Result, bail};
use glob::glob;
use rand::{Rng, distr::Alphanumeric, rng};
use tokio::fs::{create_dir_all, read_to_string, remove_file, write};

use stencila_codec_utils::move_file;
use stencila_tools::{Tool, Xelatex};

/// Convert a LaTeX string to a PDF file
#[tracing::instrument(skip(latex))]
pub async fn latex_to_pdf(latex: &str, path: &Path) -> Result<()> {
    // Use a unique job name to be able to run `latex` in the current working directory
    // (because paths in \input and \includegraphics commands are relative to that)
    // whilst also being able to clean up temporary file afterwards
    let job = [
        "temp-",
        rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>()
            .as_str(),
    ]
    .concat();

    let input_file = format!("{job}.tex");
    write(&input_file, latex).await?;

    let status = Xelatex
        .async_command()
        .args([
            "-interaction=batchmode",
            "-halt-on-error",
            "-jobname",
            &job,
            &input_file,
        ])
        .status()
        .await?;

    let output_file = PathBuf::from(format!("{job}.pdf"));
    if output_file.exists() {
        if let Some(dir) = path.parent() {
            create_dir_all(dir).await?;
        }
        move_file(output_file, path)?;
    }

    let log_file = PathBuf::from(format!("{job}.log"));
    let log = if log_file.exists() {
        read_to_string(log_file).await?
    } else {
        String::new()
    };

    for path in glob(&format!("{job}.*"))?.flatten() {
        remove_file(path).await?;
    }

    if !status.success() {
        bail!("LaTeX to PDF conversion failed:\n\n{}", log);
    }

    Ok(())
}
