use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use url::Url;

use codec::eyre::{Result, bail};

use crate::client::{api_url, request};

/// GitHub file information extracted from a URL
#[derive(Debug, Clone)]
pub struct GithubFileInfo {
    pub owner: String,
    pub repo: String,
    pub path: String,
    pub ref_: Option<String>,
}

impl GithubFileInfo {
    pub fn repo_url(&self) -> String {
        ["https://github.com/", &self.owner, "/", &self.repo].concat()
    }

    pub fn file_name(&self) -> String {
        self.path.split('/').next_back().unwrap_or("").to_string()
    }

    pub fn lang(&self) -> String {
        let ext = self.path.split('.').next_back().unwrap_or("");
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
        .to_string()
    }
}

/// Extract GitHub file information from an identifier (URL)
///
/// Supports the following URL patterns:
/// - https://github.com/owner/repo/blob/ref/path/to/file
/// - https://github.com/owner/repo/tree/ref/path/to/file
/// - https://raw.githubusercontent.com/owner/repo/ref/path/to/file
pub(super) fn extract_github_identifier(identifier: &str) -> Option<GithubFileInfo> {
    if let Ok(url) = Url::parse(identifier) {
        match url.host_str() {
            Some("github.com") | Some("www.github.com") => {
                // Parse github.com URLs
                let path_segments: Vec<_> = url.path_segments()?.collect();
                if path_segments.len() >= 5 {
                    let owner = path_segments[0];
                    let repo = path_segments[1];
                    let action = path_segments[2]; // "blob" or "tree"

                    if action == "blob" || action == "tree" {
                        let ref_ = path_segments[3];
                        let file_path = path_segments[4..].join("/");

                        return Some(GithubFileInfo {
                            owner: owner.to_string(),
                            repo: repo.to_string(),
                            path: file_path,
                            ref_: Some(ref_.to_string()),
                        });
                    }
                }
            }
            Some("raw.githubusercontent.com") => {
                // Parse raw.githubusercontent.com URLs
                let path_segments: Vec<_> = url.path_segments()?.collect();
                if path_segments.len() >= 4 {
                    let owner = path_segments[0];
                    let repo = path_segments[1];
                    let ref_ = path_segments[2];
                    let file_path = path_segments[3..].join("/");

                    return Some(GithubFileInfo {
                        owner: owner.to_string(),
                        repo: repo.to_string(),
                        path: file_path,
                        ref_: Some(ref_.to_string()),
                    });
                }
            }
            _ => {}
        }
    }

    None
}

/// Response from GitHub Contents API
#[derive(Debug, Deserialize)]
struct ContentsResponse {
    content: Option<String>,
    encoding: String,
    size: u64,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    file_type: String,
    download_url: Option<String>,
}

/// Fetch raw content from a GitHub file
#[tracing::instrument]
pub(super) async fn fetch_github_file(file_info: &GithubFileInfo) -> Result<Vec<u8>> {
    tracing::debug!("Fetching file from GitHub");

    // Build API URL for the contents endpoint
    let mut api_path = format!(
        "/repos/{}/{}/contents/{}",
        file_info.owner, file_info.repo, file_info.path
    );
    if let Some(ref_) = &file_info.ref_ {
        api_path.push_str("?ref=");
        api_path.push_str(ref_);
    };

    // Fetch the file content from GitHub
    let response: ContentsResponse = request(&api_url(&api_path)).await?;

    tracing::debug!(
        "GitHub API response: size={} bytes, encoding={}",
        response.size,
        response.encoding
    );

    let content_bytes = if let Some(content) = response.content {
        // Decode base64 content for smaller files
        if response.encoding != "base64" {
            bail!("Unsupported encoding: {}", response.encoding);
        }

        STANDARD.decode(content.replace('\n', ""))?
    } else if let Some(download_url) = response.download_url {
        tracing::debug!("Downloading contents from: {download_url}");
        let client = reqwest::Client::new();
        let raw_response = client.get(&download_url).send().await?;
        let content_bytes = raw_response.bytes().await?;
        content_bytes.to_vec()
    } else {
        bail!("File is too large and no download URL provided by GitHub API");
    };

    Ok(content_bytes)
}

#[cfg(test)]
mod tests {
    use codec::eyre::OptionExt;

    use super::*;

    #[test]
    fn test_extract_github_identifier() -> Result<()> {
        // Test github.com blob URLs
        let info =
            extract_github_identifier("https://github.com/owner/repo/blob/main/path/to/file.rs")
                .ok_or_eyre("expected some")?;
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.path, "path/to/file.rs");
        assert_eq!(info.ref_, Some("main".to_string()));

        // Test github.com tree URLs
        let info =
            extract_github_identifier("https://github.com/owner/repo/tree/v1.0.0/src/lib.rs")
                .ok_or_eyre("expected some")?;
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.path, "src/lib.rs");
        assert_eq!(info.ref_, Some("v1.0.0".to_string()));

        // Test raw.githubusercontent.com URLs
        let info = extract_github_identifier(
            "https://raw.githubusercontent.com/owner/repo/main/README.md",
        )
        .ok_or_eyre("expected some")?;
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.path, "README.md");
        assert_eq!(info.ref_, Some("main".to_string()));

        // Test invalid URLs
        assert!(extract_github_identifier("https://example.com/file").is_none());
        assert!(extract_github_identifier("not-a-url").is_none());

        Ok(())
    }
}
