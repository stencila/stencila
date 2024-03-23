use kernel::{
    common::{async_trait::async_trait, eyre::Result, tracing},
    format::Format,
    schema::{ExecutionMessage, Node, SoftwareApplication, SoftwareApplicationOptions},
    Kernel, KernelInstance,
};

/// A kernel for compiling AsciiMath math to MathML.
///
/// Note that although this is all about converting AsciiMath to MathML it is implemented
/// as a kernel, rather than a codec, because the conversion is at the level of
/// an individual node (i.e. `MathBlock` or `MathInline`) rather than at the document level.
#[derive(Default)]
pub struct AsciiMathKernel {}

impl Kernel for AsciiMathKernel {
    fn name(&self) -> String {
        "asciimath".to_string()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::AsciiMath]
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(AsciiMathKernelInstance {}))
    }
}

pub struct AsciiMathKernelInstance {}

impl AsciiMathKernelInstance {
    /// Transpile AsciiMath to MathML
    fn transpile(&self, am: &str) -> String {
        mathemascii::render_mathml(mathemascii::parse(am))
    }
}

#[async_trait]
impl KernelInstance for AsciiMathKernelInstance {
    fn name(&self) -> String {
        "asciimath".to_string()
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling AsciiMath to MathML");

        let mathml = self.transpile(code).replace(
            "<math>",
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">",
        );

        Ok((vec![Node::String(mathml)], vec![]))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling AsciiMath to MathML");

        let mathml = self.transpile(code).replace(
            "<math>",
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"inline\">",
        );

        Ok((Node::String(mathml), vec![]))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting AsciiMath runtime info");

        Ok(SoftwareApplication {
            name: "AsciiMath".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                operating_system: Some(std::env::consts::OS.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{common::tokio, schema::Node};

    use super::*;

    #[tokio::test]
    async fn execute() -> Result<()> {
        let mut instance = AsciiMathKernelInstance {};

        let (outputs, messages) = instance.execute(r"a = pi r^2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\"><mi>a</mi><mo>=</mo><mi>𝜋</mi><msup><mi>r</mi><mn>2</mn></msup></math>".to_string())]);

        Ok(())
    }

    #[tokio::test]
    async fn evaluate() -> Result<()> {
        let mut instance = AsciiMathKernelInstance {};

        let (output, messages) = instance.evaluate(r"pi r^2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(output, Node::String("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"inline\"><mi>𝜋</mi><msup><mi>r</mi><mn>2</mn></msup></math>".to_string()));

        Ok(())
    }
}
