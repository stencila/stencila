use std::path::Path;

use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    strum::Display,
};
use format::Format;

// Re-exports for the convenience of internal crates implementing
// the `Kernel` trait
pub use common;
pub use format;
pub use schema;
use schema::{ExecutionError, Node};

/// A kernel for executing code in some language
///
/// Provides a common, shared interface for the various execution kernels
/// including those that use embedded languages (e.g. Rhai, Lua), those that
/// connect to databases to execute SQL (e.g. SQLite, Postgres, DuckDB),
/// Stencila 'microkernels', and Jupyter kernels.
#[allow(unused)]
#[async_trait]
pub trait Kernel: Sync + Send {
    /// Get the id of the kernel
    ///
    /// This id should be unique amongst kernels
    fn id(&self) -> String;

    /// Get the availability of the kernel on the current machine
    fn availability(&self) -> KernelAvailability;

    /// Get the languages supported by the kernel
    fn supports_languages(&self) -> Vec<Format>;

    /// Does the kernel support evaluation of code expression with no side effects possible?
    fn supports_evaluation(&self) -> KernelEvaluation;

    /// Does the kernel support forking?
    fn supports_forking(&self) -> KernelForking;

    /// Start the kernel in a working directory
    async fn start(&mut self, directory: &Path) -> Result<()>;

    /// Start the kernel in the current working directory
    async fn start_here(&mut self) -> Result<()> {
        self.start(&std::env::current_dir()?).await
    }

    /// Stop the kernel
    async fn stop(&mut self) -> Result<()>;

    /// Execute some code, possibly with side effects, in the kernel
    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)>;

    /// Evaluate a code expression, without side effects, in the kernel
    async fn evaluate(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        bail!(
            "Kernel `{id}` does not support evaluation of code",
            id = self.id()
        )
    }

    /// Execute some code in a fork of the kernel
    async fn fork(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        bail!("Kernel `{id}` does not support forking", id = self.id());
    }
}

/// The status of a kernel instance
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelStatus {
    Started,
    Ready,
    Busy,
    Stopped,
    Failed,
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

/// Whether a kernel supports evaluation of code expressions without side effects
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelEvaluation {
    /// Kernel supports evaluation of code expressions
    Yes,
    /// Kernel does not support evaluation of code expressions
    No,
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
