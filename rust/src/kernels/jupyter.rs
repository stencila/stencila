use super::KernelTrait;
use crate::errors::incompatible_language;
use eyre::{bail, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;
use stencila_schema::Node;

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct JupyterKernel {}

impl JupyterKernel {
    pub fn new() -> Self {
        JupyterKernel::default()
    }
}

impl KernelTrait for JupyterKernel {
    fn language(&self, language: Option<String>) -> Result<String> {
        todo!()
    }

    fn get(&self, name: &str) -> Result<Node> {
        todo!()
    }

    fn set(&mut self, name: &str, value: Node) -> Result<()> {
        todo!()
    }

    fn exec(&mut self, code: &str) -> Result<Vec<Node>> {
        todo!()
    }
}
