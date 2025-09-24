use url::Url;

use stencila_codec::eyre::{Result, bail};

use crate::{
    client::{api_url, request},
    responses::Record,
};

/// Zenodo record information extracted from a URL or DOI
///
/// Note that Zenodo DOIs do not include a version suffix and that each version has a unique DOI
/// See https://zenodo.org/help/versioning.
#[derive(Debug, Clone)]
pub(crate) struct RecordInfo {
    /// The Zenodo record id (e.g. "17189289")
    pub id: String,

    /// The path of the file within the record
    pub file: Option<String>,
}

impl RecordInfo {
    /// The web page URL of the record
    #[allow(dead_code)]
    pub fn web_url(&self) -> String {
        format!("https://zenodo.org/records/{}", self.id)
    }

    /// The API URL of the record
    pub fn api_url(&self) -> String {
        api_url(&format!("/records/{}", self.id))
    }
}

/// Extract Zenodo record information from an identifier (URL or DOI)
///
/// Supports the following patterns.
///
/// Zenodo URLs:
/// - https://zenodo.org/records/12345678
/// - https://zenodo.org/record/12345678 (legacy)
/// - https://zenodo.org/records/12345678/files/filename.ext (with file path)
/// - https://zenodo.org/records/12345678/files/filename.ext?download=1 (with query params)
///
/// DOI URLs
/// - https://doi.org/10.5281/zenodo.12345678
/// - https://dx.doi.org/10.5281/zenodo.12345678
///
/// Bare DOIs
/// - 10.5281/zenodo.12345678
pub(crate) fn extract_record_info(identifier: &str) -> Option<RecordInfo> {
    // Try to parse as DOI first (without any file path)
    if let Some(id) = identifier.strip_prefix("10.5281/zenodo.") {
        return Some(RecordInfo {
            id: id.into(),
            file: None,
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
                        let id = path_segments[1].to_string();

                        // Check for file path after the record ID
                        let file_path = if matches!(path_segments.get(2), Some(&"files"))
                            && let Some(file) = path_segments.get(3)
                        {
                            Some(file.to_string())
                        } else {
                            None
                        };

                        return Some(RecordInfo {
                            id,
                            file: file_path,
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
                    return extract_record_info(doi_part);
                }
            }
            _ => {}
        }
    }

    None
}

/// Fetch a Zenodo record by ID
#[tracing::instrument]
pub(crate) async fn fetch_record(record_info: &RecordInfo) -> Result<Record> {
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
pub(crate) async fn fetch_file(file_url: &str) -> Result<Vec<u8>> {
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
    fn test_extract_record_info() {
        // Test Zenodo record URLs
        let info =
            extract_record_info("https://zenodo.org/records/12345678").expect("should extract");
        assert_eq!(info.id, "12345678");

        // Test legacy Zenodo record URLs
        let info =
            extract_record_info("https://zenodo.org/record/12345678").expect("should extract");
        assert_eq!(info.id, "12345678");

        // Test DOI URLs
        let info =
            extract_record_info("https://doi.org/10.5281/zenodo.12345678").expect("should extract");
        assert_eq!(info.id, "12345678");

        // Test raw DOI
        let info = extract_record_info("10.5281/zenodo.12345678").expect("should extract");
        assert_eq!(info.id, "12345678");

        // Test invalid URLs
        assert!(extract_record_info("https://example.com/file").is_none());
        assert!(extract_record_info("not-a-url").is_none());
        assert!(extract_record_info("10.1234/other.12345").is_none());

        // Test Zenodo URLs with file paths
        let info = extract_record_info("https://zenodo.org/records/14786795/files/BodyLength.Rmd")
            .expect("should extract");
        assert_eq!(info.id, "14786795");
        assert_eq!(info.file, Some("BodyLength.Rmd".to_string()));

        // Test Zenodo URLs with file paths and query parameters
        let info = extract_record_info(
            "https://zenodo.org/records/14786795/files/BodyLength.Rmd?download=1",
        )
        .expect("should extract");
        assert_eq!(info.id, "14786795");
        assert_eq!(info.file, Some("BodyLength.Rmd".to_string()));

        // Test legacy Zenodo URLs with file paths
        let info = extract_record_info("https://zenodo.org/record/12345678/files/data.csv")
            .expect("should extract");
        assert_eq!(info.id, "12345678");
        assert_eq!(info.file, Some("data.csv".to_string()));

        // Test various Zenodo URL formats
        assert!(extract_record_info("https://zenodo.org/records/12345678").is_some());
        assert!(extract_record_info("https://zenodo.org/record/12345678").is_some());
        assert!(extract_record_info("https://doi.org/10.5281/zenodo.12345678").is_some());
        assert!(extract_record_info("10.5281/zenodo.12345678").is_some());

        // Test invalid identifiers
        assert!(extract_record_info("https://github.com/user/repo").is_none());
        assert!(extract_record_info("https://example.com").is_none());
    }
}
