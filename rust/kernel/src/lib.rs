use std::path::Path;

use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    strum::Display,
    tokio::sync::{mpsc, watch},
};
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

    /// Does the kernel support the interrupt signal?
    fn supports_interrupt(&self) -> KernelInterrupt;

    /// Does the kernel support the terminate signal?
    fn supports_terminate(&self) -> KernelTerminate;

    /// Does the kernel support the kill signal?
    fn supports_kill(&self) -> KernelKill;

    /// Does the kernel support forking?
    fn supports_forks(&self) -> KernelForks;

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

/// Whether a kernel supports the interrupt signal on the current machine
///
/// The interrupt signal is used to stop the execution task the
/// kernel instance is current performing.
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelInterrupt {
    /// Kernel supports interrupt signal on this machine
    Yes,
    /// Kernel does not support interrupt signal on this machine
    No,
}

/// Whether a kernel supports the terminate signal on the current machine
///
/// The terminate signal is used to stop the kernel instance gracefully
/// (e.g. completing any current execution tasks)
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelTerminate {
    /// Kernel supports terminate signal on this machine
    Yes,
    /// Kernel does not support terminate signal on this machine
    No,
}

/// Whether a kernel supports the kill signal on the current machine
///
/// The kill signal is used to stop the kernel instance forcefully
/// (i.e. to exit immediately, aborting any current execution tasks)
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelKill {
    /// Kernel supports kill signal on this machine
    Yes,
    /// Kernel does not support kill signal on this machine
    No,
}

/// Whether a kernel supports forking on the current machine
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelForks {
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

    /// Get the status of the kernel instance
    async fn status(&self) -> Result<KernelStatus>;

    /// Get a watcher of the status of the kernel instance
    fn watcher(&self) -> Result<watch::Receiver<KernelStatus>>;

    /// Get a signaller to interrupt or kill the kernel instance
    fn signaller(&self) -> Result<mpsc::Sender<KernelSignal>>;

    /// Start the kernel in a working directory
    async fn start(&mut self, directory: &Path) -> Result<()>;

    /// Start the kernel in the current working directory
    async fn start_here(&mut self) -> Result<()> {
        self.start(&std::env::current_dir()?).await
    }

    /// Stop the kernel
    async fn stop(&mut self) -> Result<()>;

    /// Execute code, possibly with side effects, in the kernel instance
    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)>;

    /// Evaluate a code expression, without side effects, in the kernel instance
    async fn evaluate(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)>;

    /// Get a list of variables in the kernel instance
    async fn list(&mut self) -> Result<Vec<Variable>>;

    /// Get a variable from the kernel instance
    async fn get(&mut self, name: &str) -> Result<Option<Node>>;

    /// Set a variable in the kernel instance
    async fn set(&mut self, name: &str, value: &Node) -> Result<()>;

    /// Remove a variable from the kernel instance
    async fn remove(&mut self, name: &str) -> Result<()>;

    /// Create a fork of the kernel instance
    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        bail!("Kernel `{}` does not support forks", self.id())
    }
}

/// The status of a kernel instance
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelStatus {
    #[default]
    Pending,
    Starting,
    Ready,
    Busy,
    Stopping,
    Stopped,
    Failed,
}

impl From<KernelStatus> for u8 {
    fn from(status: KernelStatus) -> Self {
        status as u8
    }
}

impl From<u8> for KernelStatus {
    fn from(value: u8) -> Self {
        use KernelStatus::*;
        match value {
            0 => Pending,
            1 => Starting,
            2 => Ready,
            3 => Busy,
            4 => Stopping,
            5 => Stopped,
            6 => Failed,
            _ => Pending,
        }
    }
}

/// A signal to send to a kernel instance
#[derive(Clone, Copy)]
pub enum KernelSignal {
    Interrupt,
    Terminate,
    Kill,
}
