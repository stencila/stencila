use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use common::{
    eyre::{OptionExt, Report, Result, bail},
    strum::{Display, EnumIter, IntoEnumIterator},
    tempfile::tempdir,
    tracing,
};
use tools::{AsyncToolCommand, is_installed};

#[derive(Debug, Display, EnumIter)]
#[strum(crate = "common::strum")]
enum Tool {
    #[strum(serialize = "mineru")]
    Mineru,

    #[strum(serialize = "marker_single")]
    Marker,

    #[strum(serialize = "mistral")]
    Mistral,
}

impl FromStr for Tool {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Tool::*;
        Ok(match s.to_lowercase().as_str() {
            "mineru" => Mineru,
            "marker" => Marker,
            "mistral" => Mistral,
            _ => bail!("Unrecognized PDF to Markdown tool"),
        })
    }
}

/// Convert a PDF file to a Markdown file
#[tracing::instrument]
pub async fn pdf_to_md(pdf: &Path, tool: Option<&str>) -> Result<PathBuf> {
    // TODO: remove
    return pdf_to_md_mistral(pdf).await;

    let tool = match tool {
        Some(tool) => Tool::from_str(tool)?,
        None => {
            let mut tool = Tool::Mineru;
            for tool_ in Tool::iter() {
                if is_installed(&tool_.to_string())? {
                    tool = tool_;
                    break;
                }
            }
            tool
        }
    };

    match tool {
        Tool::Mistral => pdf_to_md_mistral(pdf).await,
        _ => pdf_to_md_local(pdf, tool).await,
    }
}

/// Convert a PDF file to a Markdown file using a local tool
#[tracing::instrument]
pub async fn pdf_to_md_mistral(pdf: &Path) -> Result<PathBuf> {
    bail!("Mistral PDF to Markdown not yet implemented")
}

/// Convert a PDF file to a Markdown file using a local tool
#[tracing::instrument]
pub async fn pdf_to_md_local(pdf: &Path, tool: Tool) -> Result<PathBuf> {
    let out_dir = tempdir()?.keep();

    let mut command = AsyncToolCommand::new(tool.to_string());

    match tool {
        Tool::Mineru => command.arg("--path").arg(pdf).arg("--output").arg(&out_dir),

        Tool::Marker => command
            .arg(pdf)
            .args(vec!["--output_format", "markdown", "--output_dir"])
            .arg(&out_dir),

        _ => bail!("Non-local PDF to Markdown tool `{tool}`"),
    };

    tracing::info!("Converting PDF to Markdown using `{tool}`; this may take some time");
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

    let path = match tool {
        Tool::Mineru => out_dir.join(file_stem).join("auto").join(file_name),
        Tool::Marker => out_dir.join(file_stem).join(file_name),
        _ => bail!("Non-local PDF to Markdown tool `{tool}`"),
    };

    tracing::debug!("Converted PDF to {}", path.display());

    Ok(path)
}
