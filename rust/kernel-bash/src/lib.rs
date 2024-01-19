use kernel_micro::{
    format::Format, Kernel, KernelAvailability, KernelForking, MicroKernel,
};

/// A kernel for executing Bash code locally
pub struct BashKernel {}

impl BashKernel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Kernel for BashKernel {
    fn id(&self) -> String {
        "bash-micro".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        MicroKernel::availability(self)
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Bash, Format::Shell]
    }

    fn supports_forking(&self) -> KernelForking {
        KernelForking::No
    }
}

impl MicroKernel for BashKernel {
    fn executable_name(&self) -> String {
        "bash".to_string()
    }
}
