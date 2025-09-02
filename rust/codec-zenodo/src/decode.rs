use url::Url;

use stencila_codec::eyre::{Result, bail};

use crate::{
    client::{api_url, request},
    responses::Record,
};

/// Zenodo record information extracted from a URL or DOI
#[derive(Debug, Clone)]
pub struct ZenodoRecordInfo {
    pub record_id: String,
    pub version: Option<String>,
    pub is_doi: bool,
}

impl ZenodoRecordInfo {
    pub fn record_url(&self) -> String {
        format!("https://zenodo.org/records/{}", self.record_id)
    }

    pub fn api_url(&self) -> String {
        api_url(&format!("/records/{}", self.record_id))
    }
}

/// Extract Zenodo record information from an identifier (URL or DOI)
///
/// Supports the following patterns:
/// - https://zenodo.org/records/12345678
/// - https://zenodo.org/record/12345678 (legacy)
/// - https://doi.org/10.5281/zenodo.12345678
/// - 10.5281/zenodo.12345678 (raw DOI)
pub fn extract_zenodo_identifier(identifier: &str) -> Option<ZenodoRecordInfo> {
    // Try to parse as DOI first (raw DOI format)
    if identifier.starts_with("10.5281/zenodo.") {
        let doi_suffix = identifier.strip_prefix("10.5281/zenodo.")?;

        // Check if it's just numbers or has a version
        let (id, version) = if let Some(dot_pos) = doi_suffix.find('.') {
            (
                doi_suffix[..dot_pos].to_string(),
                Some(doi_suffix[dot_pos + 1..].to_string()),
            )
        } else {
            (doi_suffix.to_string(), None)
        };

        return Some(ZenodoRecordInfo {
            record_id: id,
            version,
            is_doi: true,
        });
    }

    // Try to parse as URL
    if let Ok(url) = Url::parse(identifier) {
        match url.host_str() {
            Some("zenodo.org") | Some("www.zenodo.org") => {
                // Parse zenodo.org URLs
                let path_segments: Vec<_> = url.path_segments()?.collect();
                if path_segments.len() >= 2 {
                    let collection = path_segments[0];

                    // Handle both /records/ and /record/ (legacy) URLs
                    if collection == "records" || collection == "record" {
                        let record_id = path_segments[1].to_string();

                        return Some(ZenodoRecordInfo {
                            record_id,
                            version: None,
                            is_doi: false,
                        });
                    }
                }
            }
            Some("doi.org") | Some("dx.doi.org") => {
                // Parse DOI URLs
                let path = url.path();
                if path.contains("10.5281/zenodo.") {
                    // Extract the record ID from the DOI path
                    let doi_part = path.trim_start_matches('/');
                    return extract_zenodo_identifier(doi_part);
                }
            }
            _ => {}
        }
    }

    None
}

/// Fetch a Zenodo record by ID
#[tracing::instrument]
pub async fn fetch_zenodo_record(record_info: &ZenodoRecordInfo) -> Result<Record> {
    tracing::debug!("Fetching record from Zenodo");

    let record: Record = request(&record_info.api_url()).await?;

    tracing::debug!(
        "Zenodo API response: {} files, resource type: {}",
        record.files.len(),
        record.metadata.resource_type.type_
    );

    Ok(record)
}

/// Fetch a file from a Zenodo record
#[tracing::instrument]
pub async fn fetch_zenodo_file(file_url: &str) -> Result<Vec<u8>> {
    tracing::debug!("Fetching file from Zenodo: {}", file_url);

    let client = reqwest::Client::new();
    let response = client.get(file_url).send().await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("Failed to download file: {error}");
    }

    let content_bytes = response.bytes().await?;
    Ok(content_bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_zenodo_identifier() {
        // Test Zenodo record URLs
        let info = extract_zenodo_identifier("https://zenodo.org/records/12345678")
            .expect("should extract");
        assert_eq!(info.record_id, "12345678");
        assert!(!info.is_doi);

        // Test legacy Zenodo record URLs
        let info = extract_zenodo_identifier("https://zenodo.org/record/12345678")
            .expect("should extract");
        assert_eq!(info.record_id, "12345678");
        assert!(!info.is_doi);

        // Test DOI URLs
        let info = extract_zenodo_identifier("https://doi.org/10.5281/zenodo.12345678")
            .expect("should extract");
        assert_eq!(info.record_id, "12345678");
        assert!(info.is_doi);

        // Test raw DOI
        let info = extract_zenodo_identifier("10.5281/zenodo.12345678").expect("should extract");
        assert_eq!(info.record_id, "12345678");
        assert!(info.is_doi);

        // Test versioned DOI
        let info = extract_zenodo_identifier("10.5281/zenodo.12345678.v2").expect("should extract");
        assert_eq!(info.record_id, "12345678");
        assert_eq!(info.version, Some("v2".to_string()));
        assert!(info.is_doi);

        // Test invalid URLs
        assert!(extract_zenodo_identifier("https://example.com/file").is_none());
        assert!(extract_zenodo_identifier("not-a-url").is_none());
        assert!(extract_zenodo_identifier("10.1234/other.12345").is_none());
    }

    #[test]
    fn test_zenodo_identifier_extraction() {
        // Test various Zenodo URL formats
        assert!(extract_zenodo_identifier("https://zenodo.org/records/12345678").is_some());
        assert!(extract_zenodo_identifier("https://zenodo.org/record/12345678").is_some());
        assert!(extract_zenodo_identifier("https://doi.org/10.5281/zenodo.12345678").is_some());
        assert!(extract_zenodo_identifier("10.5281/zenodo.12345678").is_some());

        // Test invalid identifiers
        assert!(extract_zenodo_identifier("https://github.com/user/repo").is_none());
        assert!(extract_zenodo_identifier("https://example.com").is_none());
    }

    #[test]
    fn test_zenodo_doi_parsing() {
        let info = extract_zenodo_identifier("10.5281/zenodo.12345678").expect("some");
        assert_eq!(info.record_id, "12345678");
        assert!(info.is_doi);

        let info = extract_zenodo_identifier("10.5281/zenodo.12345678.v2").expect("some");
        assert_eq!(info.record_id, "12345678");
        assert_eq!(info.version, Some("v2".to_string()));
    }

    #[test]
    fn test_zenodo_url_parsing() {
        let info = extract_zenodo_identifier("https://zenodo.org/records/12345678").expect("some");
        assert_eq!(info.record_id, "12345678");
        assert!(!info.is_doi);

        let info = extract_zenodo_identifier("https://zenodo.org/record/12345678").expect("some");
        assert_eq!(info.record_id, "12345678");
        assert!(!info.is_doi);
    }
}
