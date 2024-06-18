use codec::{common::eyre::Result, schema::Node, EncodeInfo, EncodeOptions};

use codec_html_trait::{HtmlCodec as _, HtmlEncodeContext};

/// Encode a Stencila Schema [`Node`] to a HTML string
pub(super) fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    let EncodeOptions {
        compact,
        standalone,
        ..
    } = options.unwrap_or_default();

    let mut context = HtmlEncodeContext {};

    let html = node.to_html(&mut context);
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

    Ok((html, EncodeInfo::none()))
}

/// Indent HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(html: &str) -> String {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(html);
    reader.config_mut().trim_text(true);

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
