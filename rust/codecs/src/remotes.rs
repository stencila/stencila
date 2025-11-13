use std::path::Path;

use clap::ValueEnum;
use url::Url;

use stencila_codec::{eyre::Result, stencila_format::Format, stencila_schema::Node};

/// Remote document services
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RemoteService {
    /// Google Docs / Drive
    #[value(name = "gdocs")]
    GoogleDocs,

    /// Microsoft 365 / OneDrive
    #[value(name = "m365")]
    Microsoft365,
}

impl RemoteService {
    /// Check if a URL matches this remote service
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
        }
    }

    /// Get the remote service from a URL
    pub fn from_url(url: &Url) -> Option<Self> {
        [Self::GoogleDocs, Self::Microsoft365]
            .iter()
            .find(|service| service.matches_url(url))
            .copied()
    }

    /// Get the display name for user-facing messages
    pub fn display_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Doc",
            Self::Microsoft365 => "Microsoft 365 document",
        }
    }

    /// Get the plural display name for user-facing messages
    pub fn display_name_plural(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Docs",
            Self::Microsoft365 => "Microsoft 365",
        }
    }

    /// Get the CLI value name (e.g., "gdocs")
    pub fn cli_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "gdocs",
            Self::Microsoft365 => "m365",
        }
    }

    /// Get the format used by this remote service for pull/push operations
    pub fn pull_format(&self) -> Format {
        match self {
            Self::GoogleDocs => Format::Docx,
            Self::Microsoft365 => Format::Docx,
        }
    }

    /// Push a document to this remote service
    pub async fn push(
        &self,
        node: &Node,
        path: Option<&Path>,
        title: Option<&str>,
        url: Option<&Url>,
    ) -> Result<Url> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::push(node, path, title, url).await,
            Self::Microsoft365 => stencila_codec_m365::push(node, path, title, url).await,
        }
    }

    /// Pull a document from this remote service
    ///
    /// Downloads the document and saves it to the specified path.
    pub async fn pull(&self, url: &Url, dest: &Path) -> Result<()> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::pull(url, dest).await,
            Self::Microsoft365 => stencila_codec_m365::pull(url, dest).await,
        }
    }
}
