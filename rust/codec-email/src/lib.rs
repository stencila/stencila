//! A codec for encoding Stencila documents to email-friendly HTML via MJML

use mrml::prelude::render::RenderOptions;

use stencila_codec::{
    Codec, CodecSupport, EncodeInfo, EncodeOptions, async_trait,
    eyre::{Result, bail, eyre},
    stencila_format::Format,
    stencila_schema::{Node, NodeType},
};

mod encode_article;
mod encode_blocks;
mod encode_inlines;
mod encode_theme;

use encode_article::encode_article;
use encode_blocks::encode_blocks;
use encode_inlines::encode_inlines;
use encode_theme::*;

/// A codec for encoding to MJML and email-friendly HTML
pub struct EmailCodec;

#[async_trait]
impl Codec for EmailCodec {
    fn name(&self) -> &str {
        "email"
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_from_path(&self) -> bool {
        false
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Email | Format::Mjml => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        match node_type {
            NodeType::Article => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let options = options.unwrap_or_default();
        let format = options.format.clone().unwrap_or(Format::Email);

        // Get theme variables
        let theme_vars = get_theme_vars(&options).await?;

        // Generate MJML
        let (mjml, losses) = match node {
            Node::Article(article) => encode_article(article, theme_vars.as_ref()),
            _ => bail!("EmailCodec only supports encoding Article nodes"),
        };

        let encode_info = EncodeInfo {
            losses,
            mapping: stencila_codec::Mapping::none(),
        };

        match format {
            Format::Mjml => Ok((mjml, encode_info)),
            Format::Email => {
                // Parse and render MJML to HTML
                let root = mrml::parse(&mjml).map_err(|e| eyre!("Failed to parse MJML: {e}"))?;
                let opts = RenderOptions::default();
                let html = root
                    .element
                    .render(&opts)
                    .map_err(|e| eyre!("Failed to render MJML to HTML: {e}"))?;
                Ok((html, encode_info))
            }
            _ => bail!("Unsupported format: {format}"),
        }
    }
}

/// HTML escape special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_codec::stencila_schema::{Article, Block, Inline, Paragraph, Text};

    #[tokio::test]
    async fn test_simple_article_to_mjml() -> Result<()> {
        let article = Article {
            title: Some(vec![Inline::Text(Text {
                value: "Test Article".into(),
                ..Default::default()
            })]),
            content: vec![Block::Paragraph(Paragraph {
                content: vec![Inline::Text(Text {
                    value: "Hello, world!".into(),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        };

        let codec = EmailCodec;
        let (mjml, _info) = codec
            .to_string(
                &Node::Article(article),
                Some(EncodeOptions {
                    format: Some(Format::Mjml),
                    ..Default::default()
                }),
            )
            .await?;

        assert!(mjml.contains("<mjml>"));
        assert!(mjml.contains("Test Article"));
        assert!(mjml.contains("Hello, world!"));
        assert!(mjml.contains("</mjml>"));

        Ok(())
    }

    #[tokio::test]
    async fn test_mjml_compiles_to_html() -> Result<()> {
        let article = Article {
            title: Some(vec![Inline::Text(Text {
                value: "Test".into(),
                ..Default::default()
            })]),
            content: vec![Block::Paragraph(Paragraph {
                content: vec![Inline::Text(Text {
                    value: "Content".into(),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        };

        let codec = EmailCodec;
        let (html, _info) = codec
            .to_string(
                &Node::Article(article),
                Some(EncodeOptions {
                    format: Some(Format::Email),
                    ..Default::default()
                }),
            )
            .await?;

        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("Test"));
        assert!(html.contains("Content"));

        Ok(())
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }
}
