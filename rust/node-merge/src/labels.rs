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
//! into the left document's scheme. A node that replaces an identified left node, one
//! the merge deletes, takes that node's ID. Any other node keeps its own ID, or a
//! fresh one where that is already taken on the left: a left counterpart that the
//! merge keeps, such as the original of an unrepresentable move, keeps its ID, and two
//! nodes must not share it. A cross-reference is rewritten to whatever its target is
//! called on the left, and a kept left node that has no ID but is referenced by copied
//! content is given one.
//!
//! Resolution mirrors comparison: citations resolve against reference IDs, fragment
//! links against any ID, IDs within a citation's cached copy of a reference are not
//! addressable, and an ID duplicated within either namespace resolves to nothing in it. Anything unresolved on the right
//! is copied unchanged, since it pointed at nothing there either.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use stencila_node_compare::{Comparison, Correspondence, NodeRef, Side};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    InspectNode, InspectValue, Node, NodeSet, PatchContext, PatchNode, PatchOp, PatchValue, get,
};

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
/// the translated document. The right document is borrowed until a label is rewritten.
pub(crate) fn translate<'right>(
    working: &mut Node,
    right: &'right Node,
    comparison: &Comparison,
    plan: &MergePlan,
) -> MergeResult<Cow<'right, Node>> {
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
        return Ok(Cow::Borrowed(right));
    }

    let index = Index::collect(comparison.alignment(), comparison.filter());

    // The left items that accepting the merge removes. Only a copy of one of these
    // can take over its ID. A text replacement removes its item too, since the runs
    // that replace it do not carry the original's ID.
    let removed: Vec<NodePath> = plan
        .containers
        .values()
        .flat_map(|container| {
            container
                .edits
                .iter()
                .filter_map(|edit| match edit {
                    Edit::Delete { range } | Edit::Replace { range, .. } => Some(range.clone()),
                    Edit::ReplaceText { at, .. } => Some(*at..*at + 1),
                    Edit::Insert { .. } => None,
                })
                .flatten()
                .map(|position| {
                    let mut path = container.container.clone();
                    path.push_back(NodeSlot::Index(position));
                    path
                })
        })
        .collect();
    // The right paragraphs of groups, so that a node within one is found by an
    // ancestor rather than by testing every group
    let grouped: BTreeSet<&NodePath> = comparison
        .alignment()
        .groups()
        .flat_map(|(_, right)| right.nodes())
        .map(|node| &node.path)
        .collect();
    let in_group = |path: &NodePath| {
        if grouped.is_empty() {
            return false;
        }
        let mut path = path.clone();
        while !path.is_empty() {
            if grouped.contains(&path) {
                return true;
            }
            path.pop_back();
        }
        false
    };
    let counterpart = |working: &Node, node: &NodeRef| -> Counterpart {
        let Some(left) = index.left_of(&node.path) else {
            // Group detection preserves every inline ID exactly. These labels
            // survive the block replacement even though inline paths have changed.
            if in_group(&node.path)
                && let Some(id) = existing_id(right, &node.path)
            {
                return Counterpart::Grouped(id);
            }
            return Counterpart::Unpaired;
        };
        let id = existing_id(working, left);
        if removed.iter().any(|root| starts_with(left, root)) {
            Counterpart::Replaced(id)
        } else {
            Counterpart::Kept {
                path: left.clone(),
                id,
            }
        }
    };

    let mut right_nodes = Vec::new();
    for correspondence in comparison.alignment().correspondences() {
        if let Correspondence::ParagraphGroup { right: group, .. } = correspondence {
            for root in group.nodes() {
                let node =
                    get(right, root.path.clone()).map_err(|_| MergeError::PathResolution {
                        side: Side::Right,
                        path: root.path.clone(),
                    })?;
                if let NodeSet::One(node) = node {
                    collect_nodes(&node, &root.path, &mut right_nodes);
                }
            }
        } else {
            right_nodes.extend(correspondence.nodes(Side::Right).cloned());
        }
    }

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
    let mut translated = Cow::Borrowed(right);

    for node in right_nodes
        .iter()
        .filter(|node| sources.iter().any(|source| starts_with(&node.path, source)))
    {
        // The node's own ID. A cached copy of a reference names its original, so it is
        // translated as the citation holding it is. A reference is looked up as
        // citations look it up, so that the two are translated alike.
        if let Some(id) = existing_id(right, &node.path)
            && let Some(target) = resolve(&id, node.node_type)
        {
            let counterpart = counterpart(working, target);
            let label = if is_cached(&node.path) {
                labels.reference(&id, counterpart)
            } else {
                labels.own(&id, counterpart)
            };
            if label != id {
                set(translated.to_mut(), &node.path, NodeProperty::Id, &label)?;
            }
        }

        // The node's cross-reference, if it is one that resolves
        let rewritten = match node.node_type {
            NodeType::Citation => {
                string(right, &node.path, NodeProperty::Target).and_then(|target| {
                    let label = target.trim();
                    let referenced = resolve(label, NodeType::Citation)?;
                    let translated = labels.reference(label, counterpart(working, referenced));
                    (translated != label).then_some(translated)
                })
            }
            NodeType::Link => string(right, &node.path, NodeProperty::Target).and_then(|target| {
                let label = target.strip_prefix('#')?;
                let referenced = resolve(label, NodeType::Link)?;
                let translated = labels.reference(label, counterpart(working, referenced));
                (translated != label).then(|| format!("#{translated}"))
            }),
            _ => None,
        };
        if let Some(target) = rewritten {
            set(
                translated.to_mut(),
                &node.path,
                NodeProperty::Target,
                &target,
            )?;
        }
    }

    for (path, id) in labels.assignments {
        identify(working, &path, &id)?;
    }

    Ok(translated)
}

/// Visit structured nodes whose coverage is implicit in a paragraph group.
fn collect_nodes(node: &dyn InspectNode, path: &NodePath, nodes: &mut Vec<NodeRef>) {
    let Some(node_type) = node.node_type() else {
        return;
    };
    nodes.push(NodeRef::new(path.clone(), node_type));
    for property in node.properties() {
        let mut at = path.clone();
        at.push_back(NodeSlot::Property(property.decl.property));
        match property.value {
            InspectValue::Absent => {}
            InspectValue::One(node) => collect_nodes(node, &at, nodes),
            InspectValue::Many(items) => {
                for (index, node) in items.into_iter().enumerate() {
                    let mut at = at.clone();
                    at.push_back(NodeSlot::Index(index));
                    collect_nodes(node, &at, nodes);
                }
            }
        }
    }
}

/// What a right node corresponds to in the left document
enum Counterpart {
    /// An identical inline ID retained by a paragraph group.
    Grouped(String),
    /// A left node, with its ID if any, that the merge keeps
    Kept { path: NodePath, id: Option<String> },

    /// A left node, with its ID if any, that accepting the merge removes
    Replaced(Option<String>),

    /// Nothing: it exists only on the right
    Unpaired,
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
    /// The left label for the ID of a copied right node
    fn own(&mut self, right: &str, counterpart: Counterpart) -> String {
        match counterpart {
            // The copy takes the place, and so the ID, of the node it replaces
            Counterpart::Replaced(Some(left)) | Counterpart::Grouped(left) => left,
            // A kept counterpart keeps its ID, so the copy needs one of its own
            _ => self.translate(right),
        }
    }

    /// The left label for a right one that copied content refers to
    fn reference(&mut self, right: &str, counterpart: Counterpart) -> String {
        match counterpart {
            Counterpart::Kept { id: Some(left), .. }
            | Counterpart::Replaced(Some(left))
            | Counterpart::Grouped(left) => left,
            // A kept left node without an ID needs one for the reference to resolve,
            // distinct from that of any copy of it
            Counterpart::Kept { path, id: None } => {
                if let Some(label) = self.assignments.get(&path) {
                    return label.clone();
                }
                let label = self.fresh(right);
                self.assignments.insert(path, label.clone());
                label
            }
            Counterpart::Replaced(None) | Counterpart::Unpaired => self.translate(right),
        }
    }

    /// The label for a copied right label, allocated once
    fn translate(&mut self, right: &str) -> String {
        if let Some(label) = self.translated.get(right) {
            return label.clone();
        }
        let label = self.fresh(right);
        self.translated.insert(right.to_string(), label.clone());
        label
    }

    /// The right label where the left document does not use it, otherwise a fresh
    /// one derived from it
    fn fresh(&mut self, right: &str) -> String {
        let mut label = right.to_string();
        let mut suffix = 2;
        while !self.used.insert(label.clone()) {
            label = format!("{right}-{suffix}");
            suffix += 1;
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
