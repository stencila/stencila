use std::sync::Arc;

use kernel::Kernel;
use kernel_bash::BashKernel;

pub mod cli;

/// Get a list of available kernels
pub async fn list() -> Vec<Arc<dyn Kernel>> {
    vec![Arc::new(BashKernel::new()) as Arc<dyn Kernel>]
}
