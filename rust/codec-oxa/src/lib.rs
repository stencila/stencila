use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait,
    eyre::{self, Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};

mod blocks;
mod generic;
mod inlines;
mod nodes;

/// A codec for OXA JSON
pub struct OxaCodec;

#[async_trait]
impl Codec for OxaCodec {
    fn name(&self) -> &str {
        "oxa"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Oxa => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Oxa => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
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
    let value: serde_json::Value = serde_json::from_str(content)?;

    let obj = value
        .as_object()
        .ok_or_else(|| eyre::eyre!("Expected a JSON object at the root, got a non-object type"))?;

    let type_str = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("Missing or non-string \"type\" field in root object"))?;

    if type_str != "Document" {
        bail!("Expected root type \"Document\" but got \"{type_str}\"");
    }

    let node = nodes::decode_document(obj)?;

    Ok((node, DecodeInfo::none()))
}

pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    let value = nodes::encode_document(node)?;

    let json = match options.and_then(|options| options.compact) {
        Some(true) => serde_json::to_string(&value)?,
        Some(false) | None => serde_json::to_string_pretty(&value)?,
    };

    Ok((json, EncodeInfo::none()))
}
