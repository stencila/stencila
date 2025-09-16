use std::{env::current_dir, path::Path};

use stencila_dirs::closest_artifacts_for;
use tempfile::tempdir;
use url::Url;

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::{Result, bail},
    stencila_schema::Node,
};

use crate::html::download_html;
use crate::tar::download_tar;

use super::html::decode_html;
use super::tar::decode_tar;

/// Extract and PMCID from an identifier
pub(super) fn extract_pmcid(identifier: &str) -> Option<String> {
    // Match PMC IDs like "PMC1234567" (at least 4 digits)
    if identifier.len() >= 7 && identifier.starts_with("PMC") {
        let number_part = &identifier[3..];
        if number_part.len() >= 4 && number_part.chars().all(|c| c.is_ascii_digit()) {
            return Some(identifier.to_string());
        }
    }

    // Match PMC URLs like "https://pmc.ncbi.nlm.nih.gov/articles/PMC1234567/"
    if let Ok(url) = Url::parse(identifier)
        && url.host_str() == Some("pmc.ncbi.nlm.nih.gov")
        && url.path().starts_with("/articles/PMC")
    {
        // Extract PMC ID from the URL path
        let path = url.path();
        if let Some(pmc_start) = path.find("PMC") {
            let pmc_part = &path[pmc_start..];
            // Find the end of the PMC ID (either end of string or next slash)
            let pmc_end = pmc_part.find('/').unwrap_or(pmc_part.len());
            let pmcid = &pmc_part[..pmc_end];

            // Validate it's a proper PMC ID (PMC + at least 4 digits)
            if pmcid.len() >= 7 && pmcid.starts_with("PMC") {
                let number_part = &pmcid[3..];
                if number_part.len() >= 4 && number_part.chars().all(|c| c.is_ascii_digit()) {
                    return Some(pmcid.to_string());
                }
            }
        }
    }

    None
}

/// Decode a PMCID to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_pmcid(
    pmcid: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let pmcid = pmcid.trim();
    if !pmcid.starts_with("PMC") {
        bail!("Unrecognized article id, should be a PMC id, starting with `PMC`")
    }

    let ignore_artifacts = options
        .as_ref()
        .and_then(|opts| opts.ignore_artifacts)
        .unwrap_or_default();
    let no_artifacts = options
        .as_ref()
        .and_then(|opts| opts.no_artifacts)
        .unwrap_or_default();

    let tar_filename = format!("{pmcid}.tar.gz");
    let html_filename = format!("{pmcid}.html");

    // Create temporary directory (must be kept alive for entire function)
    let temp_dir = tempdir()?;

    // Determine where to store/look for the downloaded files
    let (tar_path, html_path) = if no_artifacts {
        // Don't cache, use temporary directory
        (
            temp_dir.path().join(&tar_filename),
            temp_dir.path().join(&html_filename),
        )
    } else {
        // Use artifacts directory for caching
        let artifacts_key = format!("pmcoa-{}", pmcid.trim_start_matches("PMC"));
        let artifacts_dir = closest_artifacts_for(&current_dir()?, &artifacts_key).await?;
        (
            artifacts_dir.join(&tar_filename),
            artifacts_dir.join(&html_filename),
        )
    };

    // Try to download tar package first, fall back to HTML if that fails
    let should_download_tar = !tar_path.exists() || ignore_artifacts;
    let should_download_html = !html_path.exists() || ignore_artifacts;

    let file_path = if should_download_tar {
        // Try to download tar package first
        match download_tar(pmcid, &tar_path).await {
            Ok(()) => {
                tracing::debug!("Successfully downloaded tar package for {pmcid}");
                tar_path
            }
            Err(tar_error) => {
                tracing::debug!("Failed to download tar package for {pmcid}: {tar_error}");

                if should_download_html {
                    tracing::debug!("Falling back to HTML download for {pmcid}");
                    download_html(pmcid, &html_path).await?;
                }
                html_path
            }
        }
    } else if tar_path.exists() {
        // Use existing tar package
        tar_path
    } else if should_download_html {
        // No tar package exists, download HTML
        tracing::debug!("No tar package found, downloading HTML for {pmcid}");
        download_html(pmcid, &html_path).await?;
        html_path
    } else {
        // Use existing HTML file
        html_path
    };

    // Decode the downloaded file
    let (node, .., info) = decode_path(&file_path, options).await?;

    Ok((node, info))
}

/// Decode a PMC OA Package or HTML file to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => decode_html(path, options),
        Some("gz") => decode_tar(path, options).await,
        _ => bail!("Unhandled file extension"),
    }
}
