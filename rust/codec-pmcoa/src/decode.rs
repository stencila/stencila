use std::path::{Path, PathBuf};

use codec::{
    common::{
        eyre::{bail, ContextCompat, OptionExt, Result},
        futures::StreamExt,
        glob::glob,
        regex::Regex,
        reqwest::Client,
        tar::Archive,
        tempfile,
        tokio::{
            fs::{remove_file, File},
            io::AsyncWriteExt,
        },
        tracing,
    },
    schema::Node,
    Codec, DecodeInfo, DecodeOptions,
};
use codec_jats::JatsCodec;
use flate2::read::GzDecoder;
use media_embed::embed_media;

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
