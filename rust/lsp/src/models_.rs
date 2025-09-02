//! Handling of custom requests and notifications related to models

use async_lsp::lsp_types::request::Request;

use stencila_models::ModelSpecification;

pub struct ListModels;

impl Request for ListModels {
    const METHOD: &'static str = "stencila/listModels";
    type Params = ();
    type Result = Vec<ModelSpecification>;
}

pub async fn list() -> Vec<ModelSpecification> {
    stencila_models::list()
        .await
        .into_iter()
        .map(|model| ModelSpecification::from(model.as_ref()))
        .collect()
}
