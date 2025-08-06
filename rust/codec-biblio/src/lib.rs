use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
    },
    format::Format,
    schema::{Article, Node},
    status::Status,
};

pub mod conversion;
pub mod decode;
pub mod encode;

/// A codec for bibliographic content
pub struct BiblioCodec;

#[async_trait]
impl Codec for BiblioCodec {
    fn name(&self) -> &str {
        "reference"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Yaml | Format::Bibtex | Format::Text => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        match node_type {
            NodeType::Reference => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Yaml | Format::Text => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        match node_type {
            NodeType::Reference => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_str(
        &self,
        text: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let format = options
            .as_ref()
            .and_then(|opts| opts.format.clone())
            .unwrap_or(Format::Text);

        let references = match format {
            Format::Yaml => decode::yaml(text)?,
            Format::Bibtex => decode::bibtex(text)?,
            Format::Text => decode::text(text)?,
            _ => bail!("Unsupported format: {format}"),
        };

        let article = Article {
            references: if references.is_empty() {
                None
            } else {
                Some(references)
            },
            ..Default::default()
        };

        Ok((Node::Article(article), DecodeInfo::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let format = options
            .as_ref()
            .and_then(|opts| opts.format.clone())
            .unwrap_or(Format::Text);

        let style = options.as_ref().and_then(|opts| opts.theme.as_deref());

        let Node::Article(Article { references, .. }) = node else {
            bail!("Unsupported node type: {node}")
        };

        let references = references
            .as_ref()
            .map(|refs| refs.iter().collect::<Vec<_>>())
            .unwrap_or_default();

        let output = match format {
            Format::Yaml => encode::yaml(&references)?,
            Format::Json => encode::json(&references, style)?,
            Format::Markdown => encode::markdown(&references, style)?,
            Format::Text => encode::text(&references, style)?,
            _ => bail!("Unsupported format: {format}"),
        };

        Ok((output, EncodeInfo::none()))
    }
}
