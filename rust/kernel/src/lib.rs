// use crate::utils::uuids;
use async_trait::async_trait;
use eyre::Result;
use serde::{Deserialize, Serialize};
use stencila_schema::{CodeError, Node};
use strum::Display;

// Re-export for the convenience of crates that implement `KernelTrait`
pub use ::async_trait;
pub use eyre;
pub use serde;
pub use stencila_schema;

/// A specification for kernels
///
/// All kernels, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Kernel` instance from the
/// `spec` function of `KernelTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kernel {
    /// The name of the kernel
    ///
    /// This is used for informational purposes and to allow the user to specify
    /// which kernel they want to use (e.g. in instances that they have more than one kernel that
    /// is capable of executing a language).
    pub name: String,

    /// The type of kernel
    pub r#type: KernelType,

    /// The languages supported by the kernel
    ///
    /// These should be the `name` of one of the `Format`s defined in
    /// the `formats` crate. Many kernels only support one language.
    pub languages: Vec<String>,
}

impl Kernel {
    pub fn new(name: &str, r#type: KernelType, languages: &[&str]) -> Self {
        let languages = languages
            .iter()
            .map(|language| language.to_string())
            .collect();
        Self {
            name: name.to_string(),
            r#type,
            languages,
        }
    }

    pub fn matches(&self, selector: &str) -> bool {
        selector == self.name.to_lowercase() || self.languages.contains(&selector.to_string())
    }
}

/// The type of kernel
///
/// At present this is mainly for informational purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelType {
    Builtin,
    Micro,
    Jupyter,
}

/// The status of a running kernel
#[derive(Debug, PartialEq, Clone, Serialize, Display)]
#[allow(dead_code)]
pub enum KernelStatus {
    Pending,
    Starting,
    Idle,
    Busy,
    Unresponsive,
    Stopping,
    Finished,
    Failed,
    Unknown,
}

/// A trait for kernels
///
/// This trait can be used by Rust implementations of kernels, allowing them to
/// be compiled into the Stencila binaries.
#[async_trait]
pub trait KernelTrait {
    /// Get the [`Kernel`] specification for this implementation
    fn spec(&self) -> Kernel;

    /// Start the kernel
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Stop the kernel
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the status of the kernel
    async fn status(&self) -> Result<KernelStatus>;

    /// Get a symbol from the kernel
    async fn get(&mut self, name: &str) -> Result<Node>;

    /// Set a symbol in the kernel
    async fn set(&mut self, name: &str, value: Node) -> Result<()>;

    /// Execute some code in the kernel
    async fn exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)>;
}
