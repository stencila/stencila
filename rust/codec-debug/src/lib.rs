use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, EncodeOptions, Losses,
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

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Debug]
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_from_path(&self) -> bool {
        false
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let debug = match compact {
            true => format!("{node:?}"),
            false => format!("{node:#?}"),
        };

        Ok((debug, Losses::new()))
    }
}
