use std::collections::HashMap;

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        regex::Captures,
        serde::Serialize,
    },
    formats::Format,
    stencila_schema::{CodeError, Node},
    Kernel, KernelStatus, KernelTrait, KernelType, TagMap, Task, TaskResult,
};
use parser_tailwind::VAR_REGEX;

/**
 * A kernel that performs variable interpolation for Tailwind expressions
 *
 * This is a very simple kernel. The parsing and transpiling to CSS of Tailwind is handled
 * by the `parser-tailwind` crate. This crate simply interpolates variables (residing in
 * other kernels) into Tailwind expressions.
 */
#[derive(Debug, Clone, Default, Serialize)]
#[serde(crate = "kernel::common::serde")]
pub struct TailwindKernel {
    symbols: HashMap<String, String>,
}

impl TailwindKernel {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KernelTrait for TailwindKernel {
    async fn spec(&self) -> Kernel {
        Kernel::new(
            "style",
            KernelType::Builtin,
            &[Format::Tailwind],
            true,
            false,
            true,
        )
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Idle)
    }

    async fn get(&mut self, _name: &str) -> Result<Node> {
        bail!("Method `get()` is not implemented for Tailwind kernel")
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let value = match value {
            Node::Null(..) => "none".to_string(),
            Node::Boolean(boolean) => match boolean {
                true => "1".to_string(),
                false => "none".to_string(),
            },
            Node::Integer(integer) => integer.to_string(),
            Node::Number(number) => number.0.to_string(),
            Node::String(string) => string,
            _ => bail!("Node is of type that can not be converted to a CSS string"),
        };

        self.symbols.insert(name.to_string(), value);

        Ok(())
    }

    async fn derive(&mut self, _what: &str, _from: &str) -> Result<Vec<Node>> {
        bail!("Method `derive()` is not implemented for Tailwind kernel")
    }

    async fn exec_sync(
        &mut self,
        code: &str,
        lang: Format,
        _tags: Option<&TagMap>,
    ) -> Result<Task> {
        if lang != Format::Tailwind {
            bail!("Unexpected language for Tailwind kernel: {}", lang);
        }

        let mut task = Task::begin_sync();

        let mut messages = Vec::new();
        let interpolated = VAR_REGEX.replace_all(code, |captures: &Captures| {
            let symbol = captures
                .get(1)
                .or_else(|| captures.get(2))
                .expect("Should always have one group")
                .as_str();
            match self.symbols.get(symbol) {
                Some(value) => value.to_owned(),
                None => {
                    messages.push(CodeError {
                        error_type: Some(Box::new("UnknownSymbol".to_string())),
                        error_message: format!(
                            "Symbol `{}` is not known to the Tailwind kernel",
                            symbol
                        ),
                        ..Default::default()
                    });
                    captures[0].to_string()
                }
            }
        });

        let outputs = vec![Node::String(interpolated.to_string())];
        task.end(TaskResult::new(outputs, messages));

        Ok(task)
    }

    async fn exec_fork(
        &mut self,
        code: &str,
        lang: Format,
        _tags: Option<&TagMap>,
    ) -> Result<Task> {
        // Given that there are never any side effects of executing code there
        // is no need to create a clone, just run in current kernel
        self.exec_async(code, lang, _tags).await
    }
}
