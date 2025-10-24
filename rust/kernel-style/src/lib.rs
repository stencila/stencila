use std::{
    hash::{Hash, Hasher},
    sync::LazyLock,
};

use regex::Regex;
use seahash::SeaHasher;

use stencila_kernel::{
    Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
    async_trait,
    eyre::Result,
    generate_id,
    stencila_format::Format,
    stencila_schema::{
        ExecutionBounds, ExecutionMessage, Node, SoftwareApplication, SoftwareApplicationOptions,
    },
};
use stencila_kernel_jinja::JinjaKernelInstance;

/// A kernel for compiling styles, optionally including Jinja templates, into CSS and utility classes.
#[derive(Default)]
pub struct StyleKernel;

const NAME: &str = "style";

impl Kernel for StyleKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Styling
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Css, Format::Html, Format::Tailwind]
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
        Ok(Box::new(StyleKernelInstance::new()))
    }
}

#[derive(Default)]
pub struct StyleKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja kernel instance used to render any Jinja templating
    jinja: JinjaKernelInstance,
}

impl StyleKernelInstance {
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

    /// Transpile code (CSS, Tailwind classes, or HTML) to CSS
    async fn code_to_css(&mut self, code: &str) -> Result<(String, String, Vec<ExecutionMessage>)> {
        let mut messages = Vec::new();

        // Transpile any dollar variable interpolations to Jinja interpolation
        static REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\$(\w+)").expect("Invalid regex"));
        let code = REGEX.replace_all(code, "{{$1}}");

        // Render any Jinja templating
        let style = if code.contains("{%") || code.contains("{{") {
            let (rendered, mut jinja_messages) = self.jinja.execute(&code).await?;
            messages.append(&mut jinja_messages);

            if let Some(Node::String(rendered)) = rendered.first() {
                rendered.to_string()
            } else {
                code.to_string()
            }
        } else {
            code.to_string()
        };

        // Trim to avoid whitespace at ends changing hashes and to
        // detect if it ends with a closing brace (see below)
        let code = style.trim();

        // Currently, there is no way to tell the kernel what language the style is
        // in. So this assumes it is Tailwind unless it contains characters only found in CSS.
        let (css, classes) = if !code.contains([';', '{', '}']) {
            (String::new(), code.to_string())
        } else if code.ends_with('}') {
            // Complete CSS stylesheet in RawBlock: just return the CSS
            (code.to_string(), String::new())
        } else {
            // Inline CSS rules in StyledBlock: wrap the CSS into a unique class
            let mut hash = SeaHasher::new();
            code.hash(&mut hash);
            let digest = hash.finish();

            // Prefix with letter to avoid a number ever being first
            let class = ["s", &bs58::encode(digest.to_be_bytes()).into_string()].concat();

            let css = [".", &class, "{", code, "}"].concat();
            (css, class)
        };

        Ok((css, classes, messages))
    }
}

#[async_trait]
impl KernelInstance for StyleKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling style to CSS");

        let (css, classes, messages) = self.code_to_css(code).await?;

        let css = Node::String(css);
        let classes = Node::String(classes);

        Ok((vec![css, classes], messages))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting style runtime info");

        Ok(SoftwareApplication {
            name: "Style".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                operating_system: Some(std::env::consts::OS.to_string()),
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
        Ok(Box::<StyleKernelInstance>::default())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use stencila_kernel::stencila_schema::Node;

    use super::*;

    #[tokio::test]
    async fn css() -> Result<()> {
        let mut instance = StyleKernelInstance::default();

        let (outputs, messages) = instance.execute(r" color: red; ").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String(".sXVJTsg4eGEt{color: red;}".to_string()),
                Node::String("sXVJTsg4eGEt".to_string())
            ]
        );

        Ok(())
    }

    #[tokio::test]
    async fn tailwind() -> Result<()> {
        let mut instance = StyleKernelInstance::default();

        let (outputs, messages) = instance.execute(r"bg-red-100").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String(String::new()),
                Node::String("bg-red-100".to_string())
            ]
        );

        let (outputs, messages) = instance.execute(r"foo text-blue-800").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String(String::new()),
                Node::String("foo text-blue-800".to_string())
            ]
        );

        Ok(())
    }
}
