use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, EncodeOptions, Losses, Mapping,
};

/// A codec for the Rust debug format
///
/// This is mainly useful for debugging (unsurprisingly :),
/// in particular being able to check exactly which variants
/// of enums in the schema are present within a document.
pub struct DebugCodec;

#[async_trait]
impl Codec for DebugCodec {
    fn name(&self) -> &str {
        "debug"
    }

    fn status(&self) -> Status {
        Status::Stable
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_from_path(&self) -> bool {
        false
    }

    fn supports_to_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Debug => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses, Mapping)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let debug = match compact {
            Some(true) => format!("{node:?}"),
            Some(false) | None => format!("{node:#?}"),
        };

        Ok((debug, Losses::none(), Mapping::none()))
    }
}
