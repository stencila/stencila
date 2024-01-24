use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, EncodeOptions, Losses, Mapping,
};
use codec_html_trait::{HtmlCodec as _, HtmlEncodeContext};

/// A codec for HTML
pub struct HtmlCodec;

#[async_trait]
impl Codec for HtmlCodec {
    fn name(&self) -> &str {
        "html"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_to_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Html => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Data
            String | Cord => NoLoss,
            // Prose Inlines
            Text | Emphasis | Strong | Subscript | Superscript | Underline => NoLoss,
            // Prose Blocks
            Section | Heading | Paragraph | ThematicBreak => NoLoss,
            // Code
            CodeInline | CodeBlock => NoLoss,
            // Fallback to low loss
            _ => LowLoss,
        }
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses, Mapping)> {
        let EncodeOptions {
            compact,
            standalone,
            dom,
            ..
        } = options.unwrap_or_default();

        let mut context = HtmlEncodeContext {
            dom: dom.unwrap_or_default(),
        };

        let mut html = node.to_html(&mut context);

        // Add the data-root attribute to the root node
        // (the first opening tag)
        if context.dom {
            if let Some(pos) = html.find('>') {
                html.insert_str(pos, " data-root");
            }
        }

        let html = if standalone == Some(true) {
            format!(
                r#"<!DOCTYPE html><html lang="en"><head><title>Untitled</title></head><body>{html}</body></html>"#
            )
        } else {
            html
        };

        let html = match compact {
            Some(true) | None => html,
            Some(false) => indent(&html),
        };

        Ok((html, Losses::none(), Mapping::none()))
    }
}

/// Indent HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(html: &str) -> String {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(html);
    reader.trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event),
            Err(error) => panic!(
                "Error at position {}: {:?}",
                reader.buffer_position(),
                error
            ),
        }
        .expect("Failed to parse XML");
    }

    std::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string()
}
