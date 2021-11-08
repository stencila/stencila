use codec::{
    async_trait::async_trait, eyre::Result, stencila_schema::Node, utils::vec_string, Codec,
    CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_pandoc::{decode, encode, PandocCodec};
use std::path::{Path, PathBuf};

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
            from_path: true,
            to_path: true,
            unsupported_types: [
                pandoc_codec.unsupported_types,
                vec_string![
                    // TODO: Fix decoding of quotes from DOCX
                    "Quote"
                ],
            ]
            .concat(),
            ..Default::default()
        }
    }

    async fn from_path<T: AsRef<Path>>(path: &T, _options: Option<DecodeOptions>) -> Result<Node>
    where
        T: Send + Sync,
    {
        let path = PathBuf::from(path.as_ref());
        let media = [&path.to_string_lossy(), ".media"].concat();
        decode("", Some(path), "docx", &["--extract-media", &media]).await
    }

    async fn to_path<T: AsRef<Path>>(
        node: &Node,
        path: &T,
        _options: Option<EncodeOptions>,
    ) -> Result<()>
    where
        T: Send + Sync,
    {
        let path = PathBuf::from(path.as_ref());
        encode(node, Some(path), "docx", &[]).await?;
        Ok(())
    }
}
