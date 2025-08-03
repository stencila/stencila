use codec::{
    Codec, DecodeInfo, DecodeOptions,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
        serde::de::DeserializeOwned,
        serde_json,
    },
    schema::{self, Node},
    status::Status,
};

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
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let Some(file_info) = decode::extract_github_identifier(identifier) else {
            bail!("Not a recognized GitHub URL")
        };

        // Fetch the raw content
        let content_bytes = decode::fetch_github_file(&file_info).await?;

        // For now, we'll return the content as a string in a SoftwareSourceCode node
        // In the future, this should delegate to the appropriate codec based on file type
        let content = String::from_utf8(content_bytes)?;

        // Create a SoftwareSourceCode node with the fetched content
        let node = Node::SoftwareSourceCode(schema::SoftwareSourceCode {
            name: file_info
                .path
                .split('/')
                .next_back()
                .unwrap_or("")
                .to_string(),
            programming_language: determine_programming_language(&file_info.path).to_string(),
            repository: Some(format!(
                "https://github.com/{}/{}",
                file_info.owner, file_info.repo
            )),
            path: Some(file_info.path.clone()),
            commit: file_info.ref_.clone(),
            options: Box::new(schema::SoftwareSourceCodeOptions {
                text: Some(schema::Text::from(content)),
                url: Some(identifier.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        });

        Ok((node, DecodeInfo::none()))
    }
}

/// Determine programming language from file extension
fn determine_programming_language(path: &str) -> &'static str {
    let ext = path.split('.').next_back().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "tsx" => "typescript",
        "java" => "java",
        "c" => "c",
        "cpp" | "cxx" | "cc" => "cpp",
        "cs" => "csharp",
        "go" => "go",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "r" => "r",
        "sh" | "bash" => "bash",
        "sql" => "sql",
        "html" | "htm" => "html",
        "css" => "css",
        "md" | "markdown" => "markdown",
        "json" => "json",
        "xml" => "xml",
        "yaml" | "yml" => "yaml",
        _ => "text",
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
        Ok(Node::SoftwareSourceCode(code_item.into()))
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
