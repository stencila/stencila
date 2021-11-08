use serde::{Deserialize, Serialize};

// Export and re-export for the convenience of crates that implement a provider
pub use eyre;

/// A specification for providers
///
/// All providers, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Provider` instance from the
/// `spec` function of `ProviderTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Provider {}

/// A trait for providers
///
/// This trait can be used by Rust implementations of providers, allowing them to
/// be compiled into the Stencila binaries.
///
/// It defines similar functions to `serde_json` (and other `serde_` crates) for
/// converting nodes to/from strings, files, readers etc.
pub trait ProviderTrait {
    /// Get the [`Provider`] specification
    fn spec() -> Provider;
}
