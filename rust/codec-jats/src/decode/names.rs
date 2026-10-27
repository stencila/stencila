use roxmltree::Node;

use stencila_codec::stencila_schema::{Person, PersonOptions};
use stencila_codec_utils::split_given_names;

use super::{
    DecodeContext,
    utilities::{extend_path, non_empty_text_deep, record_attrs_lost, record_node_lost},
};

/// Whether an element is one of the JATS elements for the name of a person
pub(super) fn is_name_element(tag: &str) -> bool {
    matches!(tag, "name" | "string-name" | "name-alternatives")
}

/// Decode a `<name>`, `<string-name>` or `<name-alternatives>` to a [`Person`]
///
/// Used for contributors, funding recipients and bibliographic references so
/// that a name is decoded the same way wherever it is written. A `<string-name>`
/// may have the same parts as a `<name>`, or be running text with no parts, in
/// which case the text is the person's `name`. The alternatives of a
/// `<name-alternatives>` are the same name in different forms (e.g. scripts),
/// so the first is used.
///
/// Returns `None` if the element has no name in it.
pub(super) fn decode_person_name(
    path: &str,
    node: &Node,
    losses: &mut DecodeContext,
) -> Option<Person> {
    if node.has_tag_name("name-alternatives") {
        record_attrs_lost(path, node, [], losses);
        let first = node
            .children()
            .find(|child| child.has_tag_name("name") || child.has_tag_name("string-name"))?;
        return decode_person_name(&extend_path(path, first.tag_name().name()), &first, losses);
    }

    // The family and given names are kept apart, so it is only an order other
    // than the usual one that is not represented
    let name_style =
        matches!(node.attribute("name-style"), None | Some("western")).then_some("name-style");
    record_attrs_lost(path, node, name_style, losses);

    let mut family_names = Vec::new();
    let mut given_names = Vec::new();
    for child in node.children() {
        match child.tag_name().name() {
            "surname" => family_names.extend(non_empty_text_deep(&child)),
            "given-names" => {
                if let Some(value) = non_empty_text_deep(&child) {
                    given_names.append(&mut split_given_names(&value));
                }
            }
            // Text between the parts of a `<string-name>` is punctuation
            // unless it has words in it
            _ if child.is_text() => {
                if child
                    .text()
                    .is_some_and(|text| text.chars().any(char::is_alphanumeric))
                    && node.children().any(|child| {
                        child.has_tag_name("surname") || child.has_tag_name("given-names")
                    })
                {
                    record_node_lost(path, &child, losses);
                }
            }
            _ => record_node_lost(path, &child, losses),
        }
    }

    // A name written as running text has no parts to separate
    let name = (family_names.is_empty() && given_names.is_empty())
        .then(|| non_empty_text_deep(node))
        .flatten();

    if family_names.is_empty() && given_names.is_empty() && name.is_none() {
        return None;
    }

    Some(Person {
        family_names: (!family_names.is_empty()).then_some(family_names),
        given_names: (!given_names.is_empty()).then_some(given_names),
        options: Box::new(PersonOptions {
            name,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Decode the name of a `<collab>`
///
/// The name is the collaboration's own text, including any inline formatting,
/// but not the text of the structures that JATS allows within it. The
/// `<contrib-group>`s of its members are returned so that the caller can decode
/// them, and any other structure (e.g. an `<address>` or `<xref>`) is lost.
pub(super) fn decode_collab<'a, 'input>(
    path: &str,
    node: &Node<'a, 'input>,
    losses: &mut DecodeContext,
) -> (Option<String>, Vec<Node<'a, 'input>>) {
    record_attrs_lost(path, node, [], losses);

    fn collect_text(node: &Node, text: &mut String) {
        for child in node.children() {
            if child.is_text() {
                text.push_str(child.text().unwrap_or_default());
            } else if is_collab_name_element(child.tag_name().name()) {
                collect_text(&child, text);
            }
        }
    }

    let mut text = String::new();
    let mut groups = Vec::new();
    for child in node.children() {
        if child.is_text() {
            text.push_str(child.text().unwrap_or_default());
        } else if child.is_element() {
            let tag = child.tag_name().name();
            if tag == "contrib-group" {
                groups.push(child);
            } else if is_collab_name_element(tag) {
                collect_text(&child, &mut text);
            } else {
                record_node_lost(path, &child, losses);
            }
        }
    }

    let name = text.split_whitespace().collect::<Vec<_>>().join(" ");
    ((!name.is_empty()).then_some(name), groups)
}

/// Whether an element within a `<collab>` is part of its name
fn is_collab_name_element(tag: &str) -> bool {
    matches!(
        tag,
        "collab-name"
            | "abbrev"
            | "bold"
            | "italic"
            | "monospace"
            | "named-content"
            | "overline"
            | "roman"
            | "sans-serif"
            | "sc"
            | "styled-content"
            | "sub"
            | "sup"
            | "underline"
    )
}
