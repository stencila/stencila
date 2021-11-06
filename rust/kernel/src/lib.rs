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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Kernel {
    /// The language of the kernel
    ///
    /// This should be the `name` of one of the `Format`s defined in
    /// the `formats` crate.
    pub language: String,
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
}

/// A trait for kernels
///
/// This trait can be used by Rust implementations of kernels, allowing them to
/// be compiled into the Stencila binaries.
#[async_trait]
pub trait KernelTrait {
    /// Get the [`Kernel`] specification for this implementation
    fn spec() -> Kernel;

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
    async fn get(&self, name: &str) -> Result<Node>;

    /// Set a symbol in the kernel
    async fn set(&mut self, name: &str, value: Node) -> Result<()>;

    /// Execute some code in the kernel
    async fn exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)>;
}
