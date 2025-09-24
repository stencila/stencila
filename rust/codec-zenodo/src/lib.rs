use std::path::PathBuf;

use serde::de::DeserializeOwned;
use tokio::fs::write;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, StructuringOptions, async_trait,
    eyre::{Result, bail, eyre},
    stencila_format::Format,
    stencila_schema::{
        Article, Datatable, Node, PropertyValueOrString, SoftwareSourceCode,
        SoftwareSourceCodeOptions, StringOrNumber,
    },
};
use stencila_codec_csv::CsvCodec;
use stencila_codec_ipynb::IpynbCodec;
use stencila_codec_latex::LatexCodec;
use stencila_codec_markdown::MarkdownCodec;
use stencila_codec_xlsx::XlsxCodec;

pub mod client;
pub mod conversion;
pub mod decode;
pub mod responses;

pub use client::{request, search_url};
use decode::{extract_record_info, fetch_file, fetch_record};
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

    fn structuring_options(&self, format: &Format) -> StructuringOptions {
        // Delegate to the relevant codec for each format
        match format {
            Format::Csv | Format::Tsv => CsvCodec.structuring_options(format),
            Format::Ipynb => IpynbCodec.structuring_options(format),
            Format::Latex => LatexCodec.structuring_options(format),
            Format::Markdown | Format::Myst | Format::Qmd | Format::Smd => {
                MarkdownCodec.structuring_options(format)
            }
            Format::Xlsx | Format::Xls | Format::Ods => XlsxCodec.structuring_options(format),
            _ => StructuringOptions::default(),
        }
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
        extract_record_info(identifier).is_some()
    }

    /// Decode a Stencila [`Node`] from a Zenodo identifier (URL or DOI)
    ///
    /// If the identifier includes a specific file path (e.g., "/files/data.csv"),
    /// that specific file will be fetched and decoded. Otherwise, the primary
    /// file will be selected based on format priority.
    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo, StructuringOptions)> {
        let Some(record_info) = extract_record_info(identifier) else {
            bail!("Not a recognized Zenodo URL or DOI")
        };

        // Fetch the record metadata
        let record = fetch_record(&record_info).await?;

        // If there are no files, convert the record metadata directly
        if record.files.is_empty() {
            let node: Node = record.into();
            return Ok((node, DecodeInfo::none(), StructuringOptions::none()));
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
        let selected_file = if let Some(file_path) = &record_info.file {
            // If a specific file path was requested, find that file
            record
                .files
                .iter()
                .find(|f| f.key == *file_path)
                .ok_or_else(|| eyre!("Requested file '{}' not found in Zenodo record", file_path))?
        } else {
            // Otherwise, return the first file
            record
                .files
                .first()
                .ok_or_else(|| eyre!("No files available in Zenodo record"))?
        };

        // Fetch the file content
        let content_bytes = fetch_file(&selected_file.links.self_).await?;

        // Determine the format from the file path
        let path = PathBuf::from(&selected_file.key);
        let format = Format::from_path(&path);

        // Delegate to the appropriate codec based on format
        let (mut node, decode_info) = match format {
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
                options.identifiers = doi_url.map(|url| vec![PropertyValueOrString::String(url)]);
            }
            Node::SoftwareSourceCode(code) => {
                code.repository = repository;
                code.version = version.map(StringOrNumber::String);
                code.options.url = doi_url.clone();
                code.options.identifiers =
                    doi_url.map(|url| vec![PropertyValueOrString::String(url)]);
            }
            Node::Datatable(Datatable { options, .. }) => {
                options.repository = repository;
                options.url = doi_url.clone();
                options.version = version.map(StringOrNumber::String);
                options.identifiers = doi_url.map(|url| vec![PropertyValueOrString::String(url)]);
            }
            _ => {
                // For other node types, we don't set these properties
            }
        }

        let structuring_options = Self.structuring_options(&format);

        Ok((node, decode_info, structuring_options))
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
