use kernel::Kernel;
use kernel_bash::BashKernel;
use kernel_node::NodeKernel;

pub mod cli;

/// Get a list of available kernels
pub async fn list() -> Vec<Box<dyn Kernel>> {
    vec![
        Box::new(BashKernel::default()) as Box<dyn Kernel>,
        Box::new(NodeKernel::default()) as Box<dyn Kernel>,
    ]
}
