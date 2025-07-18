use crate::client::DoiClient;
use codec::{
    DecodeInfo, DecodeOptions,
    common::{
        eyre::{Result, eyre},
        once_cell::sync::Lazy,
        regex::Regex,
        tracing,
    },
    schema::Node,
};
use url::Url;

/// Extract a DOI from an identifier string
///
/// Extracts the DOI from:
///
/// - a bare DOI e.g. 10.0001/abcd.123
///
/// - a DOI prefixed with label e.g. doi:10.0001/abcd.123 (should be flexible to label format)
///
/// - a DOI URL e.g. https://doi.org/10.0001/abcd.123 (or http:// or dx.doi.org subdomain)
pub(super) fn extract_doi(identifier: &str) -> Option<String> {
    let input = identifier.trim();

    static DOI_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)^10\.\d{4,9}/[-._;()/:A-Z0-9]+$").expect("invalid regex"));

    // Handle DOI URLs
    if let Ok(url) = input.parse::<Url>() {
        if let Some(host) = url.host_str() {
            if host == "doi.org" || host == "dx.doi.org" || host == "www.doi.org" {
                let path = url.path().trim_start_matches('/');
                if DOI_REGEX.is_match(path) {
                    return Some(path.to_string());
                }
            }
        }
    }

    // Handle prefixed DOIs (doi:, DOI:, etc.)
    if let Some(colon_pos) = input.find(':') {
        let prefix = &input[..colon_pos];
        if prefix.to_lowercase() == "doi" {
            let doi_part = input[colon_pos + 1..].trim();
            if DOI_REGEX.is_match(doi_part) {
                return Some(doi_part.to_string());
            }
        }
    }

    // Handle bare DOIs
    if DOI_REGEX.is_match(input) {
        return Some(input.to_string());
    }

    None
}

/// Decode a DOI to a Stencila [`Node`]
#[tracing::instrument(skip(_options))]
pub(super) async fn decode_doi(
    doi: &str,
    _options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    tracing::debug!("Decoding DOI: {}", doi);

    let client = DoiClient::new()?;
    
    let csl_item = client
        .get(doi)
        .await
        .map_err(|error| eyre!("Failed to fetch metadata for {doi}: {error}"))?;

    let article = csl_item.to_article()?;
    let decode_info = DecodeInfo::default();

    Ok((Node::Article(article), decode_info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doi() {
        // Test bare DOI
        assert_eq!(
            extract_doi("10.1234/example.123"),
            Some("10.1234/example.123".to_string())
        );

        // Test prefixed DOI
        assert_eq!(
            extract_doi("doi:10.1234/example.123"),
            Some("10.1234/example.123".to_string())
        );

        // Test DOI URL
        assert_eq!(
            extract_doi("https://doi.org/10.1234/example.123"),
            Some("10.1234/example.123".to_string())
        );

        // Test invalid DOI
        assert_eq!(extract_doi("not-a-doi"), None);
        assert_eq!(extract_doi("10.1234"), None); // Missing suffix
        assert_eq!(extract_doi("10.123/"), None); // Invalid registrant
    }
}
