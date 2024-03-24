use kernel_jinja::JinjaKernelInstance;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use railwind::{parse_to_string, CollectionOptions, Source};

use kernel::{
    common::{
        async_trait::async_trait, eyre::Result, once_cell::sync::Lazy, regex::Regex, tracing,
    },
    format::Format,
    schema::{
        ExecutionMessage, MessageLevel, Node, SoftwareApplication, SoftwareApplicationOptions,
    },
    Kernel, KernelInstance, KernelVariableRequester, KernelVariableResponder,
};

/// A kernel for compiling styles, including Tailwind classes and Jinja templates, into CSS.
#[derive(Default)]
pub struct StyleKernel {}

const NAME: &str = "style";

impl Kernel for StyleKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Css, Format::Tailwind]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(StyleKernelInstance {
            // It is important to give the Jinja kernel the same name since
            // it acting as a proxy to this kernel and a different name can
            // cause deadlocks for variable requests
            jinja: JinjaKernelInstance::new(NAME),
        }))
    }
}

#[derive(Default)]
pub struct StyleKernelInstance {
    jinja: JinjaKernelInstance,
}

impl StyleKernelInstance {
    /// Transpile a style specification to CSS
    async fn style_to_css(
        &mut self,
        style: &str,
    ) -> Result<(String, String, Vec<ExecutionMessage>)> {
        let mut messages = Vec::new();

        // Transpile any dollar variable interpolations to Jinja interpolation
        static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$(\w+)").expect("Invalid regex"));
        let style = REGEX.replace_all(style, "{{$1}}");

        // Render any Jinja templating
        let style = if style.contains("{%") || style.contains("{{") {
            let (rendered, mut jinja_messages) = self.jinja.execute(&style).await?;
            messages.append(&mut jinja_messages);

            if let Some(Node::String(rendered)) = rendered.first() {
                rendered.to_string()
            } else {
                style.to_string()
            }
        } else {
            style.to_string()
        };

        // Transpile Tailwind to CSS
        let (css, classes) = if !style.contains([':', '{', '}']) {
            let (css, mut tailwind_messages) = self.tailwind_to_css(&style);
            messages.append(&mut tailwind_messages);
            (css, style)
        } else {
            (style.to_string(), String::new())
        };

        // Nest the CSS within the class that we are targeting. This allows "bare" CSS to
        // be used e.g. `color: red`.
        let css = [".styled {", &css, "}"].concat();

        // Normalize the CSS (including expanding the nesting)
        let (css, normalize_message) = self.normalize_css(&css);
        if let Some(normalize_message) = normalize_message {
            messages.push(normalize_message);
        }

        Ok((css, classes, messages))
    }

    /// Transpile Tailwind to CSS
    fn tailwind_to_css(&self, tw: &str) -> (String, Vec<ExecutionMessage>) {
        let source = Source::String(tw.to_string(), CollectionOptions::String);

        let mut warnings = Vec::new();
        let css = parse_to_string(source, false, &mut warnings);

        let messages: Vec<ExecutionMessage> = warnings
            .into_iter()
            .map(|warning| ExecutionMessage {
                level: MessageLevel::Warning,
                message: warning.to_string(),
                ..Default::default()
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
    fn name(&self) -> String {
        "style".to_string()
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling style to CSS");

        let (css, classes, messages) = self.style_to_css(code).await?;

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
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{common::tokio, schema::Node};

    use super::*;

    #[tokio::test]
    async fn tailwind() -> Result<()> {
        let mut instance = StyleKernelInstance::default();

        let (outputs, messages) = instance.execute(r"bg-red-100").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String(".styled{& .bg-red-100{--tw-bg-opacity:1;background-color:rgb(254 226 226/var(--tw-bg-opacity))}}".to_string()), 
                Node::String("bg-red-100".to_string())
            ]
        );

        let (outputs, messages) = instance.execute(r"foo text-blue-800").await?;
        assert_eq!(
            messages,
            vec![ExecutionMessage {
                level: MessageLevel::Warning,
                message: "Warning on Line: 1, Col: 1 in file: ''; Could not match class 'foo'"
                    .to_string(),
                ..Default::default()
            }]
        );
        assert_eq!(
            outputs,
            vec![Node::String(".styled{& .text-blue-800{--tw-text-opacity:1;color:rgb(30 64 175/var(--tw-text-opacity))}}".to_string()), Node::String("foo text-blue-800".to_string())]
        );

        Ok(())
    }
}
