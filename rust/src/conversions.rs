use crate::graphs::{resources, Relation, Triple};
use defaults::Defaults;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::{Path, PathBuf};

/// The definition of a conversion between files within a project
#[skip_serializing_none]
#[derive(PartialEq, Clone, Debug, Defaults, JsonSchema, Deserialize, Serialize)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct Conversion {
    /// The path of the input document
    input: PathBuf,

    /// The path of the output document
    output: PathBuf,

    /// The format of the input (defaults to being inferred from the file extension of the input)
    from: Option<String>,

    /// The format of the output (defaults to being inferred from the file extension of the output)
    to: Option<String>,

    /// Whether or not the conversion is active
    #[def = "true"]
    active: bool,
}

impl Conversion {
    /// Generate a graph triple describing relation between the input and output files.
    pub fn triple(&self, project: &Path) -> Triple {
        (
            resources::file(&project.join(&self.input)),
            Relation::Converts(self.active),
            resources::file(&project.join(&self.output)),
        )
    }
}
