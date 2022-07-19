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

    async fn from_path(path: &Path, _options: Option<DecodeOptions>) -> Result<Node> {
        let path = PathBuf::from(path);
        let media = [&path.to_string_lossy(), ".media"].concat();
        decode("", Some(path), "docx", &["--extract-media", &media]).await
    }

    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
        encode(
            node,
            Some(path),
            "docx",
            &[],
            Some(EncodeOptions {
                rpng_content: true,
                ..options.unwrap_or_default()
            }),
        )
        .await?;
        Ok(())
    }
}
