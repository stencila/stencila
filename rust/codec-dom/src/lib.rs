use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, EncodeOptions, Losses, Mapping,
};
use codec_dom_trait::{DomCodec as _, DomEncodeContext};

/// A codec for DOM HTML
pub struct DomCodec;

#[async_trait]
impl Codec for DomCodec {
    fn name(&self) -> &str {
        "dom"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_to_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Dom => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses, Mapping)> {
        let EncodeOptions {
            compact,
            standalone,
            ..
        } = options.unwrap_or_default();

        // Encode to DOM HTML
        let mut context = DomEncodeContext::default();
        node.to_dom(&mut context);
        let mut dom = context.content();

        // Add the data-root attribute to the root node
        // (the first opening tag)
        if let Some(pos) = dom.find('>') {
            dom.insert_str(pos, " root");
        }

        let html = if standalone == Some(true) {
            format!(
                r#"<!DOCTYPE html><html lang="en"><head><title>Untitled</title></head><body>{dom}</body></html>"#
            )
        } else {
            dom
        };

        let html = match compact {
            Some(true) | None => html,
            Some(false) => indent(&html)?,
        };

        Ok((html, Losses::none(), Mapping::none()))
    }
}

/// Indent HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(html: &str) -> Result<String> {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(html);
    reader.trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event),
            Err(error) => bail!(
                "Error at position {}: {error:?}\n{html}",
                reader.buffer_position()
            ),
        }?;
    }

    Ok(std::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string())
}
