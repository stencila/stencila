use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, EncodeOptions, Losses,
};
use codec_html_trait::HtmlCodec as _;

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
    ) -> Result<(String, Losses)> {
        let EncodeOptions {
            compact,
            standalone,
            ..
        } = options.unwrap_or_default();

        let html = node.to_html();

        let html = if standalone == Some(true) {
            format!(
                r#"<!DOCTYPE html><html lang="en"><head><title>Untitled</title></head><body>{html}</body></html>"#
            )
        } else {
            html
        };

        let html = match compact {
            Some(true) | None => minify(&html),
            Some(false) => indent(&html),
        };

        Ok((html, Losses::none()))
    }
}

/// Minify HTML
pub fn minify(html: &str) -> String {
    let cfg = minify_html::Cfg {
        ensure_spec_compliant_unquoted_attribute_values: true,
        minify_css: true,
        minify_js: true,
        remove_processing_instructions: true,
        keep_closing_tags: false,
        keep_comments: false,
        // These more "extreme" (and sometimes hard to understand) minification options are
        // not made configurable (more likely to cause problems and require documentation for little gain)
        do_not_minify_doctype: true,
        keep_html_and_head_opening_tags: true,
        keep_spaces_between_attributes: true,
        remove_bangs: false,
        ..Default::default()
    };

    let bytes = minify_html::minify(html.as_bytes(), &cfg);

    String::from_utf8_lossy(&bytes).to_string()
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
