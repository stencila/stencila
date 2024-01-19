use kernel::common::async_trait::async_trait;

// Re-exports for the convenience of internal crates implementing
// the `MicroKernel` trait
pub use kernel::{common, Kernel};

/// A minimal, lightweight execution kernel in a spawned process
#[async_trait]
pub trait MicroKernel: Sync + Send {}
