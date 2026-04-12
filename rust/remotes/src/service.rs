use std::{path::Path, str::FromStr};

use eyre::{Report, bail, eyre};
use url::Url;

use stencila_codec::{
    PushDryRunOptions, PushResult, eyre::Result, stencila_format::Format, stencila_schema::Node,
};

/// Supported remote services and endpoint families.
///
/// A [`RemoteService`] classifies the kind of external endpoint a remote URL
/// refers to and exposes the capabilities needed to interact with it.
///
/// Most variants correspond to services that host remote document versions of a
/// local Stencila document. Some, such as GitHub pull requests, are better
/// understood as remote review or exchange endpoints that still participate in
/// the same local-path-to-remote mapping model.
#[derive(Debug, Clone, Copy)]
pub enum RemoteService {
    /// Google Docs / Drive
    GoogleDocs,

    /// Microsoft 365 / OneDrive
    Microsoft365,

    /// GitHub Issues
    GitHubIssues,

    /// GitHub Pull Requests
    GitHubPullRequests,

    /// Stencila Email
    StencilaEmail,
}

impl FromStr for RemoteService {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "gdoc" | "gdocs" => Ok(RemoteService::GoogleDocs),
            "m365" => Ok(RemoteService::Microsoft365),
            "ghi" => Ok(RemoteService::GitHubIssues),
            "ghpr" => Ok(RemoteService::GitHubPullRequests),
            "email" => Ok(RemoteService::StencilaEmail),
            _ => {
                let url = Url::parse(s).map_err(|_| {
                        eyre!("Invalid target or service: `{s}`. Use 'gdoc', 'm365', 'ghi', 'ghpr', 'email', or a full URL.")
                    })?;
                RemoteService::from_url(&url)
                    .ok_or_else(|| eyre!("URL {url} is not from a supported remote service"))
            }
        }
    }
}

impl RemoteService {
    /// The shorthand name used in CLI arguments.
    pub fn cli_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "gdoc",
            Self::Microsoft365 => "m365",
            Self::GitHubIssues => "ghi",
            Self::GitHubPullRequests => "ghpr",
            Self::StencilaEmail => "email",
        }
    }

    /// A singular display name for user-facing messages.
    pub fn display_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Doc",
            Self::Microsoft365 => "Microsoft 365 doc",
            Self::GitHubIssues => "GitHub Issue",
            Self::GitHubPullRequests => "GitHub pull request",
            Self::StencilaEmail => "Stencila Email attachment",
        }
    }

    /// A plural display name for user-facing messages.
    pub fn display_name_plural(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Docs",
            Self::Microsoft365 => "Microsoft 365",
            Self::GitHubIssues => "GitHub Issues",
            Self::GitHubPullRequests => "GitHub pull requests",
            Self::StencilaEmail => "Stencila Email",
        }
    }

    /// Whether a URL belongs to this remote service family.
    pub fn matches_url(&self, url: &Url) -> bool {
        match self {
            Self::GoogleDocs => url.host_str() == Some("docs.google.com"),
            Self::Microsoft365 => {
                if let Some(host) = url.host_str() {
                    // Personal OneDrive
                    host == "onedrive.live.com" || host == "1drv.ms" ||
                    // Business OneDrive / SharePoint
                    host.ends_with("-my.sharepoint.com") || host.ends_with(".sharepoint.com")
                } else {
                    false
                }
            }
            Self::GitHubIssues => {
                url.host_str() == Some("github.com") && url.path().contains("/issues/")
            }
            Self::GitHubPullRequests => {
                url.host_str() == Some("github.com") && url.path().contains("/pull/")
            }
            Self::StencilaEmail => stencila_cloud::email::matches_url(url),
        }
    }

    /// Infer the remote service from a URL.
    pub fn from_url(url: &Url) -> Option<Self> {
        [
            Self::GoogleDocs,
            Self::Microsoft365,
            Self::GitHubIssues,
            Self::GitHubPullRequests,
            Self::StencilaEmail,
        ]
        .iter()
        .find(|service| service.matches_url(url))
        .copied()
    }

    /// The interchange format used when pulling from or pushing to this service.
    ///
    /// This is the format used at the service boundary before conversion to or
    /// from the local source format.
    pub fn pull_format(&self) -> Format {
        match self {
            Self::GoogleDocs => Format::Docx,
            Self::Microsoft365 => Format::Docx,
            Self::GitHubIssues => Format::Docx,
            Self::GitHubPullRequests => Format::Docx,
            Self::StencilaEmail => Format::Docx,
        }
    }

    /// Whether this service supports pull but not push.
    ///
    /// Read-only remotes can be used as sources of document content but not as
    /// destinations for outbound updates.
    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::GitHubIssues | Self::StencilaEmail)
    }

    /// Whether this service supports push but not pull.
    ///
    /// Write-only remotes participate in the remote model but cannot be treated
    /// as bidirectional synchronization targets. Status calculations therefore
    /// avoid states that imply a meaningful pull path.
    pub fn is_write_only(&self) -> bool {
        matches!(self, Self::GitHubPullRequests)
    }

    /// Push a document to this remote service.
    ///
    /// The exact behavior is service-specific. For classic document remotes this
    /// usually updates or creates a remote document. For review-oriented remotes
    /// such as GitHub pull requests, a push may also project document comments
    /// and suggestions into service-native review constructs.
    pub async fn push(
        &self,
        node: &Node,
        path: Option<&Path>,
        title: Option<&str>,
        url: Option<&Url>,
        dry_run: Option<PushDryRunOptions>,
    ) -> Result<PushResult> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::push(node, path, title, url, dry_run).await,
            Self::Microsoft365 => stencila_codec_m365::push(node, path, title, url, dry_run).await,
            Self::GitHubIssues => {
                bail!("GitHub Issues remote is read-only and does not support push")
            }
            Self::GitHubPullRequests => {
                stencila_codec_github::push_pull_request(node, path, title, url, dry_run).await
            }
            Self::StencilaEmail => {
                bail!("Email Attachments remote is read-only and does not support push")
            }
        }
    }

    /// Pull a document from this remote service.
    ///
    /// Downloads the service representation and saves it to the specified path.
    ///
    /// `target_path` is the path to the local document being updated. Most
    /// services ignore it, but multi-attachment or multi-document remotes such as
    /// GitHub Issues and Stencila Email use it to choose the correct source item.
    pub async fn pull(&self, url: &Url, dest: &Path, target_path: Option<&Path>) -> Result<()> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::pull(url, dest)
                .await
                .map_err(|error| eyre!(error)),
            Self::Microsoft365 => stencila_codec_m365::pull(url, dest)
                .await
                .map_err(|error| eyre!(error)),
            Self::GitHubIssues => stencila_codec_github::issues::pull(url, dest, target_path).await,
            Self::GitHubPullRequests => {
                bail!("GitHub pull request remote is push-only and does not support pull")
            }
            Self::StencilaEmail => stencila_cloud::email::pull(url, dest, target_path, None).await,
        }
    }

    /// Get the remote modification time as a Unix timestamp.
    ///
    /// Not all services expose a meaningful or supported remote modification time.
    pub async fn modified_at(&self, url: &Url) -> Result<u64> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::modified_at(url).await,
            Self::Microsoft365 => stencila_codec_m365::modified_at(url).await,
            Self::GitHubIssues => stencila_codec_github::issues::modified_at(url).await,
            Self::GitHubPullRequests => {
                stencila_codec_github::pull_requests::activity::modified_at(url).await
            }
            Self::StencilaEmail => stencila_cloud::email::modified_at(url).await,
        }
    }

    /// Pull all documents from a multi-document remote using embedded path metadata.
    ///
    /// Returns `(target_path, temp_file)` pairs for the caller to convert or
    /// merge, or `None` if the service does not support batch pull.
    pub async fn pull_all(
        &self,
        url: &Url,
    ) -> Result<Option<Vec<(std::path::PathBuf, tempfile::NamedTempFile)>>> {
        match self {
            Self::StencilaEmail => stencila_cloud::email::pull_all(url, None).await.map(Some),
            Self::GitHubIssues => stencila_codec_github::issues::pull_all(url).await.map(Some),
            _ => Ok(None),
        }
    }
}
