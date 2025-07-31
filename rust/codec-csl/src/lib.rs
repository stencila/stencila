use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    common::{async_trait::async_trait, eyre::Result, serde_json},
    format::Format,
    schema::Node,
    status::Status,
};

mod date;
mod item;
mod name;
mod ordinary;

/// A codec for CSL-JSON (Citation Style Language JSON)
/// 
/// See https://citeproc-js.readthedocs.io/en/latest/csl-json/
pub struct CslCodec;

#[async_trait]
impl Codec for CslCodec {
    fn name(&self) -> &str {
        "csl"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Csl => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Csl => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let csl_item: item::Item = serde_json::from_str(str)?;
        let article = csl_item.into();

        Ok((Node::Article(article), DecodeInfo::default()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let json = if options.and_then(|opts| opts.compact).unwrap_or_default() {
            serde_json::to_string(node)?
        } else {
            serde_json::to_string_pretty(node)?
        };

        Ok((json, EncodeInfo::none()))
    }
}
