use std::path::Path;

use codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType,
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
};
use codec_json::JsonCodec;
use codec_pandoc::{
    coarse_to_path, pandoc_availability, pandoc_from_format, pandoc_to_format, root_from_pandoc,
    root_to_pandoc,
};
use node_reconstitute::reconstitute;

/// A codec for Open Document Format
pub struct OdtCodec;

const PANDOC_FORMAT: &str = "odt";

#[async_trait]
impl Codec for OdtCodec {
    fn name(&self) -> &str {
        "odt"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn availability(&self) -> CodecAvailability {
        pandoc_availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Odt => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Odt => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let pandoc = pandoc_from_format("", Some(path), PANDOC_FORMAT, &options).await?;
        let (mut node, info) = root_from_pandoc(pandoc, Format::Odt, &options)?;

        // If a cache is specified then use it to reconstitute the node
        let cache = if let Some(cache) = options.and_then(|options| options.cache) {
            let (cache, ..) = JsonCodec.from_path(&cache, None).await?;
            Some(cache)
        } else {
            None
        };
        reconstitute(&mut node, cache);

        Ok((node, None, info))
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let mut options = options.unwrap_or_default();
        if options.render.is_none() {
            options.render = Some(true);
        }

        if options.render.unwrap_or_default()
            && let Node::Article(article) = node
            && article.is_coarse(&Format::Latex)
        {
            return coarse_to_path(node, Format::Latex, Format::Odt, path, Some(options)).await;
        }

        let options = Some(options);
        let (pandoc, info) = root_to_pandoc(node, Format::Odt, &options)?;
        pandoc_to_format(&pandoc, Some(path), PANDOC_FORMAT, &options).await?;
        Ok(info)
    }
}
