//! Turning a comparison into a set of edits
//!
//! The unit of work is a *container* — one `Vec<Block>` or `Vec<Inline>` — rather than
//! an individual edit. Every edit a merge can make is a rewrite of some container, and
//! deciding a container's edits together, against the left indices it had before
//! anything moved, is what keeps the indices meaningful. Applying them is then a
//! rebuild of the whole vector rather than a splice, so no edit can invalidate
//! another's position.
//!
//! Differences are orthogonal: one pair may be cross-type, reordered and value-changed
//! at once. So a difference is never turned straight into an edit. It is turned into a
//! *request*, requests are gathered per container, and only then are they resolved
//! into edits — which is where a whole-node replacement is allowed to subsume the
//! smaller edits inside it.

use std::{collections::BTreeMap, ops::Range};

use stencila_node_compare::{
    Comparison, Difference, NodeRef, PropertyPresence, ScalarValue, ValueState,
};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};

use crate::{
    container::{ContainerKind, container_kind, content_ancestor, is_content_value, owner_path_of},
    index::{Index, split_container},
    options::{CommentMode, EditCoalescing, MergeOptions, MetadataChanges},
    report::{MergeReport, Unrepresentable, UnrepresentableReason},
};

/// An edit to one container, in terms of the left indices it had before any edit
#[derive(Debug, Clone)]
pub(crate) enum Edit {
    /// Wrap left items in a deletion suggestion
    Delete { range: Range<usize> },

    /// Insert right content as an insertion suggestion
    Insert { at: usize, sources: Vec<NodePath> },

    /// Replace the text of one occurrence, marking only the runs that differ
    ///
    /// Text is the one place where a difference is finer than a node. A `Cord` holds a
    /// whole paragraph's worth of prose, so marking the node it belongs to would report
    /// a two-character correction as a rewrite of everything around it.
    ReplaceText {
        at: usize,
        left: String,
        right: String,
    },

    /// Wrap left items in a replacement suggestion holding right content
    Replace {
        range: Range<usize>,
        sources: Vec<NodePath>,

        /// Why this replacement was planned
        purpose: ReplacementPurpose,
    },
}

/// Why a whole occurrence is replaced
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplacementPurpose {
    /// Its content differs
    Content,

    /// One of its non-content properties differs
    Metadata,
}

impl Edit {
    /// The left index the edit starts at, used to order edits within a container
    fn start(&self) -> usize {
        match self {
            Self::Delete { range } | Self::Replace { range, .. } => range.start,
            Self::Insert { at, .. } | Self::ReplaceText { at, .. } => *at,
        }
    }

    /// Whether the edit inserts, which orders it before an edit at the same index
    ///
    /// An insertion at index `i` goes before the item at `i`, so when a deletion of
    /// that same item is also planned the insertion is written first.
    fn is_insert(&self) -> bool {
        matches!(self, Self::Insert { .. })
    }
}

/// The edits to one container
#[derive(Debug)]
pub(crate) struct ContainerPlan {
    /// The left path of the container, ending in its property slot
    pub container: NodePath,

    /// What the container holds
    pub kind: ContainerKind,

    /// The edits, ordered by the left index they start at
    pub edits: Vec<Edit>,
}

/// A comment to attach to the merged document
#[derive(Debug)]
pub(crate) struct PendingComment {
    /// The left path of the occurrence the comment is about, when it has one
    pub target: Option<NodePath>,

    /// What the comment says
    pub message: String,
}

/// Everything a merge will do to the left document
#[derive(Debug, Default)]
pub(crate) struct MergePlan {
    /// The containers to rewrite, keyed by their left path
    pub containers: BTreeMap<NodePath, ContainerPlan>,

    /// The comments to attach
    pub comments: Vec<PendingComment>,

    /// What the merge produced, and what it could not express
    pub report: MergeReport,
}

impl MergePlan {
    /// Plan the merge of a comparison
    pub fn build(comparison: &Comparison, options: &MergeOptions) -> Self {
        let alignment = comparison.alignment();
        let index = Index::collect(alignment, comparison.filter());
        let types = NodeTypes::collect(comparison);

        let mut plan = Self::default();

        // One-sided subtrees first, so that the containers they touch already exist
        // when the differences are folded in
        for root in &index.left_only {
            plan.plan_deletion(root.node, &types);
        }
        for root in &index.right_only {
            plan.plan_insertion(root.node, &index, &types);
        }
        for difference in comparison.differences() {
            plan.plan_difference(difference, &index, &types, options);
        }

        plan.finish(options);
        plan
    }

    /// Wrap a left-only subtree in a deletion suggestion
    fn plan_deletion(&mut self, node: &NodeRef, types: &NodeTypes) {
        let Some((container, index)) = split_container(&node.path) else {
            // Held in a singular property rather than a sequence, so there is no
            // sibling position for a suggestion to occupy
            self.unrepresentable(
                Some(node.clone()),
                None,
                UnrepresentableReason::NoContentAncestor { property: None },
            );
            return;
        };

        let kind = types.container_kind(&container);
        if !kind.is_content() {
            // Nothing can wrap it, so a comment is all that is left. Without one the
            // change would be missing from the merged document altogether, which is
            // worse than saying it cannot be shown properly.
            self.comment(
                Some(node.path.clone()),
                only_on_one_side(stencila_node_compare::Side::Left, node, &container, kind),
            );
            self.unrepresentable(
                Some(node.clone()),
                None,
                UnrepresentableReason::NotContentContainer {
                    slot: slot_name(kind),
                },
            );
            return;
        }

        self.entry(container, kind).edits.push(Edit::Delete {
            range: index..index + 1,
        });
    }

    /// Insert a right-only subtree as an insertion suggestion
    fn plan_insertion(&mut self, node: &NodeRef, index: &Index, types: &NodeTypes) {
        let Some(insertion) = index.insertion(&node.path) else {
            // The right parent has no left counterpart, which means the containing
            // property is absent on the left
            let property = property_of(&node.path);
            self.unrepresentable(
                None,
                Some(node.clone()),
                match property {
                    Some(property) => UnrepresentableReason::ContainerAbsentOnLeft { property },
                    None => UnrepresentableReason::NoContentAncestor { property: None },
                },
            );
            return;
        };

        let kind = types.container_kind(&insertion.container);
        if !kind.is_content() {
            // Unanchored: the occurrence is only in the right document, so there is
            // nothing in the left one to attach the comment to
            self.comment(
                None,
                only_on_one_side(
                    stencila_node_compare::Side::Right,
                    node,
                    &insertion.container,
                    kind,
                ),
            );
            self.unrepresentable(
                None,
                Some(node.clone()),
                UnrepresentableReason::NotContentContainer {
                    slot: slot_name(kind),
                },
            );
            return;
        }

        self.entry(insertion.container, kind)
            .edits
            .push(Edit::Insert {
                at: insertion.index,
                sources: vec![node.path.clone()],
            });
    }

    /// Express a difference between two paired occurrences
    fn plan_difference(
        &mut self,
        difference: &Difference,
        index: &Index,
        types: &NodeTypes,
        options: &MergeOptions,
    ) {
        match difference {
            Difference::ValueChanged {
                location,
                left,
                right,
            } => {
                let owns_content = location
                    .property
                    .is_some_and(|property| is_content_value(location.left.node_type, property));

                if owns_content {
                    // Text a person typed. Where the occurrence is a `Text`, the
                    // difference is marked within its string rather than around the
                    // whole of it; anything else — a code block, a maths block — is
                    // replaced as a unit, because a run of its source is not
                    // separately representable.
                    match (location.left.node_type, text_of(left), text_of(right)) {
                        (NodeType::Text, Some(before), Some(after)) => {
                            self.plan_text_replacement(&location.left, before, after, types);
                        }
                        _ => self.plan_replacement(&location.left, &location.right, types),
                    }
                } else {
                    self.plan_metadata_change(
                        &location.left,
                        &location.right,
                        location.property,
                        describe_value_change(location.property, left, right),
                        index,
                        types,
                        options,
                    );
                }
            }

            Difference::NodeTypeChanged { left, right } => self.plan_metadata_change(
                left,
                right,
                None,
                format!(
                    "Node type changed from `{}` to `{}`",
                    left.node_type, right.node_type
                ),
                index,
                types,
                options,
            ),

            Difference::PropertyPresenceChanged {
                left,
                right,
                property,
                left_presence,
                right_presence,
            } => self.plan_metadata_change(
                left,
                right,
                Some(*property),
                format!(
                    "Property `{property}` is {} on the left and {} on the right",
                    presence(*left_presence),
                    presence(*right_presence)
                ),
                index,
                types,
                options,
            ),

            // A moved or reordered occurrence is paired on both sides, so there is no
            // one-sided content to insert or delete. Expressing it as a deletion plus
            // an insertion would duplicate the content, which is worse than saying so.
            Difference::ParentChanged {
                left,
                right,
                left_parent,
                right_parent,
                ..
            } => {
                self.comment(
                    Some(left.path.clone()),
                    describe_move(left_parent.as_ref(), right_parent.as_ref()),
                );
                self.unrepresentable(
                    Some(left.clone()),
                    Some(right.clone()),
                    UnrepresentableReason::Moved,
                );
            }

            Difference::Reordered {
                left,
                right,
                property,
                ..
            } => {
                self.comment(Some(left.path.clone()), describe_reorder(*property));
                self.unrepresentable(
                    Some(left.clone()),
                    Some(right.clone()),
                    UnrepresentableReason::Reordered,
                );
            }
        }
    }

    /// Mark the runs of a text that differ, leaving the rest of it alone
    fn plan_text_replacement(
        &mut self,
        left: &NodeRef,
        before: String,
        after: String,
        types: &NodeTypes,
    ) {
        let Some((container, index)) = split_container(&left.path) else {
            return;
        };

        // Only an inline container can hold the pieces a split produces
        if types.container_kind(&container) != ContainerKind::Inlines {
            self.plan_replacement(left, left, types);
            return;
        }

        self.entry(container, ContainerKind::Inlines)
            .edits
            .push(Edit::ReplaceText {
                at: index,
                left: before,
                right: after,
            });
    }

    /// Replace one occurrence with its right counterpart
    fn plan_replacement(&mut self, left: &NodeRef, right: &NodeRef, types: &NodeTypes) {
        let Some((container, index)) = split_container(&left.path) else {
            self.unrepresentable(
                Some(left.clone()),
                Some(right.clone()),
                UnrepresentableReason::NoContentAncestor { property: None },
            );
            return;
        };

        let kind = types.container_kind(&container);
        if !kind.is_content() {
            self.unrepresentable(
                Some(left.clone()),
                Some(right.clone()),
                UnrepresentableReason::NotContentContainer {
                    slot: slot_name(kind),
                },
            );
            return;
        }

        self.entry(container, kind).edits.push(Edit::Replace {
            range: index..index + 1,
            sources: vec![right.path.clone()],
            purpose: ReplacementPurpose::Content,
        });
    }

    /// Describe a change that no suggestion can carry directly
    ///
    /// A metadata change is not a substitution of content, so the only way accepting
    /// the merge can reproduce it is to replace the whole occurrence that owns the
    /// property. Where that occurrence sits in a content container, both are emitted:
    /// the comment says what changed, and the replacement carries it.
    #[allow(clippy::too_many_arguments)]
    fn plan_metadata_change(
        &mut self,
        left: &NodeRef,
        right: &NodeRef,
        property: Option<NodeProperty>,
        message: String,
        index: &Index,
        types: &NodeTypes,
        options: &MergeOptions,
    ) {
        self.comment(Some(left.path.clone()), message);

        if options.metadata_changes == MetadataChanges::CommentOnly {
            self.unrepresentable(
                Some(left.clone()),
                Some(right.clone()),
                UnrepresentableReason::NoContentAncestor { property },
            );
            return;
        }

        // The smallest occurrence a suggestion can wrap, which may be an ancestor when
        // the change is to a property of something that is not itself content
        let ancestor = content_ancestor(&left.path, |path| types.left.get(path).copied());

        let Some((container, at, kind)) = ancestor else {
            self.unrepresentable(
                Some(left.clone()),
                Some(right.clone()),
                UnrepresentableReason::NoContentAncestor { property },
            );
            return;
        };

        // The right counterpart of whatever ancestor was chosen, which is what the
        // replacement will hold
        let left_ancestor = with_index(&container, at);
        let Some(right_ancestor) = index.right_of(&left_ancestor) else {
            self.unrepresentable(
                Some(left.clone()),
                Some(right.clone()),
                UnrepresentableReason::NoContentAncestor { property },
            );
            return;
        };

        self.entry(container, kind).edits.push(Edit::Replace {
            range: at..at + 1,
            sources: vec![right_ancestor.clone()],
            purpose: ReplacementPurpose::Metadata,
        });
    }

    /// The plan for a container, creating it if this is its first edit
    fn entry(&mut self, container: NodePath, kind: ContainerKind) -> &mut ContainerPlan {
        self.containers
            .entry(container.clone())
            .or_insert_with(|| ContainerPlan {
                container,
                kind,
                edits: Vec::new(),
            })
    }

    /// Record a comment
    fn comment(&mut self, target: Option<NodePath>, message: String) {
        self.comments.push(PendingComment { target, message });
    }

    /// Record a difference that no suggestion can express
    fn unrepresentable(
        &mut self,
        left: Option<NodeRef>,
        right: Option<NodeRef>,
        reason: UnrepresentableReason,
    ) {
        self.report.unrepresentable.push(Unrepresentable {
            left,
            right,
            reason,
        });
    }

    /// Order the edits, resolve overlaps, and count what the plan will produce
    fn finish(&mut self, options: &MergeOptions) {
        if options.comments == CommentMode::Omit {
            self.comments.clear();
        }

        // A metadata replacement carries the complete right occurrence. Any finer edit
        // inside that occurrence is already represented by it and must be removed;
        // otherwise the inner suggestion would become part of the outer suggestion's
        // original value and rejecting would no longer restore the left document.
        let metadata_replacements: Vec<NodePath> = self
            .containers
            .values()
            .flat_map(|plan| {
                plan.edits.iter().filter_map(|edit| match edit {
                    Edit::Replace {
                        range,
                        purpose: ReplacementPurpose::Metadata,
                        ..
                    } => Some(with_index(&plan.container, range.start)),
                    _ => None,
                })
            })
            .collect();
        self.containers.retain(|container, _| {
            !metadata_replacements
                .iter()
                .any(|occurrence| starts_with(container, occurrence))
        });

        for plan in self.containers.values_mut() {
            // Insertions before the item they precede, so that a replaced item does
            // not swallow content inserted in front of it
            plan.edits
                .sort_by_key(|edit| (edit.start(), !edit.is_insert()));

            subsume(&mut plan.edits);

            if options.coalesce == EditCoalescing::Coalesce {
                coalesce(&mut plan.edits);
            }
        }

        // Containers with every edit subsumed away carry no work
        self.containers.retain(|_, plan| !plan.edits.is_empty());

        self.report.comments = self.comments.len();
        for plan in self.containers.values() {
            for edit in &plan.edits {
                match edit {
                    Edit::Delete { .. } => self.report.deletes += 1,
                    Edit::Insert { .. } => self.report.inserts += 1,
                    Edit::Replace { .. } | Edit::ReplaceText { .. } => self.report.replaces += 1,
                }
            }
        }
    }
}

/// Drop edits that another edit in the same container already covers
///
/// A whole-node replacement substitutes the occurrence outright, so an edit inside the
/// same range would be applied to content that the replacement discards. Dropping it
/// is safe in both directions: accepting yields the right occurrence whole, rejecting
/// yields the left one whole.
fn subsume(edits: &mut Vec<Edit>) {
    let ranges: Vec<Range<usize>> = edits
        .iter()
        .filter_map(|edit| match edit {
            Edit::Replace { range, .. } => Some(range.clone()),
            _ => None,
        })
        .collect();

    let mut seen: Vec<Range<usize>> = Vec::new();
    edits.retain(|edit| match edit {
        Edit::Replace { range, .. } => {
            // A replacement covered by an identical earlier one is a duplicate: two
            // orthogonal differences about the same pair each asked for it
            let duplicate = seen.iter().any(|other| other == range);
            if !duplicate {
                seen.push(range.clone());
            }
            !duplicate
        }
        Edit::Delete { range } => !ranges
            .iter()
            .any(|covered| covered.start <= range.start && range.end <= covered.end),
        Edit::Insert { at, .. } => !ranges
            .iter()
            .any(|covered| covered.start < *at && *at < covered.end),
        // Marks runs within one occurrence, so a replacement of that occurrence
        // covers it
        Edit::ReplaceText { at, .. } => !ranges
            .iter()
            .any(|covered| covered.start <= *at && *at < covered.end),
    });
}

/// Merge adjacent edits into single ones
///
/// A run of deletions immediately followed by a run of insertions is a rewrite, and
/// saying so as one replacement is both what a reader expects and what makes accepting
/// or rejecting that span exact.
fn coalesce(edits: &mut Vec<Edit>) {
    let mut merged: Vec<Edit> = Vec::with_capacity(edits.len());

    for edit in edits.drain(..) {
        match (merged.last_mut(), edit) {
            (Some(Edit::Delete { range: previous }), Edit::Delete { range })
                if previous.end == range.start =>
            {
                previous.end = range.end;
            }
            (
                Some(Edit::Insert {
                    at,
                    sources: previous,
                }),
                Edit::Insert { at: next, sources },
            ) if *at == next => {
                previous.extend(sources);
            }
            (Some(Edit::Delete { range: deleted }), Edit::Insert { at, sources })
                if deleted.end == at =>
            {
                let range = deleted.clone();
                merged.pop();
                merged.push(Edit::Replace {
                    range,
                    sources,
                    purpose: ReplacementPurpose::Content,
                });
            }
            (_, edit) => merged.push(edit),
        }
    }

    *edits = merged;
}

/// The node types at each left path
///
/// Needed to ask the schema what a container holds, which is a question about the type
/// of the occurrence that owns it rather than about the path.
pub(crate) struct NodeTypes {
    pub left: BTreeMap<NodePath, NodeType>,
}

impl NodeTypes {
    fn collect(comparison: &Comparison) -> Self {
        let mut left = BTreeMap::new();

        for correspondence in comparison.alignment().correspondences() {
            use stencila_node_compare::Correspondence::*;
            let node = match correspondence {
                Paired { left: node, .. } | LeftOnly { left: node, .. } => node,
                RightOnly { .. } => continue,
            };
            left.insert(node.path.clone(), node.node_type);
        }

        Self { left }
    }

    /// What the container at a left path holds
    fn container_kind(&self, container: &NodePath) -> ContainerKind {
        let owner = owner_path_of(container);
        match self.left.get(&owner) {
            Some(node_type) => container_kind(*node_type, container),
            None => ContainerKind::Other(""),
        }
    }
}

/// The string a value state holds, when it holds exactly one
fn text_of(state: &ValueState) -> Option<String> {
    match state {
        ValueState::One {
            value: ScalarValue::String { value },
        } => Some(value.clone()),
        _ => None,
    }
}

/// Whether one path lies strictly inside another
fn starts_with(path: &NodePath, prefix: &NodePath) -> bool {
    path.len() > prefix.len()
        && path
            .iter()
            .zip(prefix.iter())
            .all(|(one, other)| one == other)
}

/// A container path with an index appended
fn with_index(container: &NodePath, index: usize) -> NodePath {
    let mut path = container.clone();
    path.push_back(NodeSlot::Index(index));
    path
}

/// The property a path sits under, if any
fn property_of(path: &NodePath) -> Option<NodeProperty> {
    path.iter().rev().find_map(|slot| match slot {
        NodeSlot::Property(property) => Some(*property),
        NodeSlot::Index(..) => None,
    })
}

/// The slot name to report for a container that cannot hold a suggestion
fn slot_name(kind: ContainerKind) -> String {
    match kind {
        ContainerKind::Other(slot) => slot.to_string(),
        ContainerKind::Blocks => "Block".to_string(),
        ContainerKind::Inlines => "Inline".to_string(),
    }
}

/// How to name a property presence in a comment
fn presence(presence: PropertyPresence) -> &'static str {
    match presence {
        PropertyPresence::Undeclared => "undeclared",
        PropertyPresence::Absent => "absent",
        PropertyPresence::Present => "present",
    }
}

/// Describe a changed value
fn describe_value_change(
    property: Option<NodeProperty>,
    left: &ValueState,
    right: &ValueState,
) -> String {
    match property {
        Some(property) => format!(
            "Property `{property}` changed from {} to {}",
            state(left),
            state(right)
        ),
        None => format!("Value changed from {} to {}", state(left), state(right)),
    }
}

/// Describe a moved occurrence
fn describe_move(left_parent: Option<&NodeRef>, right_parent: Option<&NodeRef>) -> String {
    format!(
        "Moved from {} to {}",
        parent(left_parent),
        parent(right_parent)
    )
}

/// Describe an occurrence that is only on one side and cannot be wrapped
///
/// Names the property rather than repeating the node type, which otherwise reads as
/// "a `TableRow` in `TableRow`" and says nothing.
fn only_on_one_side(
    side: stencila_node_compare::Side,
    node: &NodeRef,
    container: &NodePath,
    kind: ContainerKind,
) -> String {
    match container.back() {
        Some(NodeSlot::Property(property)) => format!(
            "Only in the {side} document: a `{node_type}`. The `{property}` property holds \
             `{slot}`s rather than blocks or inlines, so no suggestion can mark this.",
            node_type = node.node_type,
            slot = slot_name(kind),
        ),
        _ => format!(
            "Only in the {side} document: a `{node_type}`, which cannot be marked by a suggestion.",
            node_type = node.node_type,
        ),
    }
}

/// How to name a path in a comment
fn render_path(path: &NodePath) -> String {
    path.iter()
        .map(|slot| slot.to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Describe a reordered occurrence
fn describe_reorder(property: NodeProperty) -> String {
    format!("Moved to a different position within `{property}`")
}

/// How to name a parent in a comment
///
/// Includes the path, not just the node type: a move is very often between two parents
/// of the *same* type — one table row to another, one section to another — and naming
/// only the type would say "from `TableRow` to `TableRow`", which tells a reader
/// nothing at all.
fn parent(node: Option<&NodeRef>) -> String {
    match node {
        Some(node) => format!("`{}` at `{}`", node.node_type, render_path(&node.path)),
        None => "the document root".to_string(),
    }
}

/// How to name a value state in a comment
///
/// A comment is read by a person, so the values are written the way the document
/// writes them rather than the way Rust debug-prints them.
fn state(state: &ValueState) -> String {
    match state {
        ValueState::Absent => "nothing".to_string(),
        ValueState::One { value } => format!("`{}`", scalar(value)),
        ValueState::Many { values } => format!(
            "`[{}]`",
            values.iter().map(scalar).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// How to write a scalar in a comment
fn scalar(value: &ScalarValue) -> String {
    match value {
        ScalarValue::Null => "null".to_string(),
        ScalarValue::Boolean { value } => value.to_string(),
        ScalarValue::Integer { value } => value.to_string(),
        ScalarValue::UnsignedInteger { value } => value.to_string(),
        ScalarValue::Number { value } => value.get().to_string(),
        ScalarValue::String { value } => format!("\"{value}\""),
        ScalarValue::Enum { variant, .. } => variant.clone(),
        ScalarValue::Array { items } => format!(
            "[{}]",
            items.iter().map(scalar).collect::<Vec<_>>().join(", ")
        ),
        ScalarValue::Object { entries } => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, value)| format!("{key}: {}", scalar(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
