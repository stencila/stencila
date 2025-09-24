use std::{sync::LazyLock, time::Duration};

use regex::Regex;
use reqwest::{Client, header};
use url::Url;

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, StructuringOptions, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_csl::CslCodec;
use stencila_version::STENCILA_USER_AGENT;

/// A codec for decoding DOIs into Stencila [`Node`]
///
/// This codec is used for fetching metadata for an [`Node`] having
/// a DOI. It is used to supplement other codecs, such as `codec-arxiv`,
/// `codec-openrxiv`, and `codec-pmcoa` by providing standardized metadata
/// for properties such as authors and references, which may not be well
/// supported by those codecs.
///
/// CSL-JSON is used because it is most widely supported across registries
/// such as DataCite and Crossref.
pub struct DoiCodec;

#[async_trait]
impl Codec for DoiCodec {
    fn name(&self) -> &str {
        "doi"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Csl => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        StructuringOptions::none()
    }
}

impl DoiCodec {
    pub fn supports_identifier(identifier: &str) -> bool {
        extract_doi(identifier).is_some()
    }

    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo, StructuringOptions)> {
        let Some(doi) = extract_doi(identifier) else {
            bail!("Not a recognized DOI")
        };

        tracing::debug!("Decoding DOI {doi}");

        static CLIENT: LazyLock<Client> = LazyLock::new(|| {
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::ACCEPT,
                header::HeaderValue::from_static("application/vnd.citationstyles.csl+json"),
            );

            Client::builder()
                .default_headers(headers)
                .user_agent(STENCILA_USER_AGENT)
                .timeout(Duration::from_secs(30))
                .build()
                .expect("invalid client")
        });

        let response = CLIENT.get(format!("https://doi.org/{doi}")).send().await?;
        response.error_for_status_ref()?;

        let json = response.text().await?;

        let (node, info) = CslCodec.from_str(&json, options).await?;
        let structuring_options = Self.structuring_options(&Format::Csl);

        Ok((node, info, structuring_options))
    }
}

/// Extract a DOI from an identifier string
///
/// Extracts the DOI from:
///
/// - a bare DOI e.g. 10.0001/abcd.123
///
/// - a DOI prefixed with label e.g. doi:10.0001/abcd.123 (should be flexible to label format)
///
/// - a DOI URL e.g. https://doi.org/10.0001/abcd.123 (or http:// or dx.doi.org subdomain)
fn extract_doi(identifier: &str) -> Option<String> {
    let input = identifier.trim();

    static DOI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^10\.\d{4,9}/[-._;()/:A-Z0-9]+$").expect("invalid regex")
    });

    // Handle DOI URLs
    if let Ok(url) = input.parse::<Url>()
        && let Some(host) = url.host_str()
        && (host == "doi.org" || host == "dx.doi.org" || host == "www.doi.org")
    {
        let path = url.path().trim_start_matches('/');
        if DOI_REGEX.is_match(path) {
            return Some(path.to_string());
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
