use kernel::common::async_trait::async_trait;
use which::which;

// Re-exports for the convenience of internal crates implementing
// the `MicroKernel` trait
pub use kernel::{common, format, Kernel, KernelAvailability, KernelForking};

/// A minimal, lightweight execution kernel in a spawned process
#[async_trait]
pub trait MicroKernel: Sync + Send {
    /// Get the name of the executable (e.g. `python`) used by this microkernel
    fn executable_name(&self) -> String;

    /// Whether the executable used by this microkernel is available on this machine
    ///
    /// Returns `true` if an executable with `executable_name()` is in the `PATH`,
    /// and `false` otherwise.
    fn executable_available(&self) -> bool {
        which(&self.executable_name()).is_ok()
    }

    /// An implementation of `Kernel::availability` for microkernels
    ///
    /// Returns `Available` if the microkernel's executable is available
    /// of this machine. Otherwise returns `Installable` to indicate that
    /// it could be available if installed.
    fn availability(&self) -> KernelAvailability {
        if self.executable_available() {
            KernelAvailability::Available
        } else {
            KernelAvailability::Installable
        }
    }
}
