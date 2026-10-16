//! Keeping the left document's labels when right content is copied into it
//!
//! Comparison treats IDs as document-local labels: it never reports a renamed ID, and it
//! compares citation and internal link targets through the alignment rather than by
//! spelling. So a merge never proposes a rename, and the left document keeps its own
//! labels. Content copied from the right document, though, carries the right
//! document's labels — a link to `#new`, a citation of `b13` — which, copied verbatim,
//! would point at nothing once accepted.
//!
//! So before anything is copied, the copied parts of the right document are translated
//! into the left document's scheme. A node paired with an identified left node takes
//! that node's ID. A node with no identified counterpart keeps its own ID, or a fresh
//! one where that is already taken on the left. A cross-reference is rewritten to
//! whatever its target is called on the left, and a paired left node that has no ID
//! but is referenced by copied content is given one.
//!
//! Resolution mirrors comparison: citations resolve against reference IDs, fragment
//! links against any ID, IDs within a citation's cached copy of a reference are not
//! addressable, and an ID duplicated within either namespace resolves to nothing in it. Anything unresolved on the right
//! is copied unchanged, since it pointed at nothing there either.

use std::collections::{BTreeMap, BTreeSet};

use stencila_node_compare::{Comparison, Correspondence, NodeRef};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{Node, NodeSet, PatchContext, PatchNode, PatchOp, PatchValue, get};

use crate::{
    comments::{existing_id, identifiers, identify},
    error::{MergeError, MergeResult},
    index::Index,
    plan::{Edit, MergePlan},
};

/// Translate the parts of the right document that the plan copies
///
/// Returns the right document with its copied labels rewritten, and gives an ID to
/// each left node that copied content refers to but that had none. Paths are
/// unchanged, since only scalar properties are rewritten, so the plan still addresses
/// the translated document.
pub(crate) fn translate(
    working: &mut Node,
    right: &Node,
    comparison: &Comparison,
    plan: &MergePlan,
) -> MergeResult<Node> {
    let sources: Vec<&NodePath> = plan
        .containers
        .values()
        .flat_map(|container| &container.edits)
        .flat_map(|edit| match edit {
            Edit::Insert { sources, .. } | Edit::Replace { sources, .. } => sources.as_slice(),
            Edit::Delete { .. } | Edit::ReplaceText { .. } => &[],
        })
        .collect();
    if sources.is_empty() {
        return Ok(right.clone());
    }

    let index = Index::collect(comparison.alignment(), comparison.filter());
    let right_nodes: Vec<&NodeRef> = comparison
        .alignment()
        .correspondences()
        .iter()
        .filter_map(|correspondence| match correspondence {
            Correspondence::Paired { right, .. } | Correspondence::RightOnly { right, .. } => {
                Some(right)
            }
            Correspondence::LeftOnly { .. } => None,
        })
        .collect();

    // The addressable IDs of the right document, and separately those of its
    // references, `None` where one is duplicated within its namespace
    let mut ids: BTreeMap<String, Option<&NodeRef>> = BTreeMap::new();
    let mut references: BTreeMap<String, Option<&NodeRef>> = BTreeMap::new();
    for node in &right_nodes {
        if is_cached(&node.path) {
            continue;
        }
        if let Some(id) = existing_id(right, &node.path) {
            if node.node_type == NodeType::Reference {
                references
                    .entry(id.clone())
                    .and_modify(|node| *node = None)
                    .or_insert(Some(node));
            }
            ids.entry(id)
                .and_modify(|node| *node = None)
                .or_insert(Some(node));
        }
    }
    // The namespace that a label is resolved in, by the type of the node using it
    let namespace = |node_type: NodeType| match node_type {
        NodeType::Citation | NodeType::Reference => &references,
        _ => &ids,
    };
    let resolve = |label: &str, node_type: NodeType| -> Option<&NodeRef> {
        namespace(node_type).get(label).copied().flatten()
    };

    let mut labels = Labels {
        used: identifiers(working),
        translated: BTreeMap::new(),
        assignments: BTreeMap::new(),
    };
    let mut translated = right.clone();

    for node in right_nodes
        .iter()
        .filter(|node| sources.iter().any(|source| starts_with(&node.path, source)))
    {
        // The node's own ID, which a cached copy shares with its original. A reference
        // is looked up as citations look it up, so that the two are translated alike.
        if let Some(id) = existing_id(right, &node.path)
            && let Some(target) = resolve(&id, node.node_type)
        {
            let label = labels.label(&id, counterpart(working, &index, target));
            if label != id {
                set(&mut translated, &node.path, NodeProperty::Id, &label)?;
            }
        }

        // The node's cross-reference, if it is one that resolves
        let rewritten = match node.node_type {
            NodeType::Citation => {
                string(right, &node.path, NodeProperty::Target).and_then(|target| {
                    let label = target.trim();
                    let referenced = resolve(label, NodeType::Citation)?;
                    let translated =
                        labels.reference(label, counterpart(working, &index, referenced));
                    (translated != label).then_some(translated)
                })
            }
            NodeType::Link => string(right, &node.path, NodeProperty::Target).and_then(|target| {
                let label = target.strip_prefix('#')?;
                let referenced = resolve(label, NodeType::Link)?;
                let translated = labels.reference(label, counterpart(working, &index, referenced));
                (translated != label).then(|| format!("#{translated}"))
            }),
            _ => None,
        };
        if let Some(target) = rewritten {
            set(&mut translated, &node.path, NodeProperty::Target, &target)?;
        }
    }

    for (path, id) in labels.assignments {
        identify(working, &path, &id)?;
    }

    Ok(translated)
}

/// What a right node is called in the left document
enum Counterpart {
    /// It is paired with a left node that has this ID
    Identified(String),

    /// It is paired with a left node that has no ID
    Unidentified(NodePath),

    /// It exists only on the right
    Unpaired,
}

/// The counterpart of a right node
fn counterpart(working: &Node, index: &Index, node: &NodeRef) -> Counterpart {
    match index.left_of(&node.path) {
        Some(left) => match existing_id(working, left) {
            Some(id) => Counterpart::Identified(id),
            None => Counterpart::Unidentified(left.clone()),
        },
        None => Counterpart::Unpaired,
    }
}

/// The labels allocated while translating
struct Labels {
    /// Every ID in the left document, and every ID allocated since
    used: BTreeSet<String>,

    /// Right labels already translated, so each is allocated once
    translated: BTreeMap<String, String>,

    /// IDs to give left nodes that copied content refers to
    assignments: BTreeMap<NodePath, String>,
}

impl Labels {
    /// The left label for a right one
    fn label(&mut self, right: &str, counterpart: Counterpart) -> String {
        if let Counterpart::Identified(left) = counterpart {
            return left;
        }
        if let Some(label) = self.translated.get(right) {
            return label.clone();
        }

        // Keep the right label where the left document does not use it, otherwise
        // derive a fresh one from it
        let mut label = right.to_string();
        let mut suffix = 2;
        while !self.used.insert(label.clone()) {
            label = format!("{right}-{suffix}");
            suffix += 1;
        }
        self.translated.insert(right.to_string(), label.clone());
        label
    }

    /// The left label for a right one that copied content refers to
    ///
    /// A paired left node without an ID needs one for the reference to resolve.
    fn reference(&mut self, right: &str, counterpart: Counterpart) -> String {
        let unidentified = match &counterpart {
            Counterpart::Unidentified(path) => Some(path.clone()),
            _ => None,
        };
        let label = self.label(right, counterpart);
        if let Some(path) = unidentified {
            self.assignments.insert(path, label.clone());
        }
        label
    }
}

/// Whether a path lies within a citation's cached copy of a reference
fn is_cached(path: &NodePath) -> bool {
    path.iter()
        .any(|slot| *slot == NodeSlot::Property(NodeProperty::Cites))
}

/// Whether a path lies at or below another
fn starts_with(path: &NodePath, prefix: &NodePath) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path.iter()).all(|(a, b)| a == b)
}

/// A string property of the occurrence at a path
fn string(node: &Node, path: &NodePath, property: NodeProperty) -> Option<String> {
    let mut path = path.clone();
    path.push_back(NodeSlot::Property(property));
    match get(node, path).ok()? {
        NodeSet::One(Node::String(value)) => Some(value),
        _ => None,
    }
}

/// Set a string property of the occurrence at a path
fn set(node: &mut Node, path: &NodePath, property: NodeProperty, value: &str) -> MergeResult<()> {
    let mut path = path.clone();
    path.push_back(NodeSlot::Property(property));
    let mut at = path.clone();
    node.apply(
        &mut at,
        PatchOp::Set(PatchValue::String(value.to_string())),
        &mut PatchContext::default(),
    )
    .map_err(|error| MergeError::Apply {
        path,
        message: error.to_string(),
    })
}
