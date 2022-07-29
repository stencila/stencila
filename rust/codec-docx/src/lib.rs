use std::path::{Path, PathBuf};

use codec::{
    common::{async_trait::async_trait, eyre::Result},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_pandoc::{decode, encode, PandocCodec};

/// A codec for Microsoft Word (.docx) files
pub struct DocxCodec {}

#[async_trait]
impl CodecTrait for DocxCodec {
    fn spec() -> Codec {
        let pandoc_codec = PandocCodec::spec();
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["docx"],
            root_types: vec_string!["Article"],
            from_string: false,
            to_string: false,
            unsupported_types: [
                pandoc_codec.unsupported_types,
                // TODO: Fix decoding of quotes from DOCX
                vec_string!["Quote"],
            ]
            .concat(),
            ..Default::default()
        }
    }

    /// Decode a document node from a DOCX file
    async fn from_path(path: &Path, _options: Option<DecodeOptions>) -> Result<Node> {
        let path = PathBuf::from(path);
        let media = [&path.to_string_lossy(), ".media"].concat();
        decode("", Some(path), "docx", &["--extract-media", &media]).await
    }

    /// Encode a document node to a DOCX file
    ///
    /// If `options.rpng_types` is empty, defaults to a standard set of types for this format.
    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
        let mut rpng_types = options
            .as_ref()
            .map(|options| options.rpng_types.clone())
            .unwrap_or_default();
        if rpng_types.is_empty() {
            rpng_types = vec_string!["CodeExpression", "CodeChunk", "Parameter"]
        }

        encode(
            node,
            Some(path),
            "docx",
            &[],
            Some(EncodeOptions {
                rpng_types,
                ..options.unwrap_or_default()
            }),
        )
        .await?;

        Ok(())
    }
}
