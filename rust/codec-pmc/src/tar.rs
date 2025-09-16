use std::path::Path;

use flate2::read::GzDecoder;
use futures::StreamExt;
use glob::glob;
use regex::Regex;
use reqwest::{Client, header::USER_AGENT};
use stencila_node_supplements::embed_supplements;
use tar::Archive;
use tempfile::tempdir;
use tokio::{fs::File, io::AsyncWriteExt};

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{ContextCompat, OptionExt, Result, bail, eyre},
    stencila_schema::Node,
};
use stencila_codec_jats::JatsCodec;
use stencila_node_media::embed_media;
use stencila_version::STENCILA_USER_AGENT;

/// Download the PMC OA Package for a PMCID
///
/// Returns the path to the downloaded package.
pub(super) async fn download_tar(pmcid: &str, to_path: &Path) -> Result<()> {
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

/// Decode a PMC OA Package (tar.gz) to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_tar(
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
    let (mut node, .., info) = JatsCodec.from_path(&jats_path, options.clone()).await?;

    // Embed media by default
    if options
        .as_ref()
        .and_then(|opts| opts.embed_media)
        .unwrap_or(true)
    {
        embed_media(&mut node, Some(&dir))?;
    }

    // Embed supplements by default
    if options
        .as_ref()
        .and_then(|opts| opts.embed_supplements)
        .unwrap_or(true)
    {
        embed_supplements(&mut node, &dir).await?;
    }

    Ok((node, None, info))
}
