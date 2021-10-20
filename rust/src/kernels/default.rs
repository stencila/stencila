use super::{Kernel, KernelTrait};
use crate::errors::incompatible_language;
use async_trait::async_trait;
use eyre::{bail, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;
use stencila_schema::Node;

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct DefaultKernel {
    #[serde(skip)]
    symbols: HashMap<String, Node>,
}

impl DefaultKernel {
    pub fn create() -> Kernel {
        Kernel::Default(DefaultKernel::default())
    }
}

#[async_trait]
impl KernelTrait for DefaultKernel {
    fn language(&self, language: Option<String>) -> Result<String> {
        let canonical = Ok("none".to_string());
        match language.as_deref() {
            Some("") | Some("none") => canonical,
            Some(language) => bail!(incompatible_language::<Self>(language)),
            None => canonical,
        }
    }

    fn get(&self, name: &str) -> Result<Node> {
        match self.symbols.get(name) {
            Some(node) => Ok(node.clone()),
            None => bail!("Symbol `{}` does not exist in this kernel", name),
        }
    }

    fn set(&mut self, name: &str, value: Node) -> Result<()> {
        self.symbols.insert(name.to_string(), value);
        Ok(())
    }

    async fn exec(&mut self, code: &str) -> Result<Vec<Node>> {
        let mut outputs = Vec::new();
        for line in code.lines() {
            let node = self.get(line.trim())?;
            outputs.push(node)
        }
        Ok(outputs)
    }
}
