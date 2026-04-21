use std::path::Path;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, async_trait, eyre::Result, stencila_format::Format,
    stencila_schema::Node,
};

mod decode;
use decode::decode_path;

/// A codec for decoding MECA
///
/// See https://www.niso.org/standards-committees/meca
pub struct MecaCodec;

#[async_trait]
impl Codec for MecaCodec {
    fn name(&self) -> &str {
        "meca"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(format, Format::Meca)
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        decode_path(path, options).await
    }
}
