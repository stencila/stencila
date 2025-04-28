use codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType,
    common::{
        async_trait::async_trait,
        eyre::Result,
        once_cell::sync::Lazy,
        regex::{Captures, Regex},
    },
    format::Format,
    schema::Node,
    status::Status,
};
use codec_latex::LatexCodec;

/// A codec for Noweb
///
/// Noweb is an early literate programming format (https://en.wikipedia.org/wiki/Noweb).
/// Although, the original Noweb, could be used with other formats such as HTML and plain text,
/// this codec is for Noweb + LaTeX only.
///
/// In addition to the code chunks of the original Noweb, this codec also supports
/// Rnw style `\Sexpr` elements for inline code expressions.
pub struct NowebCodec;

#[async_trait]
impl Codec for NowebCodec {
    fn name(&self) -> &str {
        "noweb"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn availability(&self) -> CodecAvailability {
        LatexCodec.availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Noweb => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Noweb => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        LatexCodec.supports_from_type(node_type)
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        LatexCodec.supports_to_type(node_type)
    }

    async fn from_str(
        &self,
        noweb: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let latex = latex_from_noweb(&noweb);
        LatexCodec.from_str(&latex, options).await
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let options = EncodeOptions {
            format: Some(Format::Noweb),
            ..options.unwrap_or_default()
        };
        let (noweb, info) = LatexCodec.to_string(node, Some(options)).await?;
        Ok((noweb, info))
    }
}

/// Translate Noweb LaTeX into pure LaTeX which can be passed [`LatexCodec`] for decoding
///
/// Uses regexes to convert code chunks (<<id>>=) into `lstlisting` directives
/// and code expressions (\Sexpr) into `lstinline` directives
/// (both with the `exec` attributes). These directives are then decoded by
/// the LaTeX codec into `CodeChunk` and `CodeExpression` nodes respectively.
fn latex_from_noweb(noweb: &str) -> String {
    static SEXPR: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\\Sexpr\{([^}]*)\}").expect("invalid regex"));

    let latex = SEXPR.replace_all(noweb, |captures: &Captures| {
        let code = &captures[1];

        // Delimiting character depends upon whether it is in the code
        let char = if !code.contains("!") {
            "!"
        } else if !code.contains("|") {
            "|"
        } else if !code.contains("+") {
            "+"
        } else {
            "?"
        };

        [r"\lstinline[language=rexec]", char, code, char].concat()
    });

    latex.into()
}
