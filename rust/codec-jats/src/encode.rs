use std::io::Cursor;

use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};
use roxmltree::{Node as XmlNode, NodeType as XmlNodeType};

use stencila_codec::{
    EncodeInfo, EncodeOptions, Losses,
    eyre::{Result, bail, eyre},
    stencila_schema::Node,
};
use stencila_codec_jats_trait::{encode::escape, to_jats};

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

    let (mut jats, losses) = to_jats(node)?;
    if standalone.unwrap_or_default() {
        jats.insert_str(
            0,
            concat!(
                "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\" ?>\n",
                "<!DOCTYPE article SYSTEM \"https://jats.nlm.nih.gov/archiving/1.4/",
                "JATS-archivearticle1-4-mathml3.dtd\">\n"
            ),
        );
    }
    if matches!(compact, Some(false)) {
        jats = indent(&jats)?;
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
fn indent(jats: &str) -> Result<String> {
    let mut reader = Reader::from_str(jats);
    reader.config_mut().trim_text(false);

    let mut roots = Vec::new();
    let mut elements = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Eof => break,
            Event::Start(start) => elements.push(XmlElement {
                start: start.into_owned(),
                children: Vec::new(),
                end: None,
            }),
            Event::End(end) => {
                let Some(mut element) = elements.pop() else {
                    bail!(
                        "unexpected closing XML element at position {}",
                        reader.buffer_position()
                    );
                };
                element.end = Some(end.into_owned());
                push_xml_node(&mut roots, &mut elements, BufferedXmlNode::Element(element));
            }
            event => push_xml_node(
                &mut roots,
                &mut elements,
                BufferedXmlNode::Event(event.into_owned()),
            ),
        }
    }

    if !elements.is_empty() {
        bail!("unclosed XML elements after formatting JATS");
    }

    let mut writer = Writer::new(Vec::new());
    for root in &roots {
        write_xml_node(&mut writer, root, 0, false)?;
    }

    Ok(String::from_utf8(writer.into_inner())?)
}

/// An XML element buffered so its content model can be inspected before formatting.
struct XmlElement {
    start: BytesStart<'static>,
    children: Vec<BufferedXmlNode>,
    end: Option<BytesEnd<'static>>,
}

/// A buffered XML node.
enum BufferedXmlNode {
    Element(XmlElement),
    Event(Event<'static>),
}

/// Add a node to the open element, or to the document roots when at top level.
fn push_xml_node(
    roots: &mut Vec<BufferedXmlNode>,
    elements: &mut [XmlElement],
    node: BufferedXmlNode,
) {
    if let Some(parent) = elements.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}

/// Write a buffered XML node, indenting structural content only.
fn write_xml_node<W: std::io::Write>(
    writer: &mut Writer<W>,
    node: &BufferedXmlNode,
    depth: usize,
    compact: bool,
) -> Result<()> {
    let element = match node {
        BufferedXmlNode::Element(element) => element,
        BufferedXmlNode::Event(event) => {
            writer.write_event(event.borrow())?;
            return Ok(());
        }
    };

    writer.write_event(Event::Start(element.start.borrow()))?;

    // Indentation is significant character data in mixed-content elements. Once
    // such an element is entered, keep its entire subtree compact so spaces on
    // either side of inline markup remain exactly as encoded.
    let compact = compact || element.has_mixed_content();
    let has_structural_children = !compact
        && element
            .children
            .iter()
            .any(BufferedXmlNode::is_structural_child);

    for child in &element.children {
        if has_structural_children && child.is_structural_child() {
            write_indent(writer, depth + 1)?;
        }
        write_xml_node(writer, child, depth + 1, compact)?;
    }

    if has_structural_children {
        write_indent(writer, depth)?;
    }
    let end = element
        .end
        .as_ref()
        .ok_or_else(|| eyre!("missing closing XML element"))?;
    writer.write_event(Event::End(end.borrow()))?;
    Ok(())
}

impl XmlElement {
    /// Whether indentation within this element could change its text content.
    fn has_mixed_content(&self) -> bool {
        is_mixed_content_element(self.start.name().as_ref())
            || self.children.iter().any(|child| {
                matches!(
                    child,
                    BufferedXmlNode::Event(Event::Text(_) | Event::CData(_))
                )
            })
    }
}

impl BufferedXmlNode {
    /// Whether a node should begin on its own line in structural content.
    fn is_structural_child(&self) -> bool {
        matches!(
            self,
            Self::Element(_)
                | Self::Event(
                    Event::Empty(_)
                        | Event::Comment(_)
                        | Event::CData(_)
                        | Event::PI(_)
                        | Event::DocType(_)
                )
        )
    }
}

/// Return whether a JATS element has a mixed-content content model.
fn is_mixed_content_element(name: &[u8]) -> bool {
    is_inline_element(name)
        || matches!(
            name,
            b"article-title"
                | b"alt-title"
                | b"kwd"
                | b"label"
                | b"license-p"
                | b"mixed-citation"
                | b"p"
                | b"subject"
                | b"subtitle"
                | b"td"
                | b"th"
                | b"title"
                | b"trans-title"
        )
}

/// Return whether a JATS element participates in mixed inline content.
fn is_inline_element(name: &[u8]) -> bool {
    matches!(
        name,
        b"abbrev"
            | b"alternatives"
            | b"bold"
            | b"break"
            | b"code"
            | b"email"
            | b"ext-link"
            | b"inline-formula"
            | b"inline-graphic"
            | b"inline-media"
            | b"italic"
            | b"milestone-end"
            | b"milestone-start"
            | b"monospace"
            | b"named-content"
            | b"overline"
            | b"private-char"
            | b"roman"
            | b"ruby"
            | b"sans-serif"
            | b"sc"
            | b"strike"
            | b"styled-content"
            | b"sub"
            | b"sup"
            | b"underline"
            | b"uri"
            | b"xref"
    )
}

/// Write a newline followed by two-space indentation.
fn write_indent<W: std::io::Write>(writer: &mut Writer<W>, depth: usize) -> Result<()> {
    write!(writer.get_mut(), "\n{:width$}", "", width = depth * 2)?;
    Ok(())
}

/// Recursively serialise a `roxmltree::Node` (and its subtree) to XML.
///
/// The serialized fragment stands on its own. Namespace bindings are declared
/// where needed and scoped to each element, preserving nested default
/// namespaces and prefix rebindings in content such as MathML annotations.
pub(super) fn serialize_node(node: XmlNode) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    // The xml prefix is implicitly bound in every XML document.
    let mut namespaces = vec![(
        Some("xml".to_string()),
        "http://www.w3.org/XML/1998/namespace".to_string(),
    )];
    write_node(&mut writer, node, &mut namespaces)?;
    let bytes = writer.into_inner().into_inner();
    Ok(String::from_utf8(bytes)?)
}

/// Qualify a node or attribute name with a namespace prefix.
fn qualified_name(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(prefix) if !prefix.is_empty() => [prefix, ":", name].concat(),
        _ => name.to_string(),
    }
}

/// Declare a binding only when it differs from the binding currently in scope.
/// An empty URI resets an inherited default namespace for an unqualified node.
fn declare_namespace(
    start: &mut BytesStart,
    namespaces: &mut Vec<(Option<String>, String)>,
    prefix: Option<&str>,
    uri: &str,
) {
    let current = namespaces
        .iter()
        .rev()
        .find(|(name, _)| name.as_deref() == prefix)
        .map(|(_, uri)| uri.as_str())
        .unwrap_or_default();
    if current == uri {
        return;
    }

    let declaration = match prefix {
        Some(prefix) => ["xmlns:", prefix].concat(),
        None => "xmlns".to_string(),
    };
    start.push_attribute((declaration.as_bytes(), escape(uri).as_bytes()));
    namespaces.push((prefix.map(String::from), uri.to_string()));
}

/// Internal helper that writes one node and all descendants.
fn write_node<W: std::io::Write>(
    w: &mut Writer<W>,
    node: XmlNode,
    namespaces: &mut Vec<(Option<String>, String)>,
) -> Result<()> {
    match node.node_type() {
        XmlNodeType::Element => {
            let scope_start = namespaces.len();
            let namespace = node.tag_name().namespace();
            let prefix = namespace.and_then(|uri| node.lookup_prefix(uri));
            let name = qualified_name(prefix, node.tag_name().name());
            let mut start = BytesStart::new(name.clone());
            declare_namespace(
                &mut start,
                namespaces,
                prefix,
                namespace.unwrap_or_default(),
            );

            for a in node.attributes() {
                // Default namespaces do not apply to attributes. Prefer an
                // explicit prefix, falling back to the implicit xml prefix.
                let prefix = a.namespace().and_then(|uri| {
                    node.namespaces()
                        .find(|namespace| namespace.name().is_some() && namespace.uri() == uri)
                        .and_then(|namespace| namespace.name())
                        .or_else(|| node.lookup_prefix(uri))
                });
                if let Some(uri) = a.namespace() {
                    declare_namespace(&mut start, namespaces, prefix, uri);
                }
                let name = qualified_name(prefix, a.name());
                // roxmltree has already unescaped the value. Escape it again,
                // including character references for attribute whitespace.
                start.push_attribute((name.as_bytes(), escape(a.value()).as_bytes()));
            }
            w.write_event(Event::Start(start))?;

            for child in node.children() {
                write_node(w, child, namespaces)?;
            }

            let end = BytesEnd::new(name);
            w.write_event(Event::End(end))?;
            namespaces.truncate(scope_start);
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

#[cfg(test)]
mod tests {
    use super::indent;

    use stencila_codec::eyre::Result;

    fn assert_mixed_content_unchanged(xml: &str) -> Result<()> {
        let pretty = indent(xml)?;
        assert_eq!(pretty, xml);

        let compact = roxmltree::Document::parse(xml)?;
        let pretty = roxmltree::Document::parse(&pretty)?;
        assert_eq!(compact.root_element().text(), pretty.root_element().text());
        Ok(())
    }

    #[test]
    fn preserves_spaces_around_italic() -> Result<()> {
        assert_mixed_content_unchanged("<p>text <italic>x</italic> text</p>")
    }

    #[test]
    fn preserves_spaces_around_xref() -> Result<()> {
        assert_mixed_content_unchanged("<p>text <xref>1</xref> text</p>")
    }

    #[test]
    fn preserves_spaces_between_adjacent_inline_elements() -> Result<()> {
        assert_mixed_content_unchanged("<p><italic>x</italic> <bold>y</bold></p>")
    }

    #[test]
    fn preserves_punctuation_adjacent_to_inline_elements() -> Result<()> {
        assert_mixed_content_unchanged("<p>text (<italic>x</italic>), text.</p>")
    }

    #[test]
    fn preserves_nested_inline_markup() -> Result<()> {
        assert_mixed_content_unchanged(
            "<p>text <bold>bold <italic>and italic</italic></bold> text</p>",
        )
    }
}
