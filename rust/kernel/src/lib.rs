use common::async_trait::async_trait;

// Re-exports for the convenience of internal crates implementing
// the `Kernel` trait
pub use common;

/// A kernel for executing code in some language
///
/// Provides a common, shared interface for the various execution kernels
/// including those that use embedded languages (e.g. Rhai, Lua), those that
/// connect to databases to execute SQL (e.g. SQLite, Postgres, DuckDB),
/// Stencila 'microkernels', and Jupyter kernels.
#[async_trait]
pub trait Kernel: Sync + Send {}
