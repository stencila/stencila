use schemars::JsonSchema;
/// https://en.wikipedia.org/wiki/Semantic_triple
use serde::{Deserialize, Serialize};

pub type Address = String;

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub enum Thing {
    File(String),
    CodeChunk(String),
    CodeExpression(String),
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Relation {
    ImportsFile,

    LinksFile,
    LinksUrl,
    IncludesFile,

    UsesModule,
    ReadsFile,
    WritesFile,
}
