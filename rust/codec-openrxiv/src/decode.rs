use std::{env::current_dir, sync::LazyLock};

use url::Url;

use eyre::{Result, bail};
use futures::StreamExt;
use regex::Regex;
use reqwest::{Client, Response};
use tempfile::tempdir;
use tokio::{fs::File, io::AsyncWriteExt};

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, stencila_format::Format, stencila_schema::Node,
};
use stencila_codec_meca::MecaCodec;
use stencila_codec_pdf::PdfCodec;
use stencila_dirs::closest_artifacts_for;
use stencila_version::STENCILA_USER_AGENT;

const BIORXIV: &str = "biorxiv.org";
const MEDRXIV: &str = "medrxiv.org";
const DOI_PREFIX: &str = "10.1101";

/// Extract an openRxiv id from an identifier
///
/// Extracts the id (e.g 2025.07.15.664907v1) from:
///
/// - a bioRxiv or medRxiv URL e.g. https://www.biorxiv.org/content/10.1101/2025.07.15.664907v1
///   (including suffixes such as `.full.pdf`)
///
/// - an openRxiv DOI URL e.g. https://doi.org/10.1101/2025.07.15.664907
///
/// - an openRxiv DOI e.g. 10.1101/2025.07.15.664907
///
/// Note that early openRxiv ids did not have a date at the start of the suffix
/// e.g. 809020
///
/// Returns the id, and, if possible, the server (biorxiv.org or medrxiv.org).
pub(super) fn extract_openrxiv_id(identifier: &str) -> Option<(String, Option<String>)> {
    let identifier = identifier.trim().to_lowercase();

    static OPENRXIV_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^/content/10\.1101/(.*?)(?:\.full\.pdf)?$").expect("invalid regex")
    });

    static DOI_URL_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^/10\.1101/(.*)$").expect("invalid regex"));

    static DOI_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^10\.1101/(.*)$").expect("invalid regex"));

    // Try to parse as URL first
    if let Ok(url) = Url::parse(&identifier) {
        match url.host_str() {
            Some(BIORXIV) | Some("www.biorxiv.org") | Some(MEDRXIV) | Some("www.medrxiv.org") => {
                let path = url.path();
                if let Some(captures) = OPENRXIV_URL_REGEX.captures(path) {
                    return Some((
                        captures.get(1)?.as_str().to_string(),
                        url.host_str()
                            .map(|host| host.trim_start_matches("www.").to_string()),
                    ));
                }
            }
            Some("doi.org") | Some("dx.doi.org") => {
                let path = url.path();
                if let Some(captures) = DOI_URL_REGEX.captures(path) {
                    return Some((captures.get(1)?.as_str().to_string(), None));
                }
            }
            _ => {}
        }
    }

    // Try to match DOI
    if let Some(captures) = DOI_REGEX.captures(&identifier) {
        return Some((captures.get(1)?.as_str().to_string(), None));
    }

    None
}

/// Convert an openRxiv id to a DOI
///
/// Strips any version suffix and adds the openRxiv DOI prefix.
pub fn openrxiv_id_to_doi(openrxiv_id: &str) -> String {
    // Strip version suffix (e.g., v1, v2) from the ID
    let id_without_version = if let Some(pos) = openrxiv_id.find('v') {
        &openrxiv_id[..pos]
    } else {
        openrxiv_id
    };

    [DOI_PREFIX, "/", id_without_version].concat()
}

/// Decode an openRxiv id to a Stencila [`Node`]
///
/// Tries to fetch content from openRxiv in the following order:
///
/// 1. MECA
/// 2. PDF
///
/// Returns the result from the first successful format. MECA is preferred
/// because it includes JATS XML and images for the preprint.
///
/// openRxiv MECA archives are only available via a "requester pays" AWS S3 bucket
/// and there is not published mapping between openRxiv ids (or DOIs) and MECAs.
/// Therefore, this function uses Stencila Clouds service for fetching openRxiv
/// MECA, which falls back to a PDF response if a MECA is not available yet.
#[tracing::instrument(skip(options))]
pub(super) async fn decode_openrxiv_id(
    openrxiv_id: &str,
    server: Option<&str>,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    tracing::debug!("Attempting to decode openRxiv preprint `{openrxiv_id}`");

    let (first, second) = if let Some(MEDRXIV) = server {
        (MEDRXIV, BIORXIV)
    } else {
        (BIORXIV, MEDRXIV)
    };

    for (format, url) in [
        (
            "meca",
            format!("https://api.stencila.cloud/v1/remotes/openrxiv/{openrxiv_id}"),
        ),
        (
            "pdf",
            format!("https://www.{first}/content/{DOI_PREFIX}/{openrxiv_id}.full.pdf"),
        ),
        (
            "pdf",
            format!("https://www.{second}/content/{DOI_PREFIX}/{openrxiv_id}.full.pdf"),
        ),
    ] {
        tracing::debug!("Trying `{format}` format: {url}");

        let client = if url.contains("stencila.cloud") {
            stencila_cloud::client().await?
        } else {
            Client::builder().user_agent(STENCILA_USER_AGENT).build()?
        };

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                tracing::debug!("Successfully fetched `{format}` for `{openrxiv_id}`",);
                match decode_preprint(openrxiv_id, server, response, options.clone()).await {
                    Ok(result) => return Ok(result),
                    Err(error) => {
                        tracing::warn!("Failed to decode `{format}`: {error}");
                    }
                }
            }
            Ok(response) => {
                tracing::debug!("`{format}` not available: HTTP {}", response.status());
            }
            Err(error) => {
                tracing::debug!("Failed to fetch `{format}`: {error}");
            }
        }
    }

    bail!(
        "Failed to decode openRxiv `{openrxiv_id}`, no format was available or successfully decoded",
    )
}

/// Decode a preprint to a Stencila [`Node`]
#[tracing::instrument(skip(options, response))]
pub(super) async fn decode_preprint(
    openrxiv_id: &str,
    server: Option<&str>,
    response: Response,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let ignore_artifacts = options
        .as_ref()
        .and_then(|opts| opts.ignore_artifacts)
        .unwrap_or_default();
    let no_artifacts = options
        .as_ref()
        .and_then(|opts| opts.no_artifacts)
        .unwrap_or_default();

    let format = response
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or_default();

    let format = if format.contains("application/pdf") {
        Format::Pdf
    } else {
        Format::Meca
    };

    let filename = format!("{openrxiv_id}.{}", format.extension());

    // Create temporary directory (must be kept alive for entire function)
    let temp_dir = tempdir()?;

    // Determine where to store/look for the downloaded file
    let file_path = if no_artifacts {
        // Don't cache, use temporary directory
        temp_dir.path().join(&filename)
    } else {
        // Use artifacts directory for caching
        let artifacts_key = format!("openrxiv-{openrxiv_id}-{format}");
        let artifacts_dir = closest_artifacts_for(&current_dir()?, &artifacts_key).await?;
        artifacts_dir.join(&filename)
    };

    // Download the file if needed
    let should_download = !file_path.exists() || ignore_artifacts;
    if should_download {
        let mut file = File::create(&file_path).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        drop(file);
    }

    let (mut node, .., info) = match format {
        Format::Meca => MecaCodec.from_path(&file_path, options).await?,
        Format::Pdf => PdfCodec.from_path(&file_path, options).await?,
        _ => bail!("Unhandled format `{format}`"),
    };

    // Set DOI, and other metadata
    if let Node::Article(article) = &mut node {
        let doi = openrxiv_id_to_doi(openrxiv_id);
        article.doi = Some(doi.clone());
        if let Some(server) = server {
            article.options.repository = Some(format!("https://{server}"));
            article.options.path = Some(doi);
        }
    }

    Ok((node, info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_openrxiv_id() {
        // Test bioRxiv URLs
        assert_eq!(
            extract_openrxiv_id("https://www.biorxiv.org/content/10.1101/2025.07.15.664907v1"),
            Some(("2025.07.15.664907v1".into(), Some(BIORXIV.into())))
        );

        assert_eq!(
            extract_openrxiv_id(
                "https://www.biorxiv.org/content/10.1101/2025.07.15.664907v1.full.pdf"
            ),
            Some(("2025.07.15.664907v1".into(), Some(BIORXIV.into())))
        );

        // Test medRxiv URLs
        assert_eq!(
            extract_openrxiv_id("https://www.medrxiv.org/content/10.1101/2024.12.01.24318123v2"),
            Some(("2024.12.01.24318123v2".into(), Some(MEDRXIV.into())))
        );

        // Test early format (numeric only)
        assert_eq!(
            extract_openrxiv_id("https://www.biorxiv.org/content/10.1101/809020"),
            Some(("809020".into(), Some(BIORXIV.into())))
        );

        // Test DOI URLs
        assert_eq!(
            extract_openrxiv_id("https://doi.org/10.1101/2025.07.15.664907"),
            Some(("2025.07.15.664907".into(), None))
        );

        assert_eq!(
            extract_openrxiv_id("https://doi.org/10.1101/809020v1"),
            Some(("809020v1".into(), None))
        );

        // Test plain DOIs
        assert_eq!(
            extract_openrxiv_id("10.1101/2025.07.15.664907"),
            Some(("2025.07.15.664907".into(), None))
        );

        assert_eq!(
            extract_openrxiv_id("10.1101/809020"),
            Some(("809020".into(), None))
        );

        // Test invalid inputs
        assert_eq!(
            extract_openrxiv_id("https://example.com/content/10.1101/invalid"),
            None
        );

        assert_eq!(extract_openrxiv_id("10.1038/nature12345"), None);

        assert_eq!(extract_openrxiv_id("not a valid identifier"), None);

        // Test case insensitivity
        assert_eq!(
            extract_openrxiv_id("HTTPS://WWW.BIORXIV.ORG/CONTENT/10.1101/2025.07.15.664907V1"),
            Some(("2025.07.15.664907v1".into(), Some(BIORXIV.into())))
        );
    }

    #[test]
    fn test_openrxiv_id_to_doi() {
        // Test with version suffix
        assert_eq!(
            openrxiv_id_to_doi("2025.07.15.664907v1"),
            "10.1101/2025.07.15.664907"
        );

        // Test without version suffix
        assert_eq!(
            openrxiv_id_to_doi("2025.07.15.664907"),
            "10.1101/2025.07.15.664907"
        );

        // Test early format
        assert_eq!(openrxiv_id_to_doi("809020"), "10.1101/809020");

        assert_eq!(openrxiv_id_to_doi("809020v2"), "10.1101/809020");
    }
}
