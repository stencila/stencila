use std::path::Path;

use common::{
    async_trait::async_trait,
    defaults::Defaults,
    eyre::{bail, Result},
    tokio::{
        fs::{create_dir_all, File},
        io::{AsyncReadExt, AsyncWriteExt},
    },
    tracing,
};
use format::Format;
use schema::Node;

#[async_trait]
pub trait Codec: Sync {
    /// Decode a Stencila Schema node from a string
    #[allow(clippy::wrong_self_convention)]
    async fn from_str(&self, _str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        bail!("Decoding from string is not implemented for this format")
    }

    /// Decode a Stencila Schema node from a file
    ///
    /// This function reads the file as a string and passes that on to `from_str`
    /// for decoding. If working with binary formats, you should override this function
    /// to read the file as bytes instead.
    #[tracing::instrument(skip(self))]
    async fn from_file(&self, file: &mut File, options: Option<DecodeOptions>) -> Result<Node> {
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        self.from_str(&content, options).await
    }

    /// Decode a Stencila Schema node from a file system path
    #[tracing::instrument(skip(self))]
    async fn from_path(&self, path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
        if !path.exists() {
            bail!("Path `{}` does not exist", path.display());
        }

        let mut file = File::open(path).await?;
        self.from_file(&mut file, options).await
    }

    /// Encode a Stencila Schema node to a string
    async fn to_string(&self, _node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        bail!("Encoding to a string is not implemented for this format")
    }

    /// Encode a Stencila Schema to a file
    #[tracing::instrument(skip(self))]
    async fn to_file(
        &self,
        node: &Node,
        file: &mut File,
        options: Option<EncodeOptions>,
    ) -> Result<()> {
        let content = self.to_string(node, options).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }

    /// Encode a Stencila Schema to a file system path
    #[tracing::instrument(skip(self))]
    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        let mut file = File::create(path).await?;
        self.to_file(node, &mut file, options).await
    }
}

/// Decoding options
#[derive(Debug, Default, Clone)]
pub struct DecodeOptions {
    /// The format to be decode from
    ///
    /// Most codecs only decode one format. However, for those that handle multiple
    /// format it may be necessary to specify this option.
    pub format: Option<Format>,
}

/// Encoding options
#[derive(Debug, Defaults, Clone)]
pub struct EncodeOptions {
    /// The format to encode to
    ///
    /// Most codecs only encode to one format. However, for those that handle multiple
    /// formats it may be necessary to specify this option.
    pub format: Option<Format>,

    /// Whether to encode in compact form
    ///
    /// Some formats (e.g HTML and JSON) can be encoded in either compact
    /// or "pretty-printed" (e.g. indented) forms.
    #[def = "true"]
    pub compact: bool,
}
