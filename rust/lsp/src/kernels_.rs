//! Handling of custom requests and notifications related to kernels

use async_lsp::lsp_types::request::Request;

use common::serde::{Deserialize, Serialize};
use kernels::{KernelAvailability, KernelProvider, KernelType};

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Kernel {
    name: String,
    provider: KernelProvider,
    r#type: KernelType,
    availability: KernelAvailability,
}

pub struct ListKernels;

impl Request for ListKernels {
    const METHOD: &'static str = "stencila/listKernels";
    type Params = ();
    type Result = Vec<Kernel>;
}

pub async fn list() -> Vec<Kernel> {
    kernels::list()
        .await
        .into_iter()
        .map(|kernel| Kernel {
            name: kernel.name(),
            provider: kernel.provider(),
            r#type: kernel.r#type(),
            availability: kernel.availability(),
        })
        .collect()
}
