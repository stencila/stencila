use std::{env::current_dir, path::Path};

use flate2::read::GzDecoder;
use futures::StreamExt;
use glob::glob;
use regex::Regex;
use reqwest::{Client, header::USER_AGENT};
use stencila_dirs::closest_artifacts_for;
use stencila_node_supplements::embed_supplements;
use tar::Archive;
use tempfile::tempdir;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{ContextCompat, OptionExt, Result, bail, eyre},
    stencila_schema::Node,
};
use stencila_codec_jats::JatsCodec;
use stencila_node_media::embed_media;
use stencila_version::STENCILA_USER_AGENT;

use super::decode_html;

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

    let package_filename = format!("{pmcid}.tar.gz");

    // Create temporary directory (must be kept alive for entire function)
    let temp_dir = tempdir()?;

    // Determine where to store/look for the downloaded package
    let package_path = if no_artifacts {
        // Don't cache, use temporary directory
        temp_dir.path().join(&package_filename)
    } else {
        // Use artifacts directory for caching
        let artifacts_key = format!("pmcoa-{}", pmcid.trim_start_matches("PMC"));
        let artifacts_dir = closest_artifacts_for(&current_dir()?, &artifacts_key).await?;
        artifacts_dir.join(&package_filename)
    };

    // Download the package if needed
    let should_download = !package_path.exists() || ignore_artifacts;
    if should_download {
        download_package(pmcid, &package_path).await?;
    }

    // Decode package
    let (node, .., info) = decode_path(&package_path, options).await?;

    Ok((node, info))
}

/// Decode a PMC OA Package or HTML file to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Check if this is an HTML file
    if let Some(extension) = path.extension() {
        if extension == "html" {
            return decode_html_path(path, options).await;
        }
    }

    // Handle tar.gz PMC OA Package
    decode_tar_path(path, options).await
}

/// Decode a PMC HTML file to a Stencila [`Node`]
#[tracing::instrument]
async fn decode_html_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Read the HTML content
    let html_content = std::fs::read_to_string(path)?;

    // Parse the HTML using the HTML decoder module
    let (node, info) = decode_html::decode_html(&html_content, options).await?;

    Ok((node, None, info))
}

/// Decode a PMC OA Package (tar.gz) to a Stencila [`Node`]
#[tracing::instrument]
async fn decode_tar_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Create temporary directory to extract into
    // if path is not already a directory (e.g. an unzipped PMC OA Package)
    let tempdir = tempdir()?;
    let dir = if path.is_dir() { path } else { tempdir.path() };

    if path.is_file() {
        tracing::debug!("Extracting PMC OA package");
        let file = std::fs::File::open(path)?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(dir)?;
    }

    // Find the PMCXXXX directory within the dir
    let dir = glob(&dir.join("PMC*").to_string_lossy())?
        .flatten()
        .next()
        .ok_or_eyre("Unable to find PMC subdirectory in archive")?;

    // Find the JATS file in the dir
    let jats_path = glob(&dir.join("*.nxml").to_string_lossy())?
        .next()
        .and_then(|res| res.ok())
        .ok_or_eyre("Unable to find JATS XML file in PMC OA PAckage")?;

    // Decode the JATS
    let (mut node, .., info) = JatsCodec.from_path(&jats_path, options).await?;

    // Embed media and supplements
    embed_media(&mut node, Some(&dir))?;
    embed_supplements(&mut node, &dir).await?;

    Ok((node, None, info))
}

/// Download the PMC OA Package for a PMCID
///
/// Returns the path to the downloaded package.
pub(super) async fn download_package(pmcid: &str, to_path: &Path) -> Result<()> {
    tracing::debug!("Getting URL for OA package for `{pmcid}`");
    let url = format!("https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi?id={pmcid}");
    let xml = Client::new()
        .get(&url)
        .header(USER_AGENT, STENCILA_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // First, check if the XML contains an error element and extract the message
    let error_regex = Regex::new(r#"<error[^>]*>([^<]+)</error>"#).expect("invalid error regex");
    if let Some(captures) = error_regex.captures(&xml) {
        let error_message = captures
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or("unknown error");
        bail!("PubMed Central responded with error: {error_message}");
    }

    // If no error, proceed to extract the download URL
    let link_regex = Regex::new(r#"href="([^"]+\.tar\.gz)""#).expect("invalid regex");
    let ftp_url = link_regex
        .captures(&xml)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .wrap_err_with(|| eyre!("No .tar.gz link found for {pmcid}"))?;

    let https_url = ftp_url.replacen("ftp://", "https://", 1);

    tracing::debug!("Downloading {https_url}");
    let response = Client::new()
        .get(&https_url)
        .send()
        .await?
        .error_for_status()?;

    let mut file = File::create(&to_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    Ok(())
}
