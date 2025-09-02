//! Handling of custom requests and notifications related to kernels

use async_lsp::lsp_types::request::Request;

use stencila_kernels::KernelSpecification;

pub struct ListKernels;

impl Request for ListKernels {
    const METHOD: &'static str = "stencila/listKernels";
    type Params = ();
    type Result = Vec<KernelSpecification>;
}

pub async fn list() -> Vec<KernelSpecification> {
    stencila_kernels::list()
        .await
        .into_iter()
        .map(|kernel| KernelSpecification::from(kernel.as_ref()))
        .collect()
}
