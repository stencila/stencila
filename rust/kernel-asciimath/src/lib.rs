use kernel::{
    common::{async_trait::async_trait, eyre::Result, itertools::Itertools, tracing},
    format::Format,
    schema::{
        CodeLocation, ExecutionMessage, MessageLevel, Node, Null, SoftwareApplication,
        SoftwareApplicationOptions,
    },
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
    #[allow(clippy::result_large_err)]
    fn transpile(&self, code: &str) -> Result<String, ExecutionMessage> {
        // Replace non-ascii characters (e.g. those introduced by LLMs)
        let mut line = 0;
        let mut line_start = 0;
        let code: String = code
            .char_indices()
            .map(|(index, c)| {
                Ok(match c {
                    'α' => "alpha".to_string(),
                    'β' => "beta".to_string(),
                    'γ' => "gamma".to_string(),
                    'δ' => "delta".to_string(),
                    'ε' => "epsilon".to_string(),
                    'ζ' => "zeta".to_string(),
                    'η' => "eta".to_string(),
                    'θ' => "theta".to_string(),
                    'ι' => "iota".to_string(),
                    'κ' => "kappa".to_string(),
                    'λ' => "lambda".to_string(),
                    'μ' => "mu".to_string(),
                    'ν' => "nu".to_string(),
                    'ξ' => "xi".to_string(),
                    'ο' => "omicron".to_string(),
                    'π' | '𝜋' => "pi".to_string(),
                    'ρ' => "rho".to_string(),
                    'σ' => "sigma".to_string(),
                    'τ' => "tau".to_string(),
                    'υ' => "upsilon".to_string(),
                    'φ' => "phi".to_string(),
                    'χ' => "chi".to_string(),
                    'ψ' => "psi".to_string(),
                    'ω' => "omega".to_string(),
                    'Α' => "Alpha".to_string(),
                    'Β' => "Beta".to_string(),
                    'Γ' => "Gamma".to_string(),
                    'Δ' => "Delta".to_string(),
                    'Ε' => "Epsilon".to_string(),
                    'Ζ' => "Zeta".to_string(),
                    'Η' => "Eta".to_string(),
                    'Θ' => "Theta".to_string(),
                    'Ι' => "Iota".to_string(),
                    'Κ' => "Kappa".to_string(),
                    'Λ' => "Lambda".to_string(),
                    'Μ' => "Mu".to_string(),
                    'Ν' => "Nu".to_string(),
                    'Ξ' => "Xi".to_string(),
                    'Ο' => "Omicron".to_string(),
                    'Π' => "Pi".to_string(),
                    'Ρ' => "Rho".to_string(),
                    'Σ' => "Sigma".to_string(),
                    'Τ' => "Tau".to_string(),
                    'Υ' => "Upsilon".to_string(),
                    'Φ' => "Phi".to_string(),
                    'Χ' => "Chi".to_string(),
                    'Ψ' => "Psi".to_string(),
                    'Ω' => "Omega".to_string(),
                    '\n' => {
                        line += 1;
                        line_start = index;
                        "\n".to_string()
                    }
                    _ if c.is_ascii() => c.to_string(),
                    _ => {
                        return Err(ExecutionMessage {
                            level: MessageLevel::Error,
                            message: "Non-ASCII characters are not allowed".to_string(),
                            code_location: Some(CodeLocation {
                                start_line: Some(line),
                                start_column: Some((index - line_start) as u64),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                    }
                })
            })
            .try_collect()?;

        Ok(mathemascii::render_mathml(mathemascii::parse(&code)))
    }
}

#[async_trait]
impl KernelInstance for AsciiMathKernelInstance {
    fn name(&self) -> String {
        "asciimath".to_string()
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling AsciiMath to MathML");

        let mathml = match self.transpile(code) {
            Ok(mathml) => mathml,
            Err(error) => return Ok((Vec::new(), vec![error])),
        };

        let mathml = mathml.replace(
            "<math>",
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">",
        );

        Ok((vec![Node::String(mathml)], vec![]))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        tracing::trace!("Transpiling AsciiMath to MathML");

        let mathml = match self.transpile(code) {
            Ok(mathml) => mathml,
            Err(error) => return Ok((Node::Null(Null), vec![error])),
        };

        let mathml = mathml.replace(
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

    #[tokio::test]
    async fn evaluate_non_ascii() -> Result<()> {
        let mut instance = AsciiMathKernelInstance {};

        let (output, messages) = instance.evaluate(r"π Ω^2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(output, Node::String("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"inline\"><mi>𝜋</mi><msup><mi>Ω</mi><mn>2</mn></msup></math>".to_string()));

        let (output, messages) = instance.evaluate("a\nb😃c").await?;
        assert_eq!(messages[0].message, "Non-ASCII characters are not allowed");
        assert_eq!(
            messages[0].code_location.as_ref().unwrap().start_line,
            Some(1)
        );
        assert_eq!(
            messages[0].code_location.as_ref().unwrap().start_column,
            Some(2)
        );
        assert_eq!(output, Node::Null(Null));

        Ok(())
    }
}
