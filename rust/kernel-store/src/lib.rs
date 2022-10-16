use std::collections::HashMap;

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        json5,
        serde::Serialize,
        serde_json,
    },
    formats::Format,
    stencila_schema::{CodeError, Node, Primitive},
    Kernel, KernelStatus, KernelTrait, TagMap, Task, TaskResult,
};
use node_transform::Transform;

/// A kernel that simply stores nodes
///
/// This kernel is used as the default place to store the value of document
/// parameters. The value of those parameters can then be mirrored to other
/// kernels.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(crate = "kernel::common::serde")]
pub struct StoreKernel {
    symbols: HashMap<String, Node>,
}

impl StoreKernel {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KernelTrait for StoreKernel {
    async fn spec(&self) -> Kernel {
        Kernel::new(
            "store",
            kernel::KernelType::Builtin,
            &[Format::Json, Format::Json5],
            true,
            false,
            false,
        )
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Idle)
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        match self.symbols.get(name) {
            Some(node) => Ok(node.clone()),
            None => bail!("Symbol `{}` does not exist in this kernel", name),
        }
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        self.symbols.insert(name.to_string(), value);
        Ok(())
    }

    async fn derive(&mut self, _what: &str, _from: &str) -> Result<Vec<Node>> {
        bail!("Derive is not implemented for `store` kernel");
    }

    async fn exec_sync(
        &mut self,
        code: &str,
        lang: Format,
        _tags: Option<&TagMap>,
    ) -> Result<Task> {
        let mut task = Task::begin_sync();
        let mut outputs = Vec::new();
        let mut messages = Vec::new();

        let result = match lang {
            Format::Json => serde_json::from_str::<Primitive>(code)
                .map(|primitive| primitive.to_node())
                .map_err(|error| eyre!(error)),
            Format::Json5 => json5::from_str::<Primitive>(code)
                .map(|primitive| primitive.to_node())
                .map_err(|error| eyre!(error)),
            _ => self.get(code.trim()).await,
        };

        match result {
            Ok(output) => outputs.push(output),
            Err(error) => messages.push(CodeError {
                error_message: error.to_string(),
                ..Default::default()
            }),
        }

        task.end(TaskResult::new(outputs, messages));
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use kernel::{common::tokio, stencila_schema::Number, KernelStatus, KernelTrait};

    use super::*;

    #[tokio::test]
    async fn status() -> Result<()> {
        let kernel = StoreKernel::new();

        assert_eq!(kernel.status().await?, KernelStatus::Idle);

        Ok(())
    }

    #[tokio::test]
    async fn get_set_exec() -> Result<()> {
        let mut kernel = StoreKernel::new();

        match kernel.get("a").await {
            Ok(..) => bail!("Expected an error"),
            Err(error) => assert!(error.to_string().contains("does not exist")),
        };

        kernel.set("a", Node::String("A".to_string())).await?;
        kernel.set("b", Node::Number(Number(1.23))).await?;

        let a = kernel.get("a").await?;
        assert!(matches!(a, Node::String(..)));

        let b = kernel.get("b").await?;
        assert!(matches!(b, Node::Number(..)));

        let (outputs, errors) = kernel.exec("[1,2,3]", Format::Json, None).await?;
        assert_eq!(outputs.len(), 1);
        assert_eq!(errors.len(), 0);

        let (outputs, errors) = kernel.exec("{a:1}", Format::Json5, None).await?;
        assert_eq!(outputs.len(), 1);
        assert_eq!(errors.len(), 0);

        let (outputs, errors) = kernel.exec("a", Format::Unknown, None).await?;
        assert_eq!(outputs.len(), 1);
        assert_eq!(errors.len(), 0);

        Ok(())
    }
}
