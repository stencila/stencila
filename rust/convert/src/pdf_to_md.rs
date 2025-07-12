use std::path::{Path, PathBuf};

use common::{
    eyre::{OptionExt, Result, bail},
    strum::{Display, EnumIter, IntoEnumIterator},
    tempfile::tempdir,
    tracing,
};
use tools::{AsyncToolCommand, is_installed};

#[derive(Display, EnumIter)]
#[strum(crate = "common::strum")]
enum Tool {
    #[strum(serialize = "mineru")]
    Mineru,

    #[strum(serialize = "marker_single")]
    Marker,
}

/// Convert a PDF file to a Markdown file
#[tracing::instrument]
pub async fn pdf_to_md(pdf: &Path) -> Result<PathBuf> {
    let mut tool = Tool::Mineru;
    for tool_ in Tool::iter() {
        if is_installed(&tool_.to_string())? {
            tool = tool_;
            break;
        }
    }

    let out_dir = tempdir()?.keep();

    let mut command = AsyncToolCommand::new(&tool.to_string());

    match tool {
        Tool::Mineru => command
            .arg("--path")
            .arg(&pdf)
            .arg("--output")
            .arg(&out_dir),

        Tool::Marker => command
            .arg(&pdf)
            .args(vec!["--output_format", "markdown", "--output_dir"])
            .arg(&out_dir),
    };

    tracing::debug!("Running `{tool}`");
    let output = command.output().await?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("PDF to Markdown conversion using `{tool}` failed:\n\n{stdout}\n\n{stderr}")
    }

    let file_stem = pdf
        .file_stem()
        .ok_or_eyre("PDF path has no file stem")?
        .to_os_string();

    let mut file_name = file_stem.clone();
    file_name.push(".md");

    Ok(match tool {
        Tool::Mineru => out_dir.join(file_stem).join("auto").join(file_name),
        Tool::Marker => out_dir.join(file_stem).join(file_name),
    })
}
