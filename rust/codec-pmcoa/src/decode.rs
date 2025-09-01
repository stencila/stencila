use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use url::Url;

use codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{ContextCompat, OptionExt, Result, bail},
    schema::Node,
};
use futures::StreamExt;
use glob::glob;
use regex::Regex;
use reqwest::{Client, header::USER_AGENT};
use tar::Archive;
use tokio::{
    fs::{File, remove_file},
    io::AsyncWriteExt,
};
use codec_jats::JatsCodec;
use media_embed::embed_media;
use version::STENCILA_USER_AGENT;

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
    if !pmcid.starts_with("PMC") {
        bail!("Unrecognized article id, should be a PMC id, starting with `PMC`")
    }

    // Download package
    let pmcoa = download_package(pmcid).await?;

    // Decode package
    let (node, .., info) = decode_path(&pmcoa, options).await?;

    // Remove downloaded package
    remove_file(pmcoa).await?;

    Ok((node, info))
}

/// Decode a PMC OA Package to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Create temporary directory to extract into
    // if path is not already a directory (e.g. an unzipped PMC OA Package)
    let tempdir = tempfile::TempDir::new()?;
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

    // Embed any images
    embed_media(&mut node, &dir)?;

    Ok((node, None, info))
}

/// Download the PMC OA Package for a PMCID
///
/// Returns the path to the downloaded package.
pub(super) async fn download_package(pmcid: &str) -> Result<PathBuf> {
    let pmcid = pmcid.trim();

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

    let re = Regex::new(r#"href="([^"]+\.tar\.gz)""#).expect("invalid regex");
    let ftp_url = re
        .captures(&xml)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .wrap_err("No .tar.gz link found")?;

    let https_url = ftp_url.replacen("ftp://", "https://", 1);

    tracing::debug!("Downloading {https_url}");
    let response = Client::new()
        .get(&https_url)
        .send()
        .await?
        .error_for_status()?;

    let path = PathBuf::from(format!("{pmcid}.tar.gz"));
    let mut file = File::create(&path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    Ok(path)
}
