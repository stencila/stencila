//! Page coordinates carried by TEI elements

use std::collections::HashSet;

use stencila_codec::{
    PageBox,
    stencila_schema::{InspectNode, InspectValue, NodeId, NodeProperty, ScalarRef},
};

/// Parse the `coords` of an element to boxes on pages
///
/// GROBID and ScienceBeam give the boxes as a semicolon-separated list of
/// `page,x,y,width,height`, in PDF points from the top left of the page.
/// Returns `None` if any box is malformed: pages are numbered from one, and
/// sizes are finite and not negative.
pub(super) fn page_boxes(coords: &str) -> Option<Vec<PageBox>> {
    coords
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut values = part.split(',').map(str::trim);
            let page = values.next()?.parse().ok().filter(|page| *page > 0)?;
            let mut number = || {
                values
                    .next()?
                    .parse::<f64>()
                    .ok()
                    .filter(|number| number.is_finite())
            };
            let (x, y, width, height) = (number()?, number()?, number()?, number()?);
            (values.next().is_none() && width >= 0.0 && height >= 0.0).then_some(PageBox {
                page,
                x,
                y,
                width,
                height,
            })
        })
        .collect()
}

/// Whether a node has an id
///
/// The id of a node is a schema property, unlike its node id, so is found
/// among its properties.
pub(super) fn has_id(node: &dyn InspectNode, id: &str) -> bool {
    node.properties().iter().any(|property| {
        property.decl.property == NodeProperty::Id
            && matches!(&property.value, InspectValue::One(value)
                if matches!(value.scalar(), Some(ScalarRef::String(value)) if value == id))
    })
}

/// Collect the ids (the schema property, not node ids) of all the nodes
/// within a node
///
/// Used to check that the targets of links and citations were kept.
pub(super) fn ids(node: &dyn InspectNode, ids: &mut HashSet<String>) {
    for property in node.properties() {
        match property.value {
            InspectValue::One(value) => {
                if property.decl.property == NodeProperty::Id
                    && let Some(ScalarRef::String(id)) = value.scalar()
                {
                    ids.insert(id.to_string());
                } else {
                    self::ids(value, ids)
                }
            }
            InspectValue::Many(values) => {
                for value in values {
                    self::ids(value, ids)
                }
            }
            InspectValue::Absent => {}
        }
    }
}

/// Collect the ids of all the structured nodes within a node
///
/// Used to remove the page map entries of nodes that were decoded but then
/// dropped, such as empty footnotes.
pub(super) fn node_ids(node: &dyn InspectNode, ids: &mut HashSet<NodeId>) {
    if let Some(id) = node.node_id() {
        ids.insert(id);
    }
    for property in node.properties() {
        match property.value {
            InspectValue::One(value) => node_ids(value, ids),
            InspectValue::Many(values) => {
                for value in values {
                    node_ids(value, ids)
                }
            }
            InspectValue::Absent => {}
        }
    }
}
