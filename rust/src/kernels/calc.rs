use crate::errors::incompatible_language;

use super::{Kernel, KernelTrait};
use async_trait::async_trait;
use eyre::{bail, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;
use stencila_schema::Node;

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct CalcKernel {
    #[serde(skip)]
    symbols: BTreeMap<String, f64>,
}

impl CalcKernel {
    pub fn create() -> Kernel {
        Kernel::Calc(CalcKernel::default())
    }
}

#[async_trait]
impl KernelTrait for CalcKernel {
    fn language(&self, language: Option<String>) -> Result<String> {
        let canonical = Ok("calc".to_string());
        match language.as_deref() {
            Some("calc") => canonical,
            Some(language) => bail!(incompatible_language::<Self>(language)),
            None => canonical,
        }
    }

    async fn get(&self, name: &str) -> Result<Node> {
        match self.symbols.get(name) {
            Some(number) => Ok(Node::Number(*number)),
            None => bail!("Symbol `{}` does not exist in this kernel", name),
        }
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let value = match value {
            Node::Number(number) => number,
            _ => bail!("Unable to convert node to a number"),
        };
        self.symbols.insert(name.to_string(), value);
        Ok(())
    }

    async fn exec(&mut self, code: &str) -> Result<Vec<Node>> {
        static STATEMENTS_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\r?\n|;").expect("Unable to create regex"));
        static ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\s*([a-zA-Z_][a-zA-Z_0-9]*)\s*=(.*)").expect("Unable to create regex")
        });

        let mut outputs = Vec::new();
        for statement in STATEMENTS_REGEX.split(code) {
            let statement = statement.trim();

            // Skip the statement if it is blank or is a comment
            if statement.is_empty() || statement.starts_with('#') {
                continue;
            }

            // Get the name of any assigned variable, and the expression to be evaluated
            let (symbol, expr) = if let Some(captures) = ASSIGN_REGEX.captures(statement) {
                let symbol = captures.get(1).expect("Should always have group 1");
                let expr = captures.get(2).expect("Should always have group 2");
                (Some(symbol.as_str()), expr.as_str())
            } else {
                (None, statement)
            };

            // Evaluate the expression, and assign it or add it to outputs
            let num = fasteval::ez_eval(expr, &mut self.symbols)?;
            if let Some(symbol) = symbol {
                self.symbols.insert(symbol.to_string(), num);
            } else {
                outputs.push(Node::Number(num))
            }
        }
        Ok(outputs)
    }
}
