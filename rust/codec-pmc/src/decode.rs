use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use stencila_dirs::closest_artifacts_for;
use stencila_node_media::{relocate_media, side_dir};
use tempfile::tempdir;
use url::Url;

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::{Result, WrapErr, bail},
    stencila_format::Format,
    stencila_schema::Node,
};

use crate::bucket::{self, BUCKET_URL, Bucket, decode_bucket, decode_bucket_entry};
use crate::html::download_html;

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
) -> Result<(Node, DecodeInfo, Format)> {
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

    // Keep the temporary directory alive through decoding and media retention.
    let temp_dir = tempdir()?;
    let artifacts_dir = if no_artifacts {
        temp_dir.path().to_path_buf()
    } else {
        let artifacts_key = format!("pmc-{}", pmcid.trim_start_matches("PMC"));
        closest_artifacts_for(&current_dir()?, &artifacts_key).await?
    };
    let bucket = Bucket::new(BUCKET_URL)?;
    let (mut node, info, format) = decode_pmcid_in(
        pmcid,
        &artifacts_dir,
        ignore_artifacts,
        options,
        &bucket,
        "https://pmc.ncbi.nlm.nih.gov/articles/",
    )
    .await?;

    // Set metadata
    if let Node::Article(article) = &mut node {
        article.options.repository = Some("https://pmc.ncbi.nlm.nih.gov".into());
        article.options.path = Some(format!("articles/{pmcid}/"))
    }

    Ok((node, info, format))
}

/// A cached or freshly fetched article
enum Entry {
    /// An article from the PMC AWS Open Data bucket
    Bucket(PathBuf),
    /// A legacy PMC OA Package
    Tar(PathBuf),
    /// A scraped PMC article page
    Html(PathBuf),
}

/// Select a cache entry or fetch an article. Keeping the cache directory and
/// endpoints explicit also allows testing the entire routing policy offline.
///
/// A cached page is only used if the bucket has no article or is unavailable,
/// so that pages scraped while bucket articles were unavailable get upgraded.
/// After a fetch, other entries are removed so they can not take precedence.
async fn select_entry(
    pmcid: &str,
    directory: &Path,
    ignore_artifacts: bool,
    bucket: &Bucket,
    html_base: &str,
) -> Result<Entry> {
    let manifest_path = directory.join(format!("{pmcid}.json"));
    let tar_path = directory.join(format!("{pmcid}.tar.gz"));
    let html_path = directory.join(format!("{pmcid}.html"));
    if !ignore_artifacts && manifest_path.exists() {
        return Ok(Entry::Bucket(manifest_path));
    }
    if !ignore_artifacts && tar_path.exists() {
        return Ok(Entry::Tar(tar_path));
    }
    let cached_html = !ignore_artifacts && html_path.exists();
    match bucket.download(pmcid, directory).await {
        Ok(Some(path)) => {
            remove_file_if_exists(&tar_path).await?;
            remove_file_if_exists(&html_path).await?;
            Ok(Entry::Bucket(path))
        }
        Ok(None) if cached_html => Ok(Entry::Html(html_path)),
        Ok(None) => {
            tracing::warn!(
                "No article versions found for {pmcid} in the PMC AWS Open Data bucket (empty listing); falling back to HTML"
            );
            let url = Url::parse(html_base)?.join(&format!("{pmcid}/"))?;
            download_html(&bucket.client, url, &html_path).await?;
            bucket::remove(&manifest_path).await?;
            remove_file_if_exists(&tar_path).await?;
            Ok(Entry::Html(html_path))
        }
        Err(error) if cached_html => {
            tracing::warn!(
                "Unable to fetch {pmcid} from the PMC AWS Open Data bucket; using cached HTML: {error:#}"
            );
            Ok(Entry::Html(html_path))
        }
        Err(error) => Err(error),
    }
}

async fn remove_file_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        tokio::fs::remove_file(path).await?;
    }
    Ok(())
}

/// Decode an article, from the cache in `directory` if possible
pub(super) async fn decode_pmcid_in(
    pmcid: &str,
    directory: &Path,
    ignore_artifacts: bool,
    options: Option<DecodeOptions>,
    bucket: &Bucket,
    html_base: &str,
) -> Result<(Node, DecodeInfo, Format)> {
    match select_entry(pmcid, directory, ignore_artifacts, bucket, html_base).await? {
        Entry::Bucket(path) => {
            let (mut node, info, jats_path) = decode_bucket_entry(&path, options.clone()).await?;
            // An identifier has no source path for callers to resolve against.
            // Rebase references to the working directory, retaining originals if
            // no_artifacts means the source directory is about to be deleted.
            let options = options.unwrap_or_default();
            let reference_media = options.embed_media == Some(false);
            let reference_supplements = options.embed_supplements == Some(false);
            if reference_media || reference_supplements {
                let cwd = current_dir()?;
                let retained_path = cwd.join(format!("{pmcid}.xml"));
                let retain = options.no_artifacts.unwrap_or_default();
                let media_dir =
                    (retain && reference_media).then(|| side_dir(&retained_path, ".media"));
                let supplements_dir = (retain && reference_supplements)
                    .then(|| side_dir(&retained_path, ".supplements"));
                relocate_media(
                    &mut node,
                    Some(&jats_path),
                    None,
                    media_dir.as_deref(),
                    supplements_dir.as_deref(),
                )
                .wrap_err_with(|| {
                    format!(
                        "Could not rebase or retain original media and supplements in `{}`; is it writable? \
                         Decode with embedding enabled (e.g. `--embed-media --embed-supplements`) or \
                         from a writable working directory",
                        cwd.display()
                    )
                })?;
            }
            Ok((node, info, Format::PmcOa))
        }
        Entry::Tar(path) => {
            let (node, _, info) = decode_tar(&path, options).await?;
            Ok((node, info, Format::PmcOa))
        }
        Entry::Html(path) => {
            let (node, _, info) = decode_html(&path, options)?;
            Ok((node, info, Format::Html))
        }
    }
}

/// Decode a completed bucket cache, legacy PMC OA Package, or HTML file.
#[tracing::instrument]
pub(super) async fn decode_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => decode_bucket(path, options).await,
        Some("html") => decode_html(path, options),
        Some("gz") => decode_tar(path, options).await,
        _ => bail!("Unhandled file extension"),
    }
}
