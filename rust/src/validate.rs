use anyhow::{bail, Result};
use jsonschema::JSONSchema;
use serde_json::{json, Value};

use crate::nodes::Node;

lazy_static! {
    static ref SCHEMA: Value = json!({
        "maxLength": 5, "pattern": "aaa",
    });
    // TODO cache compiled schemas in a LRU cache
    static ref VALIDATOR: JSONSchema<'static> = JSONSchema::compile(&SCHEMA).unwrap();
}

pub fn validate(node: Node) -> Result<Node> {
    let result = VALIDATOR.validate(&node);
    match result {
        Ok(_) => Ok(Node::Bool(true)),
        Err(errors) => {
            let message = errors
                .map(|error| error.to_string())
                .collect::<Vec<String>>()
                .join("; ");
            bail!("{}", message)
        }
    }
}

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub node: Node,
    }

    pub fn validate(params: Params) -> Result<Node> {
        let Params { node } = params;
        super::validate(node)
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use crate::decode::decode;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Validate", // TODO about
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub struct Args {
        /// TODO docs
        input: String,

        /// TODO docs
        #[structopt(short, long)]
        format: Option<String>,
    }

    pub fn validate(args: Args) -> Result<()> {
        let Args { input, format } = args;

        let node = decode(input, format.unwrap_or_default())?;

        super::validate(node)?;

        Ok(())
    }
}
