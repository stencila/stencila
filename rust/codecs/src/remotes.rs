use clap::ValueEnum;
use url::Url;

use stencila_codec::{
    eyre::Result,
    stencila_schema::Node,
};

/// Remote document services
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RemoteService {
    /// Google Docs
    #[value(name = "gdocs")]
    GoogleDocs,
}

impl RemoteService {
    /// Check if a URL matches this remote service
    pub fn matches_url(&self, url: &Url) -> bool {
        match self {
            Self::GoogleDocs => url.host_str() == Some("docs.google.com"),
        }
    }

    /// Get the remote service from a URL
    pub fn from_url(url: &Url) -> Option<Self> {
        [Self::GoogleDocs]
            .iter()
            .find(|service| service.matches_url(url))
            .copied()
    }

    /// Get the display name for user-facing messages
    pub fn display_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Doc",
        }
    }

    /// Get the plural display name for user-facing messages
    pub fn display_name_plural(&self) -> &str {
        match self {
            Self::GoogleDocs => "Google Docs",
        }
    }

    /// Get the CLI value name (e.g., "gdocs")
    pub fn cli_name(&self) -> &str {
        match self {
            Self::GoogleDocs => "gdocs",
        }
    }

    /// Push a document to this remote service
    pub async fn push(
        &self,
        node: &Node,
        title: &str,
        url: Option<&Url>,
    ) -> Result<Url> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::push(node, title, url).await,
        }
    }

    /// Pull a document from this remote service
    ///
    /// Downloads the document and returns it as a Node.
    pub async fn pull(&self, url: &Url) -> Result<Node> {
        match self {
            Self::GoogleDocs => stencila_codec_gdoc::pull(url).await,
        }
    }
}
