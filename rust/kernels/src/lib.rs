use std::sync::Arc;

use kernel::Kernel;
use kernel_bash::BashKernel;

pub mod cli;

/// Get a list of available kernels
pub async fn list() -> Vec<Box<dyn Kernel>> {
    vec![Box::new(BashKernel {}) as Box<dyn Kernel>]
}
