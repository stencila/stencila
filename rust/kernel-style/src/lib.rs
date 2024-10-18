use std::hash::{Hash, Hasher};

use kernel_jinja::JinjaKernelInstance;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use railwind::{parse_to_string, CollectionOptions, Source};

use kernel::{
    common::{
        async_trait::async_trait, bs58, eyre::Result, once_cell::sync::Lazy, regex::Regex,
        seahash::SeaHasher, tracing,
    },
    format::Format,
    generate_id,
    schema::{
        CodeLocation, ExecutionMessage, MessageLevel, Node, SoftwareApplication,
        SoftwareApplicationOptions,
    },
    Kernel, KernelForks, KernelInstance, KernelType, KernelVariableRequester,
    KernelVariableResponder,
};

/// A kernel for compiling styles, including Tailwind classes and Jinja templates, into CSS.
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

    fn supports_forks(&self) -> kernel::KernelForks {
        KernelForks::Yes
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
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
        static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$(\w+)").expect("Invalid regex"));
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
        let (css, classes) = if code.contains(['<', '>']) {
            // Transpile HTML in RawBlock, potentially with Tailwind classes, to CSS
            let (css, mut tailwind_messages) = self.html_to_css(code);
            messages.append(&mut tailwind_messages);
            (css, String::new())
        } else if !code.contains([';', '{', '}']) {
            // Transpile Tailwind in StyledBlock to CSS
            let (css, mut tailwind_messages) = self.tailwind_to_css(code);
            messages.append(&mut tailwind_messages);
            (css, code.to_string())
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

        // Normalize the CSS (including expanding the nesting)
        let (css, normalize_message) = self.normalize_css(&css);
        if let Some(normalize_message) = normalize_message {
            messages.push(normalize_message);
        }

        Ok((css, classes, messages))
    }

    /// Transpile Tailwind to CSS
    fn tailwind_to_css(&self, tw: &str) -> (String, Vec<ExecutionMessage>) {
        self.source_to_css(Source::String(tw.to_string(), CollectionOptions::String))
    }

    /// Parse HTML for Tailwind classes and transpile to CSS
    fn html_to_css(&self, html: &str) -> (String, Vec<ExecutionMessage>) {
        self.source_to_css(Source::String(html.to_string(), CollectionOptions::Html))
    }

    /// Tailwind [`Source`] to CSS
    fn source_to_css(&self, source: Source) -> (String, Vec<ExecutionMessage>) {
        let mut warnings = Vec::new();
        let css = parse_to_string(source, false, &mut warnings);

        let messages: Vec<ExecutionMessage> = warnings
            .into_iter()
            .map(|warning| {
                let position = warning.position();
                ExecutionMessage {
                    level: MessageLevel::Warning,
                    message: warning.message().to_string(),
                    code_location: Some(CodeLocation {
                        start_line: Some(position.line().saturating_sub(1) as u64),
                        start_column: Some(position.column().saturating_sub(1) as u64),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            })
            .collect();

        (css, messages)
    }

    /// Normalize and minify CSS
    fn normalize_css(&self, css: &str) -> (String, Option<ExecutionMessage>) {
        match StyleSheet::parse(css, ParserOptions::default()) {
            Ok(stylesheet) => {
                match stylesheet.to_css(PrinterOptions {
                    minify: true,
                    ..Default::default()
                }) {
                    Ok(result) => (result.code, None),
                    Err(error) => (
                        css.to_string(),
                        Some(ExecutionMessage::new(
                            MessageLevel::Warning,
                            error.to_string(),
                        )),
                    ),
                }
            }
            Err(error) => (
                css.to_string(),
                Some(ExecutionMessage::new(
                    MessageLevel::Warning,
                    error.to_string(),
                )),
            ),
        }
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

    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::<StyleKernelInstance>::default())
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{common::tokio, schema::Node};

    use super::*;

    #[tokio::test]
    async fn css() -> Result<()> {
        let mut instance = StyleKernelInstance::default();

        let (outputs, messages) = instance.execute(r" color: red; ").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String(".sXVJTsg4eGEt{color:red}".to_string()),
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
                Node::String(".bg-red-100{--tw-bg-opacity:1;background-color:rgb(254 226 226/var(--tw-bg-opacity))}".to_string()), 
                Node::String("bg-red-100".to_string())
            ]
        );

        let (outputs, messages) = instance.execute(r"foo text-blue-800").await?;
        assert_eq!(
            messages,
            vec![ExecutionMessage {
                level: MessageLevel::Warning,
                message: "Could not match class 'foo'".to_string(),
                code_location: Some(CodeLocation {
                    start_line: Some(0),
                    start_column: Some(0),
                    ..Default::default()
                }),
                ..Default::default()
            }]
        );
        assert_eq!(
            outputs,
            vec![
                Node::String(".text-blue-800{--tw-text-opacity:1;color:rgb(30 64 175/var(--tw-text-opacity))}".to_string()),
                Node::String("foo text-blue-800".to_string())
            ]
        );

        Ok(())
    }
}
