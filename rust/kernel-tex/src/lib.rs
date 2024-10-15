use kernel::{
    common::{
        async_trait::async_trait, eyre::Result, once_cell::sync::Lazy, regex::Regex, tracing,
    },
    format::Format,
    generate_id,
    schema::{
        ExecutionMessage, MessageLevel, Node, SoftwareApplication, SoftwareApplicationOptions,
    },
    Kernel, KernelForks, KernelInstance, KernelType,
};
use latex2mathml::{latex_to_mathml, DisplayStyle};

const NAME: &str = "tex";

/// A kernel for compiling TeX math to MathML.
///
/// Note that although this is all about converting TeX to MathML it is implemented
/// as a kernel, rather than a codec, because the conversion is at the level of
/// an individual node (i.e. `MathBlock` or `MatchInline`) rather than at the document level.
#[derive(Default)]
pub struct TexKernel;

impl Kernel for TexKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Math
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Tex, Format::Latex]
    }

    fn supports_forks(&self) -> kernel::KernelForks {
        KernelForks::Yes
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(TexKernelInstance::new()))
    }
}

pub struct TexKernelInstance {
    /// The unique id of the kernel instance
    id: String,
}

impl Default for TexKernelInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl TexKernelInstance {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            id: generate_id(NAME),
        }
    }

    /// Transpile TeX to MathML
    fn transpile(&self, tex: &str, style: DisplayStyle) -> (Option<String>, Vec<ExecutionMessage>) {
        match latex_to_mathml(tex, style) {
            Ok(mathml) => {
                // Some (most?) errors are embedded into the MathML so we attempt to regex them out
                static REGEX: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"\[PARSE ERROR: (.*?)\]").expect("invalid regex"));

                if let Some(matches) = REGEX.captures(&mathml) {
                    (
                        None,
                        vec![ExecutionMessage {
                            level: MessageLevel::Exception,
                            message: matches.get(1).map_or_else(
                                || "Parse error".to_string(),
                                |group| group.as_str().to_string(),
                            ),
                            error_type: Some("Syntax error".to_string()),
                            ..Default::default()
                        }],
                    )
                } else {
                    (Some(mathml), vec![])
                }
            }
            Err(error) => (
                None,
                vec![ExecutionMessage::new(
                    MessageLevel::Exception,
                    error.to_string(),
                )],
            ),
        }
    }
}

#[async_trait]
impl KernelInstance for TexKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling TeX to MathML");

        let (mathml, messages) = self.transpile(code, DisplayStyle::Block);
        let mathml = mathml.map_or_else(Vec::new, |mathml| vec![Node::String(mathml)]);

        Ok((mathml, messages))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling TeX to MathML");

        let (mathml, messages) = self.transpile(code, DisplayStyle::Inline);
        let mathml = mathml.map_or_else(|| Node::String(String::new()), Node::String);

        Ok((mathml, messages))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting TeX runtime info");

        Ok(SoftwareApplication {
            name: "Tex".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                operating_system: Some(std::env::consts::OS.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new()))
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{common::tokio, schema::Node};

    use super::*;

    #[tokio::test]
    async fn execute() -> Result<()> {
        let mut instance = TexKernelInstance::new();

        let (outputs, messages) = instance.execute(r"a = \pi r^2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\"><mi>a</mi><mo>=</mo><mi>π</mi><msup><mi>r</mi><mn>2</mn></msup></math>".to_string())]);

        let (outputs, messages) = instance.execute(r"\foo").await?;
        assert_eq!(
            messages,
            vec![ExecutionMessage {
                level: MessageLevel::Exception,
                message: "Undefined(\"Command(\\\"foo\\\")\")".to_string(),
                error_type: Some("Syntax error".to_string()),
                ..Default::default()
            }]
        );
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn evaluate() -> Result<()> {
        let mut instance = TexKernelInstance::new();

        let (output, messages) = instance.evaluate(r"\pi r^2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(output, Node::String("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"inline\"><mi>π</mi><msup><mi>r</mi><mn>2</mn></msup></math>".to_string()));

        let (output, messages) = instance.evaluate(r"\foo").await?;
        assert_eq!(
            messages,
            vec![ExecutionMessage {
                level: MessageLevel::Exception,
                message: "Undefined(\"Command(\\\"foo\\\")\")".to_string(),
                error_type: Some("Syntax error".to_string()),
                ..Default::default()
            }]
        );
        assert_eq!(output, Node::String(String::new()));

        Ok(())
    }
}
