//! Resolve cross-reference targets without treating document-local labels as content.

use std::collections::HashMap;

use stencila_node_type::{NodeProperty, NodeType};

use crate::{
    projection::{Item, OccurrenceId, ProjectedProperty, Projection},
    scalar::ScalarValue,
};

/// Whether a property carries identity rather than content for matching purposes.
///
/// Citation targets are identifiers or URLs, not rendered citation text. Internal
/// links likewise carry local labels; external link URLs remain matching evidence.
pub(crate) fn is_identity(node_type: NodeType, property: &ProjectedProperty) -> bool {
    property.decl.property == NodeProperty::Id
        || (property.decl.property == NodeProperty::Target
            && (node_type == NodeType::Citation
                || (node_type == NodeType::Link
                    && string(property).is_some_and(|value| value.starts_with('#')))))
}

/// Unique IDs in a snapshot. Duplicate IDs cannot establish target equivalence.
pub(crate) struct TargetIndex<'projection> {
    nodes: HashMap<&'projection str, Option<OccurrenceId>>,
    references: HashMap<&'projection str, Option<OccurrenceId>>,
}

impl<'projection> TargetIndex<'projection> {
    pub fn new(projection: &'projection Projection) -> Self {
        let mut index = Self {
            nodes: HashMap::new(),
            references: HashMap::new(),
        };
        // `Citation.cites` caches a resolved reference. Neither it nor anything within
        // it is another addressable node, so none of its IDs may make an original
        // ambiguous. Projection is depth first, so the cache is a contiguous run.
        let mut skip_until = 0;
        for (position, occurrence) in projection.occurrences().iter().enumerate() {
            if position < skip_until {
                continue;
            }
            if occurrence.parent_property == Some(NodeProperty::Cites) {
                skip_until = position + usize::try_from(occurrence.subtree_size).unwrap_or(1);
                continue;
            }
            let Some(id) = occurrence
                .properties
                .iter()
                .find(|property| property.decl.property == NodeProperty::Id)
                .and_then(string)
                .filter(|id| !id.is_empty())
            else {
                continue;
            };
            index
                .nodes
                .entry(id)
                .and_modify(|id| *id = None)
                .or_insert(Some(occurrence.id));
            if occurrence.node_type == NodeType::Reference {
                index
                    .references
                    .entry(id)
                    .and_modify(|id| *id = None)
                    .or_insert(Some(occurrence.id));
            }
        }
        index
    }

    /// Resolve citations against reference IDs and fragment-only links against IDs
    /// throughout the document. Relative file paths and external URLs stay literal.
    pub fn resolve(&self, node_type: NodeType, value: &ScalarValue) -> Option<OccurrenceId> {
        let ScalarValue::String { value } = value else {
            return None;
        };
        match node_type {
            NodeType::Citation => self.references.get(value.trim()).copied().flatten(),
            NodeType::Link => self.nodes.get(value.strip_prefix('#')?).copied().flatten(),
            _ => None,
        }
    }
}

fn string(property: &ProjectedProperty) -> Option<&str> {
    match property.items.first() {
        Some(Item::Scalar(ScalarValue::String { value })) => Some(value),
        _ => None,
    }
}
