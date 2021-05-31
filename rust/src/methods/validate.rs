use crate::nodes::Node;
use eyre::{bail, Result};
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde_json::{json, Value};

pub fn validate(node: Node) -> Result<Node> {
    static SCHEMA: Lazy<Value> = Lazy::new(|| json!({ "maxLength": 5, "pattern": "aaa" }));
    static VALIDATOR: Lazy<JSONSchema<'static>> =
        Lazy::new(|| JSONSchema::compile(&SCHEMA).unwrap());

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
