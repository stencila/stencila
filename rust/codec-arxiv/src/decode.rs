use std::sync::LazyLock;

use regex::Regex;
use url::Url;

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::{Result, bail, eyre},
    stencila_format::Format,
    stencila_schema::Node,
};

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
#[tracing::instrument(skip(options))]
pub(super) async fn decode_arxiv_id(
    arxiv_id: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo, Format)> {
    tracing::debug!("Attempting to decode arXiv preprint `{arxiv_id}`");

    for (format, url) in [
        (
            Format::Html,
            format!("https://export.arxiv.org/html/{arxiv_id}"),
        ),
        (
            Format::Latex,
            format!("https://export.arxiv.org/src/{arxiv_id}"),
        ),
        (
            Format::Pdf,
            format!("https://export.arxiv.org/pdf/{arxiv_id}.pdf"),
        ),
    ] {
        tracing::debug!("Trying `{format}` format: {url}");

        if let Some((node, decode_info)) =
            decode_arxiv_url(arxiv_id, &format, &url, options.clone()).await
        {
            return Ok((node, decode_info, format));
        }
    }

    bail!("Failed to decode arXiv `{arxiv_id}`, no format was available or successfully decoded",)
}

#[tracing::instrument(skip(options))]
pub(super) async fn decode_arxiv_url(
    arxiv_id: &str,
    format: &Format,
    url: &str,
    options: Option<DecodeOptions>,
) -> Option<(Node, DecodeInfo)> {
    tracing::debug!("Trying `{format}` format: {url}");

    match reqwest::get(url).await {
        Ok(response) if response.status().is_success() => {
            tracing::debug!("Successfully fetched `{format}` for `{arxiv_id}`",);

            let result = match format {
                Format::Html => match response.text().await {
                    Ok(html) => decode_arxiv_html(arxiv_id, &html, options).await,
                    Err(error) => Err(eyre!("Failed to fetch `{format}`: {error}")),
                },
                Format::Latex => decode_arxiv_src(arxiv_id, response, options).await,
                Format::Pdf => decode_arxiv_pdf(arxiv_id, response, options).await,
                _ => unreachable!(),
            };

            match result {
                Ok((node, info)) => return Some((node, info)),
                Err(error) => {
                    tracing::warn!("Failed to decode `{format}`: {error}");
                }
            }
        }
        Ok(response) => {
            tracing::trace!("`{format}` not available: HTTP {}", response.status());
        }
        Err(error) => {
            tracing::debug!("Failed to fetch `{format}`: {error}");
        }
    };

    None
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
