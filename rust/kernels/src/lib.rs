use kernel::Kernel;
use kernel_bash::BashKernel;
use kernel_node::NodeKernel;
use kernel_rhai::RhaiKernel;

pub mod cli;

/// Get a list of available kernels
pub async fn list() -> Vec<Box<dyn Kernel>> {
    vec![
        Box::<BashKernel>::default() as Box<dyn Kernel>,
        Box::<NodeKernel>::default() as Box<dyn Kernel>,
        Box::<RhaiKernel>::default() as Box<dyn Kernel>,
    ]
}
