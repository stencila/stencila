use codec::{common::eyre::Result, schema::Node, EncodeInfo, EncodeOptions, Losses};
use codec_jats_trait::JatsCodec as _;

/// Encode a [`Node`] as JATS XML
pub(super) fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    let EncodeOptions {
        compact,
        standalone,
        ..
    } = options.unwrap_or_default();

    if !matches!(node, Node::Article(..)) {
        return Ok((
            String::new(),
            EncodeInfo {
                losses: Losses::one(node.to_string()),
                ..Default::default()
            },
        ));
    }

    let (mut jats, losses) = node.to_jats();
    if standalone.unwrap_or_default() {
        jats.insert_str(
            0,
            r#"<?xml version="1.0" encoding="utf-8" standalone="yes" ?>"#,
        );
    }
    if matches!(compact, Some(false)) {
        jats = indent(&jats);
    }

    Ok((
        jats,
        EncodeInfo {
            losses,
            ..Default::default()
        },
    ))
}

/// Indent JATS
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(jats: &str) -> String {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(jats);
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
