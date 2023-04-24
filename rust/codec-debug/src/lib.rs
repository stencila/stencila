use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    Codec, EncodeOptions,
};

/// A codec for the Rust debug format
///
/// This is mainly useful for debugging (unsurprisingly :),
/// in particular being able to check exactly which variants
/// of enums in the schema are present within a document.
pub struct DebugCodec;

#[async_trait]
impl Codec for DebugCodec {
    fn formats(&self) -> Vec<Format> {
        vec![Format::Debug]
    }

    async fn to_string(&self, node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        match compact {
            true => Ok(format!("{node:?}")),
            false => Ok(format!("{node:#?}")),
        }
    }
}
