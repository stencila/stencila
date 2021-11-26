use crate::{
    config::{Config, ConfigEvent},
    documents::{Document, DocumentEvent},
    errors::Error,
    files::{File, FileEvent},
    patches::Patch,
    projects::{Project, ProjectEvent},
    sessions::{Session, SessionEvent},
};
use eyre::{bail, Result};
use formats::Format;
use schemars::schema_for;
use serde_json::json;

/// Get all the JSON Schema definitions for types in this crate (including those defined in sub-crates)
pub fn all() -> Vec<serde_json::Value> {
    let mut schemas = vec![
        json!(schema_for!(Config)),
        json!(schema_for!(ConfigEvent)),
        json!(schema_for!(Document)),
        json!(schema_for!(DocumentEvent)),
        json!(schema_for!(Error)),
        json!(schema_for!(File)),
        json!(schema_for!(FileEvent)),
        json!(schema_for!(Format)),
        json!(schema_for!(Patch)),
        json!(schema_for!(Project)),
        json!(schema_for!(ProjectEvent)),
        json!(schema_for!(Session)),
        json!(schema_for!(SessionEvent)),
    ];
    schemas.append(&mut graph_triples::schemas());
    schemas.append(&mut graph::schemas());
    schemas
}

/// Get the JSON Schema for a type
pub fn get(type_: &str) -> Result<serde_json::Value> {
    for schema in all() {
        if let Some(title) = schema
            .get("title")
            .and_then(|title| title.as_str())
            .map(|title| title.to_lowercase())
        {
            if title == type_.to_lowercase() {
                return Ok(schema);
            }
        }
    }
    bail!("No type with name matching `{}`", type_)
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
    use structopt::StructOpt;

    /// Get JSON Schema for internal application types
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Command {
        /// The type to generate a schema for (defaults to generating schemas for all types)
        #[structopt(name = "type")]
        pub type_: Option<String>,
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            if let Some(type_) = &self.type_ {
                result::value(get(type_)?)
            } else {
                result::value(all())
            }
        }
    }
}
