use std::{env::current_dir, path::Path, sync::LazyLock};

use futures::StreamExt;
use regex::Regex;
use reqwest::{Client, header::USER_AGENT};
use tempfile::tempdir;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_dirs::closest_artifacts_for;
use stencila_version::STENCILA_USER_AGENT;

use super::decode_html::decode_arxiv_html;
use super::decode_pdf::decode_arxiv_pdf;
use super::decode_src::decode_arxiv_src;

/// Extract an arXiv id from an identifier
///
/// Extracts the id (e.g 2507.11254) from:
///
/// - a bare arXiv id e.g. arXiv:2507.11254
///
/// - an arXiv URL e.g. https://arxiv.org/abs/2507.11254 (abs, pdf, src, html,
///   format & export.arxiv.org subdomain)
///
/// - an arXiv DOI e.g. 10.48550/arXiv.2507.11254
///
/// - an arXiv DOI URL e.g. https://doi.org/10.48550/arXiv.2507.11254
pub(super) fn extract_arxiv_id(identifier: &str) -> Option<String> {
    let identifier = identifier.trim().to_lowercase();

    static ARXIV_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^/(?:(?:abs|pdf|src|html|format)/)?([0-9]{4}\.[0-9]{4,5}(?:v[0-9]+)?)$")
            .expect("invalid regex")
    });

    static DOI_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^/10\.48550/arxiv\.([0-9]{4}\.[0-9]{4,5}(?:v[0-9]+)?)$")
            .expect("invalid regex")
    });

    static ARXIV_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^arxiv:([0-9]{4}\.[0-9]{4,5}(?:v[0-9]+)?)$").expect("invalid regex")
    });

    static DOI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^10\.48550/arxiv\.([0-9]{4}\.[0-9]{4,5}(?:v[0-9]+)?)$").expect("invalid regex")
    });

    static BARE_ID_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[0-9]{4}\.[0-9]{4,5}(?:v[0-9]+)?$").expect("invalid regex"));

    // Try to parse as URL first
    if let Ok(url) = Url::parse(&identifier) {
        match url.host_str() {
            Some("arxiv.org") | Some("export.arxiv.org") => {
                let path = url.path();
                if let Some(captures) = ARXIV_URL_REGEX.captures(path) {
                    return Some(captures.get(1)?.as_str().to_string());
                }
            }
            Some("doi.org") => {
                let path = url.path();
                if let Some(captures) = DOI_URL_REGEX.captures(path) {
                    return Some(captures.get(1)?.as_str().to_string());
                }
            }
            _ => {}
        }
    }

    // Try to match bare arXiv id (e.g., arXiv:2507.11254)
    if let Some(captures) = ARXIV_ID_REGEX.captures(&identifier) {
        return Some(captures.get(1)?.as_str().to_string());
    }

    // Try to match DOI (e.g., 10.48550/arxiv.2507.11254)
    if let Some(captures) = DOI_REGEX.captures(&identifier) {
        return Some(captures.get(1)?.as_str().to_string());
    }

    // Try to match just the ID (e.g., 2507.11254)
    if BARE_ID_REGEX.is_match(&identifier) {
        return Some(identifier.to_string());
    }

    None
}

/// Convert an arXiv id to a DOI
///
/// Strips any version suffix and adds the arXiv DOI prefix.
pub fn arxiv_id_to_doi(arxiv_id: &str) -> String {
    // Strip version suffix (e.g., v1, v2) from the ID
    let id_without_version = if let Some(pos) = arxiv_id.find('v') {
        &arxiv_id[..pos]
    } else {
        arxiv_id
    };

    ["10.48550/arxiv.", id_without_version].concat()
}

/// Download a file for an arXiv ID from export.arxiv.org
async fn download_arxiv_file(arxiv_id: &str, format: &Format, to_path: &Path) -> Result<()> {
    let url = match format {
        Format::Html => format!("https://export.arxiv.org/html/{arxiv_id}"),
        Format::Latex => format!("https://export.arxiv.org/src/{arxiv_id}"),
        Format::Pdf => format!("https://export.arxiv.org/pdf/{arxiv_id}.pdf"),
        _ => bail!("Unsupported format: {format}"),
    };

    tracing::debug!("Downloading {format} for {arxiv_id} from {url}");

    let response = Client::new()
        .get(&url)
        .header(USER_AGENT, STENCILA_USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        bail!(
            "Failed to download {format} for {arxiv_id}: HTTP {}",
            response.status()
        );
    }

    if let Some(parent) = to_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(to_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }
    file.sync_all().await?;

    tracing::debug!(
        "Successfully downloaded {format} for {arxiv_id} to {}",
        to_path.display()
    );
    Ok(())
}

/// Decode an arXiv file from a path to a Stencila [`Node`]
async fn decode_arxiv_path(
    arxiv_id: &str,
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => decode_arxiv_html(arxiv_id, path, options).await,
        Some("gz") => decode_arxiv_src(arxiv_id, path, options).await,
        Some("pdf") => decode_arxiv_pdf(arxiv_id, path, options).await,
        _ => bail!("Unhandled file extension for {}", path.display()),
    }
}

/// Decode an arXiv id to a Stencila [`Node`]
///
/// Tries to fetch content from arXiv in the following order:
/// 1. HTML format (processed/rendered version)
/// 2. Source format (LaTeX/TeX files)
/// 3. PDF format (as fallback)
///
/// Returns the result from the first successful format.
///
/// HTML is preferred because (a) it includes images (it would seem to be
/// necessary to do a bulk download from an S3 bucket to obtain these otherwise)
/// and (b) it is generated using https://math.nist.gov/~BMiller/LaTeXML/ which
/// has advanced handling of LaTeX packages and produces "semantically tagged
/// HTML", (c) does not have a dependency on Pandoc.
///
/// The function respects caching options:
/// - `no_artifacts`: disables caching entirely, uses temporary directory
/// - `ignore_artifacts`: ignores existing cached files, forces re-download
/// - Otherwise: caches files in artifacts directory for reuse
#[tracing::instrument(skip(options))]
pub(super) async fn decode_arxiv_id(
    arxiv_id: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo, Format)> {
    let arxiv_id = arxiv_id.trim();
    tracing::debug!("Attempting to decode arXiv preprint `{arxiv_id}`");

    let ignore_artifacts = options
        .as_ref()
        .and_then(|opts| opts.ignore_artifacts)
        .unwrap_or_default();
    let no_artifacts = options
        .as_ref()
        .and_then(|opts| opts.no_artifacts)
        .unwrap_or_default();

    let html_filename = format!("{arxiv_id}.html");
    let src_filename = format!("{arxiv_id}.tar.gz");
    let pdf_filename = format!("{arxiv_id}.pdf");

    // Create temporary directory (must be kept alive for entire function)
    let temp_dir = tempdir()?;

    // Determine where to store/look for the downloaded files
    let (html_path, src_path, pdf_path) = if no_artifacts {
        // Don't cache, use temporary directory
        (
            temp_dir.path().join(&html_filename),
            temp_dir.path().join(&src_filename),
            temp_dir.path().join(&pdf_filename),
        )
    } else {
        let artifacts_key = format!("arxiv-{arxiv_id}");
        let artifacts_dir = closest_artifacts_for(&current_dir()?, &artifacts_key).await?;
        (
            artifacts_dir.join(&html_filename),
            artifacts_dir.join(&src_filename),
            artifacts_dir.join(&pdf_filename),
        )
    };

    // Try each format in order of preference
    for (format, path) in [
        (Format::Html, &html_path),
        (Format::Latex, &src_path),
        (Format::Pdf, &pdf_path),
    ] {
        let should_download = !path.exists() || ignore_artifacts;

        if should_download {
            tracing::debug!("Downloading `{format}` format for {arxiv_id}");
            let download_result = download_arxiv_file(arxiv_id, &format, path).await;

            match download_result {
                Ok(()) => {
                    tracing::debug!("Successfully downloaded `{format}` for {arxiv_id}");
                }
                Err(download_error) => {
                    tracing::debug!(
                        "Failed to download `{format}` for {arxiv_id}: {download_error}"
                    );
                    continue;
                }
            }
        } else {
            tracing::debug!("Using cached `{format}` file for {arxiv_id}");
        }

        // Try to decode the file
        match decode_arxiv_path(arxiv_id, path, options.clone()).await {
            Ok((mut node, info)) => {
                // Set metadata
                if let Node::Article(article) = &mut node {
                    article.options.repository = Some("https://arxiv.org".into());
                    article.options.path = Some(format!("abs/{arxiv_id}"));
                }

                tracing::debug!("Successfully decoded `{format}` for {arxiv_id}");
                return Ok((node, info, format));
            }
            Err(decode_error) => {
                tracing::warn!("Failed to decode `{format}` for {arxiv_id}: {decode_error}");
                continue;
            }
        }
    }

    bail!("Failed to decode arXiv `{arxiv_id}`, no format was available or successfully decoded")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_arxiv_id_from_urls() {
        // Test arXiv URLs
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/abs/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/pdf/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/src/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/format/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/html/2507.11254"),
            Some("2507.11254".to_string())
        );

        // Test export.arxiv.org subdomain
        assert_eq!(
            extract_arxiv_id("https://export.arxiv.org/abs/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://export.arxiv.org/pdf/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://export.arxiv.org/src/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://export.arxiv.org/html/2507.11254"),
            Some("2507.11254".to_string())
        );

        // Test with version
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/abs/2507.11254v1"),
            Some("2507.11254v1".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/pdf/2507.11254v2"),
            Some("2507.11254v2".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://export.arxiv.org/src/2507.11254v3"),
            Some("2507.11254v3".to_string())
        );

        // Test URLs without path prefix. These are not (currently) valid but
        // respect the intent of the user
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://export.arxiv.org/2507.11254v1"),
            Some("2507.11254v1".to_string())
        );
    }

    #[test]
    fn test_extract_arxiv_id_from_doi_urls() {
        // Test DOI URLs
        assert_eq!(
            extract_arxiv_id("https://doi.org/10.48550/arxiv.2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("https://doi.org/10.48550/arXiv.2507.11254v1"),
            Some("2507.11254v1".to_string())
        );
    }

    #[test]
    fn test_extract_arxiv_id_from_bare_ids() {
        // Test bare arXiv IDs
        assert_eq!(
            extract_arxiv_id("arXiv:2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("arxiv:2507.11254v1"),
            Some("2507.11254v1".to_string())
        );
        assert_eq!(
            extract_arxiv_id("arXiv:2507.11254v10"),
            Some("2507.11254v10".to_string())
        );
    }

    #[test]
    fn test_extract_arxiv_id_from_dois() {
        // Test DOI strings
        assert_eq!(
            extract_arxiv_id("10.48550/arxiv.2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("10.48550/arXiv.2507.11254v1"),
            Some("2507.11254v1".to_string())
        );
    }

    #[test]
    fn test_extract_arxiv_id_from_just_ids() {
        // Test just the ID
        assert_eq!(
            extract_arxiv_id("2507.11254"),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("2507.11254v1"),
            Some("2507.11254v1".to_string())
        );
        assert_eq!(extract_arxiv_id("1234.5678"), Some("1234.5678".to_string()));
        assert_eq!(
            extract_arxiv_id("1234.56789"),
            Some("1234.56789".to_string())
        );
    }

    #[test]
    fn test_extract_arxiv_id_with_whitespace() {
        // Test with whitespace
        assert_eq!(
            extract_arxiv_id("  arXiv:2507.11254  "),
            Some("2507.11254".to_string())
        );
        assert_eq!(
            extract_arxiv_id("  https://arxiv.org/abs/2507.11254  "),
            Some("2507.11254".to_string())
        );
    }

    #[test]
    fn test_extract_arxiv_id_invalid() {
        // Test invalid formats
        assert_eq!(extract_arxiv_id(""), None);
        assert_eq!(extract_arxiv_id("not-an-arxiv-id"), None);
        assert_eq!(extract_arxiv_id("https://example.com/2507.11254"), None);
        assert_eq!(extract_arxiv_id("2507.112"), None); // too short
        assert_eq!(extract_arxiv_id("2507.112540"), None); // too long
        assert_eq!(extract_arxiv_id("25072.11254"), None); // wrong year format
        assert_eq!(extract_arxiv_id("2507.11254v"), None); // incomplete version
        assert_eq!(extract_arxiv_id("https://arxiv.org/abs/"), None); // no ID
        assert_eq!(extract_arxiv_id("https://doi.org/10.48550/"), None); // incomplete DOI
        assert_eq!(extract_arxiv_id("10.48550/arXiv."), None); // incomplete DOI

        assert_eq!(extract_arxiv_id("2507.112"), None); // 3 digits after dot - too short
        assert_eq!(extract_arxiv_id("2507.112540"), None); // 6 digits after dot - too long
        assert_eq!(extract_arxiv_id("507.11254"), None); // 4 digits before dot required
        assert_eq!(extract_arxiv_id("25072.11254"), None); // 4 digits before dot maximum
    }
}
