use std::path::Path;

use clap::ValueEnum;
use url::Url;

use stencila_codec::{
    PushDryRunOptions, PushResult, eyre::Result, stencila_format::Format, stencila_schema::Node,
};

/// Remote document services
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RemoteService {
    /// Google Docs / Drive
    #[value(name = "gdoc", alias = "gdocs")]
    GoogleDocs,

    /// Microsoft 365 / OneDrive
    #[value(name = "m365")]
    Microsoft365,

    /// Stencila Sites
    #[value(name = "site", alias = "sites")]
    StencilaSites,
}

impl RemoteService {
    /// Get the CLI value name
    pub fn cli_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "gdoc",
            Self::Microsoft365 => "m365",
            Self::StencilaSites => "site",
        }
    }

    /// Get the display name for user-facing messages
    pub fn display_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Doc",
            Self::Microsoft365 => "Microsoft 365 doc",
            Self::StencilaSites => "Stencila Site route",
        }
    }

    /// Get the plural display name for user-facing messages
    pub fn display_name_plural(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Docs",
            Self::Microsoft365 => "Microsoft 365",
            Self::StencilaSites => "Stencila Sites",
        }
    }

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
            Self::StencilaSites => {
                if let Some(host) = url.host_str() {
                    host.ends_with(".stencila.site")
                } else {
                    false
                }
            }
        }
    }

    /// Get the remote service from a URL
    pub fn from_url(url: &Url) -> Option<Self> {
        [Self::GoogleDocs, Self::Microsoft365, Self::StencilaSites]
            .iter()
            .find(|service| service.matches_url(url))
            .copied()
    }

    /// Get the format used by this remote service for pull/push operations
    pub fn pull_format(&self) -> Format {
        match self {
            Self::GoogleDocs => Format::Docx,
            Self::Microsoft365 => Format::Docx,
            Self::StencilaSites => Format::JsonLd,
        }
    }

    /// Push a document to this remote service
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
            Self::StencilaSites => stencila_codec_site::push(node, path, title, url, dry_run).await,
        }
    }

    /// Pull a document from this remote service
    ///
    /// Downloads the document and saves it to the specified path.
    pub async fn pull(&self, url: &Url, dest: &Path) -> Result<()> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::pull(url, dest).await,
            Self::Microsoft365 => stencila_codec_m365::pull(url, dest).await,
            Self::StencilaSites => stencila_codec_site::pull(url, dest).await,
        }
    }

    /// Time that a remote was last modified as a Unix timestamp
    pub async fn modified_at(&self, url: &Url) -> Result<u64> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::modified_at(url).await,
            Self::Microsoft365 => stencila_codec_m365::modified_at(url).await,
            Self::StencilaSites => stencila_codec_site::modified_at(url).await,
        }
    }
}
