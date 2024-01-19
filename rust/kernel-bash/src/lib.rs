use kernel_micro::{Kernel, MicroKernel};

/// A kernel for executing Bash code locally
pub struct BashKernel {}

impl Kernel for BashKernel {}

impl MicroKernel for BashKernel {}
