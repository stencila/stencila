use kernel::{
    common::{async_trait::async_trait, eyre::Result, tracing},
    format::Format,
    generate_id,
    schema::{
        ExecutionBounds, ExecutionMessage, ImageObject, Node, SoftwareApplication,
        SoftwareApplicationOptions,
    },
    Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
};
use kernel_jinja::JinjaKernelInstance;

/// A kernel for rendering Mermaid diagrams
#[derive(Default)]
pub struct MermaidKernel;

const NAME: &str = "mermaid";

impl Kernel for MermaidKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Diagrams
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Mermaid]
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![
            ExecutionBounds::Full,
            // Fork, Limit & Box all supported because no state mutation,
            // or filesystem or network access
            ExecutionBounds::Fork,
            ExecutionBounds::Limit,
            ExecutionBounds::Box,
        ]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(MermaidKernelInstance::new()))
    }
}

#[derive(Default)]
pub struct MermaidKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja kernel instance used to render any Jinja templating
    jinja: JinjaKernelInstance,
}

impl MermaidKernelInstance {
    /// Create a new instance
    pub fn new() -> Self {
        let id = generate_id(NAME);
        Self {
            // It is important to give the Jinja kernel the same id since
            // it acting as a proxy to this kernel and a different id can
            // cause deadlocks for variable requests
            jinja: JinjaKernelInstance::with_id(&id),

            id,
        }
    }
}

#[async_trait]
impl KernelInstance for MermaidKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Generating Mermaid image");

        let mut messages = Vec::new();

        // Render any Jinja templating
        let code = if code.contains("{%") || code.contains("{{") {
            let (rendered, mut jinja_messages) = self.jinja.execute(code).await?;
            messages.append(&mut jinja_messages);

            if let Some(Node::String(rendered)) = rendered.first() {
                rendered.to_string()
            } else {
                code.to_string()
            }
        } else {
            code.to_string()
        };

        // Generate an `ImageObject` with correct media type and Mermaid code in the `content_url`
        let image = Node::ImageObject(ImageObject {
            content_url: code,
            media_type: Some("text/vnd.mermaid".to_string()),
            ..Default::default()
        });

        Ok((vec![image], messages))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Mermaid kernel info");

        Ok(SoftwareApplication {
            name: "Mermaid".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    fn variable_channel(
        &mut self,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) {
        self.jinja.variable_channel(requester, responder)
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::<MermaidKernelInstance>::default())
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{
        common::{eyre::bail, tokio},
        schema::Node,
    };

    use super::*;

    #[tokio::test]
    async fn execute() -> Result<()> {
        let mut instance = MermaidKernelInstance::default();

        let code = "graph\n  A --> B";
        let (outputs, messages) = instance.execute(code).await?;
        assert_eq!(messages, vec![]);
        if let Some(Node::ImageObject(ImageObject {
            media_type,
            content_url,
            ..
        })) = outputs.first()
        {
            assert_eq!(media_type.as_deref(), Some("text/vnd.mermaid"));
            assert_eq!(content_url, code);
        } else {
            bail!("Unexpected output type")
        }

        Ok(())
    }
}
