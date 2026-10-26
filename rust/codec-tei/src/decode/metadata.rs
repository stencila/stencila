//! Page coordinates carried by TEI elements

use roxmltree::Node;

use stencila_codec::stencila_schema::{
    Primitive, PropertyValue, PropertyValueOptions, PropertyValueOrString,
};

/// Keep the original coordinate string, including multiple boxes and whitespace.
///
/// A name identifies a part of a metadata node (e.g. `persName`), or the ID of
/// a content node when the identifier is stored on the article.
pub(super) fn coordinates(node: &Node, name: Option<&str>) -> Option<Vec<PropertyValueOrString>> {
    let coords = node.attribute("coords")?;
    if coords.trim().is_empty() {
        return None;
    }

    Some(vec![PropertyValueOrString::PropertyValue(PropertyValue {
        property_id: Some("coords".into()),
        value: Primitive::String(coords.into()),
        options: Box::new(PropertyValueOptions {
            name: name.map(String::from),
            ..Default::default()
        }),
        ..Default::default()
    })])
}

/// Append optional identifiers without creating an empty list.
pub(super) fn append(
    identifiers: &mut Option<Vec<PropertyValueOrString>>,
    more: Option<Vec<PropertyValueOrString>>,
) {
    if let Some(mut more) = more {
        identifiers.get_or_insert_default().append(&mut more);
    }
}
