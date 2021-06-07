use eyre::{bail, Result};
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use stencila_schema::Node;

pub fn validate(node: Node) -> Result<Node> {
    // TODO Read the actual schema
    static SCHEMA: Lazy<Value> = Lazy::new(|| json!({}));
    static VALIDATOR: Lazy<JSONSchema<'static>> =
        Lazy::new(|| JSONSchema::compile(&SCHEMA).unwrap());

    let value = serde_json::to_value(node)?;
    let result = VALIDATOR.validate(&value);
    match result {
        Ok(_) => Ok(Node::Boolean(true)),
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
