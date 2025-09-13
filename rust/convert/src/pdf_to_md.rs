use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose};
use eyre::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use tokio::fs::{read, read_to_string, write};

use stencila_codec::PageSelector;
use stencila_dirs::closest_artifacts_for;
use stencila_secrets::MISTRAL_API_KEY;

use crate::md_to_md::{clean_md, clean_md_page};

/// Convert a PDF file to a Markdown file using Mistral OCR API
#[tracing::instrument]
pub async fn pdf_to_md(
    pdf_path: &Path,
    include_pages: Option<&Vec<PageSelector>>,
    exclude_pages: Option<&Vec<PageSelector>>,
    ignore_artifacts: bool,
    no_artifacts: bool,
) -> Result<PathBuf> {
    // Read PDF
    let pdf_bytes = read(pdf_path).await?;

    // Create temporary directory (must be kept alive for entire function)
    let temp_dir = tempdir()?;

    // Determine where to store/look for artifacts
    let artifacts_path = if no_artifacts {
        // Don't cache, use temporary directory
        temp_dir.path().to_path_buf()
    } else {
        // Use artifacts directory for caching using PDF hash digest as key
        let digest = seahash::hash(&pdf_bytes);
        let key = format!("pdfmd-{digest:x}");
        closest_artifacts_for(&current_dir()?, &key).await?
    };

    // Read or get response JSON
    let response_path = artifacts_path.join("response.json");
    let should_fetch = !response_path.exists() || ignore_artifacts;
    let response_json = if should_fetch {
        // Get API key
        let api_key = stencila_secrets::env_or_get(MISTRAL_API_KEY)?;

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

        let json = response.text().await?;

        // Store response JSON (only if not using temp directory)
        if !no_artifacts {
            write(&response_path, &json).await?;
        }

        json
    } else {
        read_to_string(response_path).await?
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
