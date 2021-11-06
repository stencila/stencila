use kernel::{
    async_trait::async_trait,
    eyre::{bail, Result},
    serde::Serialize,
    stencila_schema::{CodeError, Node, Null},
    Kernel, KernelStatus, KernelTrait,
};

//mod connection;
mod dirs;
//mod kernel;
//mod messages;
//mod server;

//pub use server::JupyterServer;

/// A kernel that delegates to a Jupyter kernel
#[derive(Debug, Clone, Default, Serialize)]
#[serde(crate = "kernel::serde")]
pub struct JupyterKernel {}

impl JupyterKernel {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KernelTrait for JupyterKernel {
    fn spec() -> Kernel {
        Kernel {
            language: "calc".to_string(),
        }
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Idle)
    }

    async fn get(&self, name: &str) -> Result<Node> {
        Ok(Node::Null(Null {}))
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        Ok(())
    }

    async fn exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)> {
        let mut outputs = Vec::new();
        let mut errors = Vec::new();
        Ok((outputs, errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{KernelStatus, KernelTrait};
    use test_utils::{assert_json_eq, serde_json::json};

    #[tokio::test]
    async fn status() -> Result<()> {
        let kernel = JupyterKernel::new();

        assert_eq!(kernel.status().await?, KernelStatus::Idle);

        Ok(())
    }

    #[tokio::test]
    async fn get_set_exec() -> Result<()> {
        let mut kernel = JupyterKernel::new();

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
