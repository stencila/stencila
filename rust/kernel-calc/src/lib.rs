use std::collections::BTreeMap;

use fasteval::{ez_eval, Error};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        once_cell::sync::Lazy,
        regex::Regex,
        serde::Serialize,
    },
    formats::Format,
    stencila_schema::{
        CodeError, Node, Number, NumberValidator, Parameter, Primitive, ValidatorTypes,
    },
    Kernel, KernelStatus, KernelTrait, KernelType, TagMap, Task, TaskResult,
};

/// A kernel that evaluates simple calculator like numerical expressions
///
/// Based on [`fasteval`](https://github.com/likebike/fasteval). See the
/// `fasteval` docs for more on the syntax and functions supported.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(crate = "kernel::common::serde")]
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
    async fn spec(&self) -> Kernel {
        Kernel::new(
            "calc",
            KernelType::Builtin,
            &[Format::Calc],
            true,
            false,
            true,
        )
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Ready)
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        match self.symbols.get(name) {
            Some(number) => Ok(Node::Number(Number(*number))),
            None => bail!("Symbol `{}` does not exist in this kernel", name),
        }
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let value = match value {
            Node::Null(..) => 0.,
            Node::Boolean(boolean) => match boolean {
                true => 1.,
                false => 0.,
            },
            Node::Integer(integer) => integer as f64,
            Node::Number(number) => number.0,
            Node::String(string) => match string.trim().parse() {
                Ok(number) => number,
                Err(..) => bail!("Unable to convert string `{}` to a number", string),
            },
            Node::Array(array) => match array.first() {
                Some(value) => match value {
                    Primitive::Null(..) => 0.,
                    Primitive::Boolean(boolean) => match boolean {
                        true => 1.,
                        false => 0.,
                    },
                    Primitive::Integer(integer) => *integer as f64,
                    Primitive::Number(number) => number.0,
                    Primitive::String(string) => match string.trim().parse() {
                        Ok(number) => number,
                        Err(..) => bail!("Unable to convert string `{}` to a number", string),
                    },
                    _ => bail!("Unable to convert first item of array to a number"),
                },
                _ => bail!("Unable to convert empty array to a number"),
            },
            _ => bail!("Node is of type that can not be converted to a number"),
        };
        self.symbols.insert(name.to_string(), value);
        Ok(())
    }

    async fn derive(&mut self, what: &str, from: &str) -> Result<Vec<Node>> {
        if what != "parameter" {
            bail!("Only know how to derive a single parameter from a `calc` kernel")
        }

        match self.symbols.get(from) {
            Some(..) => Ok(vec![Node::Parameter(Parameter {
                name: from.to_string(),
                validator: Some(Box::new(ValidatorTypes::NumberValidator(
                    NumberValidator::default(),
                ))),
                ..Default::default()
            })]),
            None => bail!("Symbol `{}` does not exist in this `calc` kernel", from),
        }
    }

    async fn exec_sync(
        &mut self,
        code: &str,
        lang: Format,
        _tags: Option<&TagMap>,
    ) -> Result<Task> {
        if lang != Format::Calc {
            bail!("Unexpected language for `CalcKernel`: {}", lang);
        }

        static STATEMENTS_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\r?\n|;").expect("Unable to create regex"));
        static ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\s*([a-zA-Z_][a-zA-Z_0-9]*)\s*=(.*)").expect("Unable to create regex")
        });

        let mut task = Task::begin_sync();
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

            // A fasteval callback function that defines some custom functions
            let mut cb = |name: &str, _args: Vec<f64>| -> Option<f64> {
                match name {
                    "now" => now(),
                    _ => self.symbols.get(name).copied(),
                }
            };

            // Evaluate the expression
            match ez_eval(expr, &mut cb) {
                Ok(num) => {
                    // Either assign the result, or add it to outputs
                    if let Some(symbol) = symbol {
                        self.symbols.insert(symbol.to_string(), num);
                    } else {
                        outputs.push(Node::Number(Number(num)))
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
        task.end(TaskResult::new(outputs, messages));
        Ok(task)
    }

    async fn exec_fork(
        &mut self,
        code: &str,
        lang: Format,
        _tags: Option<&TagMap>,
    ) -> Result<Task> {
        let mut fork = self.clone();
        fork.exec_async(code, lang, _tags).await
    }
}

// Custom functions

/// The current system time as seconds (to millisecond resolution) since the Unix epoch
fn now() -> Option<f64> {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis() as f64 / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{common::tokio, stencila_schema::Number, KernelStatus, KernelTrait};
    use test_utils::{assert_json_eq, common::serde_json::json};

    #[tokio::test]
    async fn status() -> Result<()> {
        let kernel = CalcKernel::new();

        assert_eq!(kernel.status().await?, KernelStatus::Ready);

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
                .contains("Unable to convert string `A` to a number")),
        };

        kernel.set("a", Node::Number(Number(1.23))).await?;

        let a = kernel.get("a").await?;
        assert!(matches!(a, Node::Number(..)));
        assert_json_eq!(a, json!(1.23));

        let (outputs, errors) = kernel.exec("a * 2", Format::Calc, None).await?;
        assert_json_eq!(outputs, json!([2.46]));
        assert_eq!(errors.len(), 0);

        let (outputs, errors) = kernel.exec("x * 2", Format::Calc, None).await?;
        assert_eq!(outputs.len(), 0);
        assert_json_eq!(
            errors,
            json!([{"type": "CodeError", "errorMessage": "Undefined variable or function: x"}])
        );

        Ok(())
    }
}
