//! Handling of custom requests and notifications related to models

use async_lsp::lsp_types::request::Request;

use common::serde::{Deserialize, Serialize};
use models::{ModelAvailability, ModelType};

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Model {
    id: String,
    provider: String,
    name: String,
    version: String,
    r#type: ModelType,
    availability: ModelAvailability,
}

pub struct ListModels;

impl Request for ListModels {
    const METHOD: &'static str = "stencila/listModels";
    type Params = ();
    type Result = Vec<Model>;
}

pub async fn list() -> Vec<Model> {
    models::list()
        .await
        .into_iter()
        .map(|model| Model {
            id: model.id(),
            provider: model.provider(),
            name: model.name(),
            version: model.version(),
            r#type: model.r#type(),
            availability: model.availability(),
        })
        .collect()
}
