use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, EncodeOptions, Losses,
};
use node_html::ToHtml;

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

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Html]
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let html = node.to_html();
        let html = match compact {
            true => html,
            false => indent(&html),
        };

        Ok((html, Losses::new()))
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
