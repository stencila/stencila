use eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use stencila_schema::Node;

// Export and re-export for the convenience of crates that implement a provider
pub use async_trait::async_trait;
pub use eyre;
pub use node_address::Address;
pub use stencila_schema;

/// A specification for providers
///
/// All providers, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Provider` instance from the
/// `spec` function of `ProviderTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Provider {
    /// The name of the provider
    pub name: String,
}

/// A trait for providers
///
/// This trait can be used by Rust implementations of providers, allowing them to
/// be compiled into the Stencila binaries.
#[async_trait]
pub trait ProviderTrait {
    /// Get the [`Provider`] specification
    fn spec() -> Provider;

    /// Detect nodes within a root node that the provider may be able to identify and enrich.
    ///
    /// Returns a vector of [`Detection`].
    async fn detect(_root: &Node) -> Result<Vec<ProviderDetection>> {
        bail!(
            "Detection is not implemented for provider `{}`",
            Self::spec().name
        )
    }

    /// Identify a node
    ///
    /// The node is supplied to the provider, with one or more properties populated.
    /// The provider then attempts to identify the node based on those properties,
    /// and if it was able to do so, returns a copy of the node with one or more identifying
    /// properties populated (e.g. the `GithubProvider` might populate the `codeRepository` property
    /// of a `SofwareSourceCode` node).
    async fn identify(_node: &Node) -> Result<Node> {
        bail!(
            "Identification is not implemented for provider `{}`",
            Self::spec().name
        )
    }

    /// Enrich a node
    ///
    /// If the provider had previously identified the node, then the relevant identifiers
    /// will be used to fetch enrichment data, otherwise `identify` will be called.
    /// Then, the provider will return a opy of the node with properties that are missing.
    async fn enrich(_node: &Node) -> Result<Node> {
        bail!(
            "Enrichment is not implemented for provider `{}`",
            Self::spec().name
        )
    }

    /// Import files associated with a node, from the provider, into a project
    async fn import(_node: &Node) -> Result<Vec<PathBuf>> {
        bail!(
            "Import is not implemented for provider `{}`",
            Self::spec().name
        )
    }
}

#[derive(Debug, Serialize)]
pub struct ProviderDetection {
    /// The name of the provider that detected the node
    pub provider: String,

    /// The percent confidence in the detection (0-100)
    pub confidence: u32,

    /// The [`Address`], within the node tree, that the node detected node begins
    pub begin: Address,

    /// The [`Address`], within the node tree, that the node detected node ends
    pub end: Address,

    /// The detected [`Node`] usually with some properties populated (i.e. those
    /// properties that were used to detect it)
    pub node: Node,
}
