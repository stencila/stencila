use std::path::PathBuf;

use serde::de::DeserializeOwned;

use codec::{
    Codec, DecodeInfo, DecodeOptions,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
        serde_json, tempfile,
        tokio::fs::write,
    },
    format::Format,
    schema::{
        Article, Datatable, Node, SoftwareSourceCode, SoftwareSourceCodeOptions, StringOrNumber,
    },
    status::Status,
};
use codec_csv::CsvCodec;
use codec_ipynb::IpynbCodec;
use codec_latex::LatexCodec;
use codec_markdown::MarkdownCodec;
use codec_xlsx::XlsxCodec;

pub mod client;
pub mod decode;
pub mod responses;
pub mod search_records;

pub use client::{request, search_url};
pub use decode::{
    ZenodoRecordInfo, extract_zenodo_identifier, fetch_zenodo_file, fetch_zenodo_record,
};
pub use responses::{Record, SearchRecordsResponse};

/// A codec for decoding Zenodo REST API responses to Stencila Schema nodes
///
/// This codec allows searching for and decoding resources (datasets, publications,
/// software) from Zenodo (https://zenodo.org), an open-access repository.
///
/// See https://developers.zenodo.org/ for API details.
pub struct ZenodoCodec;

#[async_trait]
impl Codec for ZenodoCodec {
    fn name(&self) -> &str {
        "zenodo"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    async fn from_str(
        &self,
        json: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        Ok((from_str_any(json)?, DecodeInfo::none()))
    }
}

impl ZenodoCodec {
    /// Check if an identifier is a supported Zenodo URL or DOI
    pub fn supports_identifier(identifier: &str) -> bool {
        extract_zenodo_identifier(identifier).is_some()
    }

    /// Decode a Stencila [`Node`] from a Zenodo identifier (URL or DOI)
    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let Some(record_info) = extract_zenodo_identifier(identifier) else {
            bail!("Not a recognized Zenodo URL or DOI")
        };

        // Fetch the record metadata
        let record = fetch_zenodo_record(&record_info).await?;

        // If there are no files, convert the record metadata directly
        if record.files.is_empty() {
            let node: Node = record.into();
            return Ok((node, DecodeInfo::none()));
        }

        // Store metadata we'll need later
        let doi_url = record
            .metadata
            .doi
            .as_ref()
            .map(|doi| format!("https://doi.org/{doi}"));
        let repository = Some(record.links.html.clone());
        let version = record.metadata.version.clone();

        // Find the most appropriate file to decode
        let selected_file = select_primary_file(&record)?;

        // Fetch the file content
        let content_bytes = fetch_zenodo_file(&selected_file.links.self_).await?;

        // Determine the format from the file path
        let path = PathBuf::from(&selected_file.key);
        let format = Format::from_path(&path);

        // Delegate to the appropriate codec based on format
        let (mut node, info) = match format {
            Format::Xlsx | Format::Xls | Format::Ods => {
                // Spreadsheet formats require binary file access
                let temp_file = tempfile::Builder::new()
                    .suffix(&format!(".{format}"))
                    .tempfile()?;
                let temp_path = temp_file.path();

                // Write binary content to temporary file
                write(temp_path, &content_bytes).await?;

                // Decode using XlsxCodec from_path
                let (node, _losses, info) = XlsxCodec.from_path(temp_path, options).await?;
                (node, info)
            }
            _ => {
                // For text-based formats, convert to string
                let content = String::from_utf8(content_bytes)?;
                match format {
                    Format::Ipynb => IpynbCodec.from_str(&content, options).await?,
                    Format::Latex => LatexCodec.from_str(&content, options).await?,
                    Format::Markdown | Format::Myst | Format::Qmd | Format::Smd => {
                        MarkdownCodec.from_str(&content, options).await?
                    }
                    Format::Csv | Format::Tsv | Format::Parquet | Format::Arrow => {
                        CsvCodec.from_str(&content, options).await?
                    }
                    _ => {
                        // For other formats, use the SoftwareSourceCode representation
                        let node = Node::SoftwareSourceCode(SoftwareSourceCode {
                            name: selected_file.key.clone(),
                            programming_language: String::new(),
                            doi: record.metadata.doi.clone(),
                            version: version.clone().map(StringOrNumber::String),
                            options: Box::new(SoftwareSourceCodeOptions {
                                url: Some(selected_file.links.self_.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                        (node, DecodeInfo::none())
                    }
                }
            }
        };

        // Set Zenodo-specific metadata on the work

        match &mut node {
            Node::Article(Article { options, .. }) => {
                options.repository = repository;
                options.url = doi_url.clone();
                options.version = version.map(StringOrNumber::String);
                options.identifiers =
                    doi_url.map(|url| vec![codec::schema::PropertyValueOrString::String(url)]);
            }
            Node::SoftwareSourceCode(code) => {
                code.repository = repository;
                code.version = version.map(StringOrNumber::String);
                code.options.url = doi_url.clone();
                code.options.identifiers =
                    doi_url.map(|url| vec![codec::schema::PropertyValueOrString::String(url)]);
            }
            Node::Datatable(Datatable { options, .. }) => {
                options.repository = repository;
                options.url = doi_url.clone();
                options.version = version.map(StringOrNumber::String);
                options.identifiers =
                    doi_url.map(|url| vec![codec::schema::PropertyValueOrString::String(url)]);
            }
            _ => {
                // For other node types, we don't set these properties
            }
        }

        Ok((node, info))
    }

    /// Search Zenodo records
    pub async fn search(
        query: &str,
        communities: Option<&str>,
        record_type: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<SearchRecordsResponse> {
        let mut params = vec![("q", query.to_string())];

        if let Some(communities) = communities {
            params.push(("communities", communities.to_string()));
        }

        if let Some(record_type) = record_type {
            params.push(("type", record_type.to_string()));
        }

        if let Some(page) = page {
            params.push(("page", page.to_string()));
        }

        if let Some(size) = size {
            params.push(("size", size.to_string()));
        }

        let url = search_url(&params);
        request(&url).await
    }
}

/// Select the primary file from a Zenodo record
///
/// Prioritizes files based on format and size
fn select_primary_file(record: &Record) -> Result<&responses::FileInfo> {
    // Priority order for file formats
    let priority_formats = [
        Format::Ipynb,
        Format::Markdown,
        Format::Csv,
        Format::Xlsx,
        Format::Latex,
        Format::Pdf,
    ];

    // Try to find a file matching priority formats
    for format in &priority_formats {
        if let Some(file) = record.files.iter().find(|f| {
            let path = PathBuf::from(&f.key);
            Format::from_path(&path) == *format
        }) {
            return Ok(file);
        }
    }

    // If no priority format found, return the first file
    record
        .files
        .first()
        .ok_or_else(|| codec::common::eyre::eyre!("No files available in Zenodo record"))
}

/// Decode a Stencila [`Node`] from a Zenodo response JSON of known type
pub fn from_str<T>(json: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_str(json)?)
}

/// Decode a Stencila [`Node`] from a Zenodo response JSON of unknown type
pub fn from_str_any(json: &str) -> Result<Node> {
    let value: serde_json::Value = serde_json::from_str(json)?;

    // Check if it's a search response
    if value.get("hits").is_some() {
        let response: SearchRecordsResponse = serde_json::from_value(value)?;
        if let Some(first_record) = response.hits.hits.into_iter().next() {
            Ok(first_record.into())
        } else {
            bail!("Empty Zenodo search response")
        }
    }
    // Check if it's a single record
    else if value.get("metadata").is_some() {
        let record: Record = serde_json::from_value(value)?;
        Ok(record.into())
    } else {
        bail!("Unsupported Zenodo API response format")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_supports_identifier() {
        // Test that codec supports various Zenodo identifiers
        assert!(ZenodoCodec::supports_identifier(
            "https://zenodo.org/records/12345678"
        ));
        assert!(ZenodoCodec::supports_identifier(
            "https://zenodo.org/record/12345678"
        ));
        assert!(ZenodoCodec::supports_identifier("10.5281/zenodo.12345678"));
        assert!(ZenodoCodec::supports_identifier(
            "https://doi.org/10.5281/zenodo.12345678"
        ));

        // Test that codec doesn't support non-Zenodo identifiers
        assert!(!ZenodoCodec::supports_identifier(
            "https://github.com/user/repo"
        ));
        assert!(!ZenodoCodec::supports_identifier("https://example.com"));
    }
}
