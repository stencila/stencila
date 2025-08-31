use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

use codec::PageSelector;
use common::{
    eyre::{Context, OptionExt, Report, Result, bail},
    reqwest, seahash, serde_json,
    strum::{Display, EnumIter, IntoEnumIterator},
    tempfile::tempdir,
    tokio::fs::{read, read_to_string, write},
    tracing,
};
use dirs::closest_artifacts_for;
use secrets::MISTRAL_API_KEY;
use tools::{AsyncToolCommand, is_installed};

use crate::md_to_md::{clean_md, clean_md_page};

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

// Prefixes used for artifact directories cp2m = "convert PDF to Markdown"

const MISTRAL_ARTIFACT_PREFIX: &str = "cp2mmi";

impl FromStr for Tool {
    type Err = Report;

    fn from_str(tool: &str) -> Result<Self, Self::Err> {
        use Tool::*;
        Ok(match tool.to_lowercase().as_str() {
            "mineru" => Mineru,
            "marker" => Marker,
            "mistral" => Mistral,
            _ => bail!("Unrecognized PDF to Markdown tool: {tool}"),
        })
    }
}

/// Convert a PDF file to a Markdown file
#[tracing::instrument]
pub async fn pdf_to_md(
    pdf: &Path,
    tool: Option<&str>,
    include_pages: Option<&Vec<PageSelector>>,
    exclude_pages: Option<&Vec<PageSelector>>,
) -> Result<PathBuf> {
    let tool = match tool {
        // Use the specified tool
        Some(tool) => Tool::from_str(tool)?,
        None => {
            if secrets::env_or_get(MISTRAL_API_KEY).is_ok() {
                // If a Mistral API key is available, use that
                Tool::Mistral
            } else {
                // Check if any of the local tools are available, defaulting to Mineru
                let mut tool = Tool::Mineru;
                for tool_ in Tool::iter() {
                    if is_installed(&tool_.to_string())? {
                        tool = tool_;
                        break;
                    }
                }
                tool
            }
        }
    };

    match tool {
        Tool::Mistral => pdf_to_md_mistral(pdf, include_pages, exclude_pages).await,
        _ => pdf_to_md_local(pdf, tool, include_pages, exclude_pages).await,
    }
}

/// Convert a PDF file to a Markdown file using a local tool
#[tracing::instrument]
pub async fn pdf_to_md_local(
    pdf: &Path,
    tool: Tool,
    include_pages: Option<&Vec<PageSelector>>,
    exclude_pages: Option<&Vec<PageSelector>>,
) -> Result<PathBuf> {
    let out_dir = tempdir()?.keep();

    // TODO: Implement page filtering for local tools
    // For now, local tools process all pages - page filtering would require
    // either tool-specific page range options or post-processing the Markdown
    if include_pages.is_some() || exclude_pages.is_some() {
        tracing::warn!(
            "Page filtering is not yet supported for local PDF tools, processing all pages"
        );
    }

    let mut command = AsyncToolCommand::new(tool.to_string());

    match tool {
        Tool::Mineru => command.arg("--path").arg(pdf).arg("--output").arg(&out_dir),

        Tool::Marker => command
            .arg(pdf)
            .args(vec!["--output_format", "markdown", "--output_dir"])
            .arg(&out_dir),

        _ => bail!("Non-local PDF to Markdown tool `{tool}`"),
    };

    tracing::info!("Converting PDF to Markdown locally using `{tool}`; this may take some time");

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

/// Convert a PDF file to a Markdown file using Mistral OCR API
#[tracing::instrument]
pub async fn pdf_to_md_mistral(
    pdf_path: &Path,
    include_pages: Option<&Vec<PageSelector>>,
    exclude_pages: Option<&Vec<PageSelector>>,
) -> Result<PathBuf> {
    // Read PDF
    let pdf_bytes = read(pdf_path).await?;

    // Get / create a new artifacts directory
    let digest = seahash::hash(&pdf_bytes);
    let key = format!("{MISTRAL_ARTIFACT_PREFIX}-{digest:x}");
    let artifacts_path = closest_artifacts_for(pdf_path, &key).await?;

    // Read or get response JSON
    let response_path = artifacts_path.join("response.json");
    let response_json = if response_path.exists() {
        read_to_string(response_path).await?
    } else {
        // Get API key
        let api_key = secrets::env_or_get(MISTRAL_API_KEY)?;

        // Send request
        tracing::info!("Converting PDF to Markdown using Mistral OCR; this may take some time");
        let client = reqwest::Client::new();
        let pdf_base64 = general_purpose::STANDARD.encode(&pdf_bytes);
        let payload = MistralOcrRequest::new(&pdf_base64);
        let response = client
            .post("https://api.mistral.ai/v1/ocr")
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        // Bail on fail
        if !response.status().is_success() {
            let error_text = response.text().await?;
            bail!("Mistral OCR request failed: {}", error_text);
        }

        // Store response text
        let json = response.text().await?;
        write(response_path, &json).await?;

        json
    };

    // Parse response JSON
    let response: MistralOcrResponse = serde_json::from_str(&response_json)?;

    // Resolve which pages to include based on total page count
    let total_pages = response.pages.len();
    let pages_to_include = PageSelector::resolve_include_exclude(
        include_pages.map(|v| v.as_slice()),
        exclude_pages.map(|v| v.as_slice()),
        total_pages,
    );

    // Accumulate Markdown and save images from selected pages only
    let mut md = String::new();
    let mut first_included_page = true;
    for (index, page) in response.pages.into_iter().enumerate() {
        let page_number = index + 1; // Convert to 1-based page numbering

        // Skip pages that are not in the resolved set
        if !pages_to_include.contains(&page_number) {
            continue;
        }

        if !first_included_page {
            // Note: only one newline here to avoid splitting paragraphs unnecessarily
            md.push('\n');
        }
        first_included_page = false;
        let cleaned_page = clean_md_page(&page.markdown);
        md.push_str(&cleaned_page);

        // Write Base64 encoded images to directory
        for image in page.images {
            if let Some(mut image_base64) = image.image_base64 {
                if let Some(pos) = image_base64.find(";base64,") {
                    image_base64 = image_base64[(pos + 8)..].to_string();
                }

                let image_path = artifacts_path.join(&image.id);
                let image_bytes = general_purpose::STANDARD
                    .decode(&image_base64)
                    .wrap_err_with(|| {
                        format!(
                            "Unable to decode Base64 image `{}`: {}...",
                            image.id,
                            &image_base64[..image_base64.len().min(20)]
                        )
                    })?;
                write(&image_path, image_bytes).await?;
            }
        }
    }

    // Clean the accumulated Markdown
    let cleaned_md = clean_md(&md);

    // Write the cleaned Markdown to file
    let md_path = artifacts_path.join("output.md");
    write(&md_path, cleaned_md).await?;

    Ok(md_path)
}

#[derive(Serialize)]
struct MistralOcrRequest {
    model: String,

    document: MistralOcrDocument,

    #[serde(skip_serializing_if = "Option::is_none")]
    include_image_base64: Option<bool>,
}

impl MistralOcrRequest {
    fn new(pdf_base64: &str) -> Self {
        Self {
            model: "mistral-ocr-latest".to_string(),
            document: MistralOcrDocument {
                doc_type: "document_url".to_string(),
                document_url: format!("data:application/pdf;base64,{pdf_base64}"),
            },
            include_image_base64: Some(true),
        }
    }
}

#[derive(Serialize)]
struct MistralOcrDocument {
    #[serde(rename = "type")]
    doc_type: String,

    document_url: String,
}

#[derive(Deserialize)]
struct MistralOcrResponse {
    pages: Vec<MistralOcrPage>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct MistralOcrPage {
    index: usize,

    markdown: String,

    #[serde(default)]
    images: Vec<MistralOcrImage>,

    dimensions: MistralOcrDimensions,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct MistralOcrImage {
    id: String,

    top_left_x: f64,
    top_left_y: f64,
    bottom_right_x: f64,
    bottom_right_y: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    image_base64: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct MistralOcrDimensions {
    dpi: f64,
    height: f64,
    width: f64,
}
