use super::KernelTrait;
use eyre::{bail, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;
use stencila_schema::Node;

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
pub struct CalcKernel {
    #[serde(skip)]
    variables: HashMap<String, f64>,
}

impl KernelTrait for CalcKernel {
    fn get(&mut self, name: &str) -> Result<Node> {
        match self.variables.get(name) {
            Some(number) => Ok(Node::Number(*number)),
            None => bail!("Variable does not exist in kernel: {}", name),
        }
    }

    fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let value = match value {
            Node::Number(number) => number,
            _ => bail!("Unable to convert node to a number"),
        };
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    fn exec(&mut self, _code: &str) -> Result<Vec<Node>> {
        //let lines = code.lines()
        Ok(Vec::new())
    }
}
