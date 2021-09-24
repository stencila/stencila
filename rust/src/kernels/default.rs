use super::KernelTrait;
use eyre::{bail, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;
use stencila_schema::Node;

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
pub struct DefaultKernel {
    #[serde(skip)]
    variables: HashMap<String, Node>,
}

impl DefaultKernel {
    pub fn new() -> Self {
        DefaultKernel::default()
    }
}

impl KernelTrait for DefaultKernel {
    fn get(&mut self, name: &str) -> Result<Node> {
        match self.variables.get(name) {
            Some(node) => Ok(node.clone()),
            None => bail!("Variable does not exist in kernel {}", name),
        }
    }

    fn set(&mut self, name: &str, value: Node) -> Result<()> {
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    fn exec(&mut self, code: &str) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        for line in code.lines() {
            let node = self.get(line.trim())?;
            nodes.push(node)
        }
        Ok(nodes)
    }
}
