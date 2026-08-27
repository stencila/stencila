//! What a property slot accepts, and where a suggestion can therefore go
//!
//! `SuggestionBlock` is a `Block` and `SuggestionInline` is an `Inline`, so whether a
//! difference can be carried by a suggestion is a question about the *container*, not
//! about the node that changed. A right-only paragraph inside `Article.content` can be
//! wrapped; the same paragraph inside a `ListItem` cannot, because `List.items` holds
//! list items and a suggestion is not one.
//!
//! The answer comes from the schema rather than from a hand-maintained list of types:
//! [`stencila_schema::inspect_declared_properties`] reports, for every property, the
//! name of the schema type filling its slot. That is also what separates content from
//! metadata: a slot of `Cord` is text a person typed and a format round-trips as
//! content, and everything else the schema declares as a scalar is metadata about it.

use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{PropertyDecl, inspect_declared_properties};

use crate::index::{path_from, split_container};

/// What a container holds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContainerKind {
    /// A sequence of blocks, which accepts a `SuggestionBlock`
    Blocks,

    /// A sequence of inlines, which accepts a `SuggestionInline` and a `Boundary`
    Inlines,

    /// Anything else, which accepts neither
    Other(&'static str),
}

impl ContainerKind {
    /// Whether the container accepts a suggestion
    pub fn is_content(self) -> bool {
        matches!(self, Self::Blocks | Self::Inlines)
    }
}

/// What the schema declares about a property of a node type
///
/// `None` when the type does not declare the property at all, which happens on a
/// cross-type pair: the comparison takes the union of both types' properties, so a
/// property reported on the pair may belong only to the other side.
pub(crate) fn declaration_of(node_type: NodeType, property: NodeProperty) -> Option<PropertyDecl> {
    inspect_declared_properties(node_type)?
        .into_iter()
        .find(|declaration| declaration.property == property)
}

/// The kind of value a property of a node type holds
pub(crate) fn slot_of(node_type: NodeType, property: NodeProperty) -> Option<&'static str> {
    declaration_of(node_type, property).map(|declaration| declaration.slot)
}

/// What the container at a path holds
///
/// `owner` is the node type of the occurrence that owns the container, and the path
/// is the container's own, ending in the property slot.
pub(crate) fn container_kind(owner: NodeType, container: &NodePath) -> ContainerKind {
    let Some(NodeSlot::Property(property)) = container.back() else {
        return ContainerKind::Other("");
    };

    let Some(declaration) = declaration_of(owner, *property) else {
        return ContainerKind::Other("");
    };

    // A container is a *sequence*. A property that holds a single block or inline has
    // no positions to insert between, and reading it back as though it were a vector
    // would be wrong rather than merely unsupported.
    if !declaration.repeated {
        return ContainerKind::Other(declaration.slot);
    }

    match declaration.slot {
        "Block" => ContainerKind::Blocks,
        "Inline" => ContainerKind::Inlines,
        slot => ContainerKind::Other(slot),
    }
}

/// Whether a property holds text that a suggestion can replace
///
/// True exactly for a `Cord`, which is the schema's type for text a person typed:
/// `Text.value`, `CodeBlock.code`, `MathInline.code`. Every other scalar the schema
/// declares — a string identifier, a heading level, an enum, a date — describes the
/// content rather than being it.
///
/// This cannot be decided from the compared value: the comparison projects a `Cord`
/// down to its string, so by the time a difference exists the distinction is gone and
/// has to be recovered from the schema.
pub(crate) fn is_content_value(node_type: NodeType, property: NodeProperty) -> bool {
    slot_of(node_type, property) == Some("Cord")
}

/// The nearest ancestor-or-self that sits in a block or inline container
///
/// A change to a property has to be carried by replacing some whole node, and the
/// smallest one that a suggestion can wrap is the nearest enclosing occurrence held in
/// a content container. `owner_of` resolves the node type at each candidate path.
///
/// `None` when the walk reaches the root through metadata only — a change under
/// `authors` or `references` has no content ancestor, and so no suggestion can carry
/// it at all.
pub(crate) fn content_ancestor(
    path: &NodePath,
    owner_of: impl Fn(&NodePath) -> Option<NodeType>,
) -> Option<(NodePath, usize, ContainerKind)> {
    let mut candidate = path.clone();

    loop {
        if let Some((container, index)) = split_container(&candidate) {
            // The owner of the container is the occurrence the container hangs off,
            // which is the container path with its property slots removed
            let owner_path = owner_path_of(&container);
            if let Some(owner) = owner_of(&owner_path) {
                let kind = container_kind(owner, &container);
                if kind.is_content() {
                    return Some((container, index, kind));
                }
            }
        }

        if candidate.pop_back().is_none() || candidate.is_empty() {
            return None;
        }
    }
}

/// The path of the occurrence that owns a container
///
/// A container path ends in one or more property slots hanging off an occurrence, so
/// the owner is what remains once those are dropped.
pub(crate) fn owner_path_of(container: &NodePath) -> NodePath {
    let owner_len = container
        .iter()
        .rposition(|slot| matches!(slot, NodeSlot::Index(..)))
        .map_or(0, |position| position + 1);
    path_from(container.iter().take(owner_len).cloned())
}
