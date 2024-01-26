use std::path::Path;

use common::{async_trait::async_trait, eyre::Result, strum::Display};
use format::Format;

// Re-exports for the convenience of internal crates implementing
// the `Kernel` trait
pub use common;
pub use format;
pub use schema;
use schema::{ExecutionError, Node, Variable};

/// A kernel for executing code in some language
///
/// Provides a common, shared interface for the various execution kernels
/// including those that use embedded languages (e.g. Rhai, Lua), those that
/// connect to databases to execute SQL (e.g. SQLite, Postgres, DuckDB),
/// Stencila 'microkernels', and Jupyter kernels.
///
/// This trait specifies the kernel and its capabilities (similar to a Jupyter "kernel spec")
/// The `KernelInstance` trait is the interface for instances of kernels.
pub trait Kernel: Sync + Send {
    /// Get the id of the kernel
    ///
    /// This id should be unique amongst all kernels.
    fn id(&self) -> String;

    /// Get the availability of the kernel on the current machine
    fn availability(&self) -> KernelAvailability;

    /// Get the languages supported by the kernel
    fn supports_languages(&self) -> Vec<Format>;

    /// Does the kernel support forking?
    fn supports_forking(&self) -> KernelForking;

    /// Create a new instance of the kernel
    fn create_instance(&self) -> Result<Box<dyn KernelInstance>>;
}

/// The availability of a kernel on the current machine
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelAvailability {
    /// Available on this machine
    Available,
    /// Available on this machine but requires installation
    Installable,
    /// Not available on this machine
    Unavailable,
}

/// Whether a kernel supports forking on the current machine
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelForking {
    /// Kernel supports forking on this machine
    Yes,
    /// Kernel does not support forking on this machine
    No,
}

/// An instance of a kernel
#[async_trait]
pub trait KernelInstance: Sync + Send {
    /// Get the id of the kernel instance
    ///
    /// This id should be unique amongst all kernel instances,
    /// including those for other `Kernel`s.
    fn id(&self) -> String;

    /// Get the status of the kernel
    async fn status(&self) -> Result<KernelStatus>;

    /// Start the kernel in a working directory
    async fn start(&mut self, directory: &Path) -> Result<()>;

    /// Start the kernel in the current working directory
    async fn start_here(&mut self) -> Result<()> {
        self.start(&std::env::current_dir()?).await
    }

    /// Stop the kernel
    async fn stop(&mut self) -> Result<()>;

    /// Execute some code, possibly with side effects, in the kernel instance
    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)>;

    /// Evaluate a code expression, without side effects, in the kernel instance
    async fn evaluate(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)>;

    /// Execute some code in a fork of the kernel instance
    async fn fork(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)>;

    /// Get a list of variables in the kernel instance
    async fn list(&mut self) -> Result<Vec<Variable>>;

    /// Get a variable from the kernel instance
    async fn get(&mut self, name: &str) -> Result<Option<Node>>;

    /// Set a variable in the kernel instance
    async fn set(&mut self, name: &str, value: &Node) -> Result<()>;
}

/// The status of a kernel instance
#[derive(Default, Clone, Copy, Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelStatus {
    #[default]
    Pending,
    Starting,
    Ready,
    Busy,
    Stopped,
    Failed,
}
