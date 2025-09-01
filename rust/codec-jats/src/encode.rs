use std::io::Cursor;

use quick_xml::{
    Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};
use roxmltree::{Node as XmlNode, NodeType as XmlNodeType};

use codec::{EncodeInfo, EncodeOptions, Losses, eyre::Result, schema::Node};
use codec_jats_trait::JatsCodec as _;

/// Encode a [`Node`] as JATS XML
pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
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
    use quick_xml::{Reader, Writer, events::Event};

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

/// Recursively serialise a `roxmltree::Node` (and its subtree) to XML.
pub(super) fn serialize_node(node: XmlNode) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    write_node(&mut writer, node)?;
    let bytes = writer.into_inner().into_inner();
    Ok(String::from_utf8(bytes).expect("UTF-8 in quick-xml writer"))
}

/// Internal helper that writes one node and all descendants.
fn write_node<W: std::io::Write>(w: &mut Writer<W>, node: XmlNode) -> Result<()> {
    match node.node_type() {
        XmlNodeType::Element => {
            // <elem …attrs…>
            let mut start = BytesStart::new(node.tag_name().name());
            for a in node.attributes() {
                start.push_attribute((a.name().as_bytes(), a.value().as_bytes()));
            }
            w.write_event(Event::Start(start))?;

            // children
            for child in node.children() {
                write_node(w, child)?;
            }

            // </elem>
            let end = BytesEnd::new(node.tag_name().name());
            w.write_event(Event::End(end))?;
        }

        XmlNodeType::Text => {
            w.write_event(Event::Text(BytesText::new(node.text().unwrap_or(""))))?;
        }

        XmlNodeType::Comment => {
            w.write_event(Event::Comment(BytesText::new(node.text().unwrap_or(""))))?;
        }

        // Skip document nodes / DTD etc. for brevity. Add if you need them.
        _ => {}
    }
    Ok(())
}
