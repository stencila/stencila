use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait, eyre::Result,
    stencila_format::Format, stencila_schema::Node,
};

use lexical::LexicalDoc;
use nodes::{root_from_lexical, root_to_lexical};

mod blocks;
mod inlines;
mod lexical;
mod nodes;
mod shared;

/// A codec for Lexical JSON
pub struct LexicalCodec;

#[async_trait]
impl Codec for LexicalCodec {
    fn name(&self) -> &str {
        "lexical"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(format, Format::Lexical | Format::Koenig)
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(format, Format::Lexical | Format::Koenig)
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode(str, options)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        encode(node, options)
    }
}

pub fn decode(content: &str, _options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    let doc: LexicalDoc = serde_json::from_str(content)?;

    root_from_lexical(doc)
}

pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    let (doc, info) = root_to_lexical(node, &options)?;

    let json = match options.and_then(|options| options.compact) {
        Some(true) | None => serde_json::to_string(&doc)?,
        Some(false) => serde_json::to_string_pretty(&doc)?,
    };

    Ok((json, info))
}
