//! Lookups over a completed alignment
//!
//! The alignment says which occurrences correspond; a merge needs to ask that the
//! other way round, and by position rather than by identity. This module builds the
//! indexes for those questions once, so that planning is a linear pass rather than a
//! repeated scan.
//!
//! Ordered maps throughout, never hashed ones: identifiers for the suggestions and
//! comments are allocated in plan order, so an unstable iteration order would produce
//! a document that differs between runs.

use std::collections::BTreeMap;

use stencila_node_compare::{Alignment, Correspondence, DifferenceFilter, NodeRef, Side};
use stencila_node_path::{NodePath, NodeSlot};

/// The maximal root of a one-sided subtree
///
/// Every structured descendant of a one-sided occurrence is itself one-sided, so
/// reporting each of them would say the same thing many times over. Collapsing to the
/// root is also a precondition for [`Anchors::insertion`]: the parent of a maximal
/// root is always paired, so mapping it to the other side never has to search upward.
pub(crate) struct OneSidedRoot<'comparison> {
    /// The root of the subtree
    pub node: &'comparison NodeRef,

    /// How many occurrences the subtree covers, including the root
    pub occurrences: usize,
}

/// Lookups over the correspondences of an alignment
pub(crate) struct Index<'comparison> {
    /// Paired left paths to their right counterparts
    left_to_right: BTreeMap<&'comparison NodePath, &'comparison NodePath>,

    /// Paired right paths to their left counterparts
    right_to_left: BTreeMap<&'comparison NodePath, &'comparison NodePath>,

    /// The maximal one-sided roots of the left document
    pub left_only: Vec<OneSidedRoot<'comparison>>,

    /// The maximal one-sided roots of the right document
    pub right_only: Vec<OneSidedRoot<'comparison>>,
}

impl<'comparison> Index<'comparison> {
    /// Build the indexes for an alignment
    pub fn collect(alignment: &'comparison Alignment, filter: &DifferenceFilter) -> Self {
        let mut index = Self {
            left_to_right: BTreeMap::new(),
            right_to_left: BTreeMap::new(),
            left_only: Vec::new(),
            right_only: Vec::new(),
        };

        for correspondence in alignment.correspondences() {
            let (side, node, ancestor) = match correspondence {
                Correspondence::Paired { left, right, .. } => {
                    index.left_to_right.insert(&left.path, &right.path);
                    index.right_to_left.insert(&right.path, &left.path);
                    continue;
                }
                Correspondence::LeftOnly {
                    left,
                    nearest_one_sided_ancestor,
                    ..
                } => (Side::Left, left, nearest_one_sided_ancestor),
                Correspondence::RightOnly {
                    right,
                    nearest_one_sided_ancestor,
                    ..
                } => (Side::Right, right, nearest_one_sided_ancestor),
            };

            let roots = match side {
                Side::Left => &mut index.left_only,
                Side::Right => &mut index.right_only,
            };

            // Correspondences are in canonical path order, so on each side the
            // descendants of a root immediately follow it. A record that names an
            // ancestor is therefore folded into the root just pushed.
            match (ancestor, roots.last_mut()) {
                (Some(..), Some(root)) => root.occurrences += 1,
                _ => roots.push(OneSidedRoot {
                    node,
                    occurrences: 1,
                }),
            }
        }

        // Filtering never changes correspondence, but it does change which one-sided
        // subtrees are reported. Merge only those roots that survived the same filter
        // used by the differences axis, so an excluded comparison cannot emit hidden
        // insertions or deletions.
        index.left_only.retain(|root| filter.allows_node(root.node));
        index
            .right_only
            .retain(|root| filter.allows_node(root.node));

        index
    }

    /// The left path paired with a right one, if any
    pub fn left_of(&self, right: &NodePath) -> Option<&'comparison NodePath> {
        self.right_to_left.get(right).copied()
    }

    /// The right path paired with a left one, if any
    pub fn right_of(&self, left: &NodePath) -> Option<&'comparison NodePath> {
        self.left_to_right.get(left).copied()
    }

    /// Where right-only content belongs in the left document
    ///
    /// The right path addresses the right document, so it cannot be used as a
    /// position in the left one. What carries across is the *ordering among
    /// siblings*: the nearest preceding sibling that is paired occupies a known left
    /// index, and the new content goes just after it.
    ///
    /// Returns the left container and the index within it, or `None` when the right
    /// parent has no left counterpart, which happens when the containing property is
    /// absent on the left.
    pub fn insertion(&self, right: &NodePath) -> Option<Insertion> {
        let (container, index) = split_container(right)?;

        // The property slots between the owning occurrence and the index. A merge
        // only ever inserts into a repeated property, so this is a single slot, but
        // taking it generically keeps the split honest.
        let owner_len = container
            .iter()
            .rposition(|slot| matches!(slot, NodeSlot::Index(..)))
            .map_or(0, |position| position + 1);
        let owner = path_from(container.iter().take(owner_len).cloned());
        let properties: Vec<NodeSlot> = container.iter().skip(owner_len).cloned().collect();

        let left_owner = if owner.is_empty() {
            // The root is always paired with the root
            NodePath::new()
        } else {
            self.left_of(&owner)?.clone()
        };

        let mut left_container = left_owner;
        for slot in properties {
            left_container.push_back(slot);
        }

        // Scan back over the right siblings for the nearest paired one; its left
        // index is the anchor. With none, the content belongs at the start.
        for preceding in (0..index).rev() {
            let mut sibling = container.clone();
            sibling.push_back(NodeSlot::Index(preceding));
            if let Some(left) = self.left_of(&sibling)
                && let Some(NodeSlot::Index(left_index)) = left.back()
            {
                return Some(Insertion {
                    container: left_container,
                    index: left_index + 1,
                });
            }
        }

        Some(Insertion {
            container: left_container,
            index: 0,
        })
    }
}

/// Where right-only content belongs in the left document
pub(crate) struct Insertion {
    /// The left container to insert into
    pub container: NodePath,

    /// The index within that container
    pub index: usize,
}

/// Build a path from a sequence of slots
///
/// `NodePath` does not implement `FromIterator`, and wraps a `VecDeque` whose front
/// is popped while patching, so it is built by pushing onto the back.
pub(crate) fn path_from(slots: impl IntoIterator<Item = NodeSlot>) -> NodePath {
    let mut path = NodePath::new();
    for slot in slots {
        path.push_back(slot);
    }
    path
}

/// Split a path into the container that holds the occurrence and its index in it
///
/// Returns `None` for a path that does not end in an index, which is an occurrence
/// held in a singular property rather than a sequence, and so has no container to
/// insert beside it.
pub(crate) fn split_container(path: &NodePath) -> Option<(NodePath, usize)> {
    match path.back() {
        Some(NodeSlot::Index(index)) => {
            let index = *index;
            let mut container = path.clone();
            container.pop_back();
            Some((container, index))
        }
        _ => None,
    }
}
