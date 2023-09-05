use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, EncodeOptions, Losses,
};
use codec_markdown_trait::MarkdownCodec as _;

/// A codec for Markdown
pub struct MarkdownCodec;

#[async_trait]
impl Codec for MarkdownCodec {
    fn name(&self) -> &str {
        "markdown"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Markdown]
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let (markdown, losses) = node.to_markdown();

        let markdown = markdown.trim().to_string();

        Ok((markdown, losses))
    }
}
