use stencila_kernel::{
    Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
    async_trait,
    eyre::{Result, bail},
    generate_id,
    stencila_format::Format,
    stencila_schema::{
        ExecutionBounds, ExecutionMessage, ImageObject, Node, Null, SoftwareApplication,
        StringOrNumber,
    },
};
use stencila_kernel_jinja::JinjaKernelInstance;
use stencila_version::STENCILA_VERSION;

/// A kernel for rendering visualization that can be defined via a JSON spec
/// (e.g. Cytoscape, Plotly, VegaLite)
#[derive(Default)]
pub struct JvizKernel;

const NAME: &str = "jviz";

impl Kernel for JvizKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Visualization
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![
            Format::Cytoscape,
            Format::Echarts,
            Format::Plotly,
            Format::VegaLite,
        ]
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![
            ExecutionBounds::Main,
            // Fork & Box supported because no state mutation,
            // or filesystem or network access
            ExecutionBounds::Fork,
            ExecutionBounds::Box,
        ]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(JvizKernelInstance::new()))
    }
}

#[derive(Default)]
pub struct JvizKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja kernel instance used to render any Jinja templating
    jinja: JinjaKernelInstance,
}

impl JvizKernelInstance {
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
impl KernelInstance for JvizKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        self.execute_language(code, None).await
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        self.evaluate_language(code, None).await
    }

    async fn execute_language(
        &mut self,
        code: &str,
        language: Option<&str>,
    ) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Generating Jviz image");

        let Some(language) = language else {
            bail!("Language must be specified")
        };

        let format = Format::from_name(language);
        if !JvizKernel.supports_language(&format) {
            bail!("Unsupported language: {format}")
        }

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

        // Generate an `ImageObject` with correct media type and JSON spec in the `content_url`
        let image = Node::ImageObject(ImageObject {
            media_type: Some(format.media_type()),
            content_url: code,
            ..Default::default()
        });

        Ok((vec![image], messages))
    }

    async fn evaluate_language(
        &mut self,
        code: &str,
        language: Option<&str>,
    ) -> Result<(Node, Vec<ExecutionMessage>)> {
        let (nodes, messages) = self.execute_language(code, language).await?;
        Ok((
            nodes
                .first()
                .map_or_else(|| Node::Null(Null), |node| node.clone()),
            messages,
        ))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Jviz kernel info");

        Ok(SoftwareApplication {
            name: "Jviz".to_string(),
            version: Some(StringOrNumber::from(STENCILA_VERSION)),
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
        Ok(Box::<JvizKernelInstance>::default())
    }
}
