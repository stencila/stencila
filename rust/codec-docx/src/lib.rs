use codec_pandoc::{decode, encode};
use codec_trait::{
    async_trait::async_trait, eyre::Result, stencila_schema::Node, Codec, DecodeOptions,
    EncodeOptions,
};
use std::path::{Path, PathBuf};

/// A codec for Microsoft Word (.docx) files
pub struct DocxCodec {}

#[async_trait]
impl Codec for DocxCodec {
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
