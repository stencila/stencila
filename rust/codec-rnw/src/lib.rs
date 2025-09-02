use std::sync::LazyLock;

use itertools::Itertools;
use regex::{Captures, Regex};

use codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType, async_trait, eyre::Result, format::Format, schema::Node, status::Status,
};
use codec_latex::LatexCodec;

/// A codec for Rnw
///
/// Noweb is an early literate programming format (https://en.wikipedia.org/wiki/Noweb).
/// Although the original Noweb could be used with a variety of formats and languages
/// its most enduring use has been with LaTeX and R in `Rnw` files.
/// In addition to the code chunks of the original Noweb, Rnw added `\Sexpr` commands
/// for inline code expressions.
pub struct RnwCodec;

#[async_trait]
impl Codec for RnwCodec {
    fn name(&self) -> &str {
        "rnw"
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    fn availability(&self) -> CodecAvailability {
        LatexCodec.availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Rnw => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Rnw => CodecSupport::LowLoss,
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
        let latex = latex_from_rnw(noweb);
        LatexCodec.from_str(&latex, options).await
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let options = EncodeOptions {
            format: Some(Format::Rnw),
            ..options.unwrap_or_default()
        };
        let (noweb, info) = LatexCodec.to_string(node, Some(options)).await?;
        Ok((noweb, info))
    }
}

/// Translate Rnw LaTeX into pure LaTeX which can be passed [`LatexCodec`] for decoding
///
/// Uses regexes to convert code chunks (<<id>>=) into `lstlisting` directives
/// and code expressions (\Sexpr) into `lstinline` directives
/// (both with the `exec` attributes). These directives are then decoded by
/// the LaTeX codec into `CodeChunk` and `CodeExpression` nodes respectively.
fn latex_from_rnw(noweb: &str) -> String {
    // Code expression regex
    static SEXPR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\\Sexpr\{([^}]*)\}").expect("invalid regex"));

    let latex = SEXPR.replace_all(noweb, |captures: &Captures| {
        let code = &captures[1];

        ["\\expr{", code, "}"].concat()
    });

    static CHUNK: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?ms)^\s*<<\s*(.*?)\s*>>=\n(.*?)^\s*@\s*$").expect("invalid regex")
    });

    let latex = CHUNK.replace_all(&latex, |captures: &Captures| {
        let mut options = captures[1].split(",").collect_vec();
        let code = &captures[2];

        let id = if let Some(id) = options.first().and_then(|first| {
            (!first.is_empty() && !first.contains("=")).then_some(first.to_string())
        }) {
            options.remove(0);
            [",id=", &id].concat()
        } else {
            String::new()
        };

        let options = if !options.is_empty() {
            [",", &options.join(",")].concat()
        } else {
            String::new()
        };

        [
            "\\begin{chunk}[r",
            &id,
            &options,
            "]\n",
            code,
            "\\end{chunk}\n",
        ]
        .concat()
    });

    latex.into()
}
