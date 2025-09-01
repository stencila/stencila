use std::path::PathBuf;

use serde::de::DeserializeOwned;

use codec::{
    Codec, DecodeInfo, DecodeOptions,
    async_trait,
    eyre::{Result, bail},
    format::Format,
    schema::{Article, Datatable, Node, SoftwareSourceCode, SoftwareSourceCodeOptions, Text},
    status::Status,
};
use tokio::fs::write;
use codec_csv::CsvCodec;
use codec_ipynb::IpynbCodec;
use codec_latex::LatexCodec;
use codec_markdown::MarkdownCodec;
use codec_xlsx::XlsxCodec;

pub mod client;
pub mod decode;
pub mod responses;
pub mod search_code;
pub mod search_repos;
pub mod search_users;

pub use client::{request, search_url};
pub use responses::{SearchCodeResponse, SearchRepositoriesResponse, SearchUsersResponse};
pub use search_code::CodeSearchItem;
pub use search_repos::RepositorySearchItem;
pub use search_users::UserSearchItem;

/// A codec for decoding GitHub REST API responses to Stencila Schema nodes
///
/// Not exposed as a standalone codec but used by sibling crates that
/// make use of the GitHub API.
///
/// See https://docs.github.com/en/rest for details.
pub struct GithubCodec;

#[async_trait]
impl Codec for GithubCodec {
    fn name(&self) -> &str {
        "github"
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

impl GithubCodec {
    /// Check if an identifier is a supported GitHub URL
    pub fn supports_identifier(identifier: &str) -> bool {
        decode::extract_github_identifier(identifier).is_some()
    }

    /// Decode a Stencila [`Node`] from a GitHub identifier (URL)
    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let Some(file_info) = decode::extract_github_identifier(identifier) else {
            bail!("Not a recognized GitHub URL")
        };

        // Fetch the raw content
        let content_bytes = decode::fetch_github_file(&file_info).await?;

        // Determine the format from the file path
        let path = PathBuf::from(&file_info.path);
        let format = Format::from_path(&path);

        // Delegate to the appropriate codec based on format
        let (mut node, info) = match format {
            Format::Xlsx | Format::Xls | Format::Ods => {
                // Spreadsheet formats require binary file access, so we create a temporary file
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
                        // For other formats, return as SoftwareSourceCode
                        let node = Node::SoftwareSourceCode(SoftwareSourceCode {
                            name: file_info.file_name(),
                            programming_language: file_info.lang(),
                            options: Box::new(SoftwareSourceCodeOptions {
                                text: Some(Text::from(content)),
                                url: Some(identifier.to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                        (node, DecodeInfo::none())
                    }
                }
            }
        };

        // Set repository, path & commit properties on the work
        let repository = Some(file_info.repo_url());
        let path = Some(file_info.path);
        let commit = file_info.ref_;
        match &mut node {
            Node::Article(Article { options, .. }) => {
                options.repository = repository;
                options.path = path;
                options.commit = commit;
            }
            Node::SoftwareSourceCode(code) => {
                code.repository = repository;
                code.path = path;
                code.commit = commit;
            }
            Node::Datatable(Datatable { options, .. }) => {
                options.repository = repository;
                options.path = path;
                options.commit = commit;
            }
            _ => {
                // For other node types, we don't set these properties
            }
        }

        Ok((node, info))
    }
}

/// Decode a Stencila [`Node`] from a GitHub response JSON of known type
pub fn from_str<T>(json: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_str(json)?)
}

/// Decode a Stencila [`Node`] from a GitHub response JSON of unknown type
pub fn from_str_any(json: &str) -> Result<Node> {
    let value: serde_json::Value = serde_json::from_str(json)?;

    let node = if let Some(items) = value.get("items") {
        if let Some(first_item_value) = items.as_array().and_then(|arr| arr.first()) {
            from_value_any(first_item_value)?
        } else {
            bail!("Empty GitHub search response")
        }
    } else {
        from_value_any(&value)?
    };

    Ok(node)
}

/// Decode a Stencila [`Node`] from a [`serde_json::Value`] in a GitHub response JSON
pub fn from_value_any(value: &serde_json::Value) -> Result<Node> {
    // Check if it's a code search item
    if value.get("name").is_some()
        && value.get("path").is_some()
        && value.get("repository").is_some()
    {
        let code_item: CodeSearchItem = serde_json::from_value(value.clone())?;
        Ok(code_item.into())
    }
    // Check if it's a user search item
    else if value.get("login").is_some()
        && value.get("avatar_url").is_some()
        && value.get("type").is_some()
    {
        let user_item: UserSearchItem = serde_json::from_value(value.clone())?;
        Ok(user_item.into())
    }
    // Check if it's a repository search item
    else if value.get("full_name").is_some()
        && value.get("owner").is_some()
        && value.get("html_url").is_some()
    {
        let repo_item: RepositorySearchItem = serde_json::from_value(value.clone())?;
        Ok(repo_item.into())
    } else {
        bail!("Unsupported GitHub API response format")
    }
}
