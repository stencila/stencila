use fasteval::{ez_eval, Error};
use kernel::{
    async_trait::async_trait,
    eyre::{bail, Result},
    serde::Serialize,
    stencila_schema::{CodeError, Node},
    Kernel, KernelStatus, KernelTrait, KernelType, Task, TaskResult,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeMap;

/// A kernel that evaluates simple calculator like numerical expressions
///
/// Based on [`fasteval`](https://github.com/likebike/fasteval). See the
/// `fasteval` docs for more on the syntax and functions supported.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(crate = "kernel::serde")]
pub struct CalcKernel {
    symbols: BTreeMap<String, f64>,
}

impl CalcKernel {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KernelTrait for CalcKernel {
    fn spec(&self) -> Kernel {
        Kernel::new("calc", KernelType::Builtin, &["calc"])
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Idle)
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        match self.symbols.get(name) {
            Some(number) => Ok(Node::Number(*number)),
            None => bail!("Symbol `{}` does not exist in this kernel", name),
        }
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let value = match value {
            Node::Integer(integer) => integer as f64,
            Node::Number(number) => number,
            _ => bail!("Unable to convert node to a number"),
        };
        self.symbols.insert(name.to_string(), value);
        Ok(())
    }

    async fn exec_sync(&mut self, code: &str) -> Result<Task> {
        static STATEMENTS_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\r?\n|;").expect("Unable to create regex"));
        static ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\s*([a-zA-Z_][a-zA-Z_0-9]*)\s*=(.*)").expect("Unable to create regex")
        });

        let mut task = Task::start_sync();
        let mut outputs = Vec::new();
        let mut messages = Vec::new();
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

            // Evaluate the expression
            match ez_eval(expr, &mut self.symbols) {
                Ok(num) => {
                    // Either assign the result, or add it to outputs
                    if let Some(symbol) = symbol {
                        self.symbols.insert(symbol.to_string(), num);
                    } else {
                        outputs.push(Node::Number(num))
                    }
                }
                Err(error) => {
                    let error_message = match error {
                        // Custom error strings for common errors
                        Error::EOF | Error::EofWhileParsing(..) => {
                            "Unexpected end of Calc expression".to_string()
                        }
                        Error::Undefined(name) => {
                            format!("Undefined variable or function: {}", name)
                        }
                        Error::WrongArgs(msg) => {
                            format!("Function called with wrong number of arguments: {}", msg)
                        }
                        Error::InvalidValue => "Unexpected value in expression".to_string(),
                        Error::TooLong => "Calc expression was too long".to_string(),
                        Error::TooDeep => "Calc expression was too recursive".to_string(),
                        // Use the debug string for others
                        _ => format!("Could not execute Calc expression: {:?}", error),
                    };
                    messages.push(CodeError {
                        error_message,
                        ..Default::default()
                    });
                }
            }
        }
        task.finished(TaskResult::new(outputs, messages));
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{KernelStatus, KernelTrait};
    use test_utils::{assert_json_eq, serde_json::json};

    #[tokio::test]
    async fn status() -> Result<()> {
        let kernel = CalcKernel::new();

        assert_eq!(kernel.status().await?, KernelStatus::Idle);

        Ok(())
    }

    #[tokio::test]
    async fn get_set_exec() -> Result<()> {
        let mut kernel = CalcKernel::new();

        match kernel.get("a").await {
            Ok(..) => bail!("Expected an error"),
            Err(error) => assert!(error.to_string().contains("does not exist")),
        };

        match kernel.set("a", Node::String("A".to_string())).await {
            Ok(..) => bail!("Expected an error"),
            Err(error) => assert!(error
                .to_string()
                .contains("Unable to convert node to a number")),
        };

        kernel.set("a", Node::Number(1.23)).await?;

        let a = kernel.get("a").await?;
        assert!(matches!(a, Node::Number(..)));
        assert_json_eq!(a, json!(1.23));

        let (outputs, errors) = kernel.exec("a * 2").await?;
        assert_json_eq!(outputs, json!([2.46]));
        assert_eq!(errors.len(), 0);

        let (outputs, errors) = kernel.exec("x * 2").await?;
        assert_eq!(outputs.len(), 0);
        assert_json_eq!(
            errors,
            json!([{"type": "CodeError", "errorMessage": "Undefined variable or function: x"}])
        );

        Ok(())
    }
}
