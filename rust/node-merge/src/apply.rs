//! Applying a plan to the left document
//!
//! Containers are rewritten whole rather than spliced. Each one is read back out of
//! the working document, rebuilt from its original items and the edits planned against
//! them, and written back with a clear and an append. There is no incremental index
//! arithmetic anywhere, so no edit can shift another out from under itself.
//!
//! Containers are applied deepest-first. Nesting then composes: by the time an outer
//! container reads its items back, any edit inside them is already there, and is
//! carried into whatever wrapper the outer plan builds. This matters because the
//! comparison's paths describe the *input* snapshots and are not promised to survive
//! mutation — applying outermost-first would resolve inner paths against a tree that
//! had already moved.

use stencila_node_path::{NodePath, NodeSlot};
use stencila_schema::{
    Block, Inline, Node, NodeSet, Paragraph, PatchContext, PatchNode, PatchOp, PatchValue,
    SuggestionBlock, SuggestionInline, SuggestionType, Text, get,
};

use stencila_node_compare::Side;

use crate::{
    container::ContainerKind,
    diff::{TextRun, text_runs},
    error::{MergeError, MergeResult},
    options::MergeOptions,
    plan::{ContainerPlan, Edit, MergePlan},
    report::{MergeReport, Unrepresentable, UnrepresentableReason},
};

/// Apply a plan to the working document
pub(crate) fn apply(
    working: &mut Node,
    right: &Node,
    plan: &MergePlan,
    options: &MergeOptions,
    report: &mut MergeReport,
) -> MergeResult<()> {
    // Deepest first, then latest first, so that an inner container is rewritten
    // before the outer one reads it back
    let mut plans: Vec<&ContainerPlan> = plan.containers.values().collect();
    plans.sort_by(|first, second| {
        second
            .container
            .len()
            .cmp(&first.container.len())
            .then_with(|| second.container.cmp(&first.container))
    });

    let mut ids = Ids::new(&options.id_prefix);

    for container in plans {
        match container.kind {
            ContainerKind::Blocks => {
                rebuild::<Block>(working, right, container, options, &mut ids, report)?;
            }
            ContainerKind::Inlines => {
                rebuild::<Inline>(working, right, container, options, &mut ids, report)?;
            }
            // Never planned: a container that holds neither is reported as
            // unrepresentable rather than being given edits
            ContainerKind::Other(..) => continue,
        }
    }

    Ok(())
}

/// Whether a container is there at all
///
/// An optional property that is `None` is not an empty container: it has to be created
/// before anything can be appended to it, and clearing it is an error rather than a
/// no-op. The two are told apart by what reading it back gives — see [`read`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Presence {
    Present,
    Absent,
}

/// Read a container, rebuild it with its edits, and write it back
fn rebuild<T: Content>(
    working: &mut Node,
    right: &Node,
    plan: &ContainerPlan,
    options: &MergeOptions,
    ids: &mut Ids,
    report: &mut MergeReport,
) -> MergeResult<()> {
    let (items, presence) = read::<T>(working, &plan.container, Side::Left)?;

    if presence == Presence::Absent {
        // The insertion still goes in, and accepting it still yields the right
        // document. Rejecting it leaves the property present but empty rather than
        // absent, which is a difference from the left document, so it is recorded.
        if let Some(NodeSlot::Property(property)) = plan.container.back() {
            report.unrepresentable.push(Unrepresentable {
                left: None,
                right: None,
                reason: UnrepresentableReason::ContainerAbsentOnLeft {
                    property: *property,
                },
            });
        }
    }

    let mut rebuilt: Vec<T> = Vec::with_capacity(items.len() + plan.edits.len());
    let mut cursor = 0usize;

    for edit in &plan.edits {
        let (start, end) = match edit {
            Edit::Insert { at, .. } => (*at, *at),
            Edit::ReplaceText { at, .. } => (*at, *at + 1),
            Edit::Delete { range } | Edit::Replace { range, .. } => (range.start, range.end),
        };

        // Everything before the edit is carried over untouched
        while cursor < start && cursor < items.len() {
            rebuilt.push(items[cursor].clone());
            cursor += 1;
        }

        match edit {
            Edit::Insert { sources, .. } => {
                let content = collect::<T>(right, sources)?;
                rebuilt.push(T::suggestion(
                    SuggestionType::Insert,
                    content,
                    None,
                    options,
                    ids,
                ));
            }
            Edit::Delete { range } => {
                let content = slice(&items, range.clone());
                rebuilt.push(T::suggestion(
                    SuggestionType::Delete,
                    content,
                    None,
                    options,
                    ids,
                ));
            }
            Edit::Replace { range, sources, .. } => {
                let original = slice(&items, range.clone());
                let content = collect::<T>(right, sources)?;
                rebuilt.push(T::suggestion(
                    SuggestionType::Replace,
                    content,
                    Some(original),
                    options,
                    ids,
                ));
            }
            Edit::ReplaceText { left, right, .. } => {
                T::extend_with_text_runs(&mut rebuilt, left, right, options, ids);
            }
        }

        cursor = cursor.max(end);
    }

    while cursor < items.len() {
        rebuilt.push(items[cursor].clone());
        cursor += 1;
    }

    write(working, &plan.container, rebuilt, presence)
}

/// Read the items of a container out of a document
///
/// A container is always a repeated property, so reading one back always gives
/// `NodeSet::Many` — even when it is empty. The one case that gives `NodeSet::One` is an
/// optional property that is `None`, which probing represents as a null. That is how the
/// two are told apart, and telling them apart matters twice over: an absent container
/// must not be cleared, and its null must not be mistaken for a genuine null item, which
/// is a value an inline is allowed to hold.
fn read<T: Content>(node: &Node, path: &NodePath, side: Side) -> MergeResult<(Vec<T>, Presence)> {
    let set = get(node, path.clone()).map_err(|_| MergeError::PathResolution {
        side,
        path: path.clone(),
    })?;

    let nodes = match set {
        NodeSet::Many(nodes) => nodes,
        NodeSet::One(Node::Null(..)) => return Ok((Vec::new(), Presence::Absent)),
        NodeSet::One(node) => vec![node],
    };

    nodes
        .into_iter()
        .enumerate()
        .map(|(index, node)| {
            let actual = node.node_type();
            T::from_node(node).ok_or(MergeError::UnexpectedValue {
                side,
                path: path.clone(),
                index,
                actual,
                expected: T::NAME,
            })
        })
        .collect::<MergeResult<Vec<T>>>()
        .map(|items| (items, Presence::Present))
}

/// Read the occurrences at a set of paths out of a document
fn collect<T: Content>(node: &Node, paths: &[NodePath]) -> MergeResult<Vec<T>> {
    let mut items = Vec::with_capacity(paths.len());
    for path in paths {
        let one = get(node, path.clone()).map_err(|_| MergeError::PathResolution {
            side: Side::Right,
            path: path.clone(),
        })?;
        let nodes = match one {
            NodeSet::One(node) => vec![node],
            NodeSet::Many(nodes) => nodes,
        };
        for (index, node) in nodes.into_iter().enumerate() {
            let actual = node.node_type();
            items.push(T::from_node(node).ok_or(MergeError::UnexpectedValue {
                side: Side::Right,
                path: path.clone(),
                index,
                actual,
                expected: T::NAME,
            })?);
        }
    }
    Ok(items)
}

/// Replace the items of a container
///
/// An absent container is appended to without being cleared: appending to an optional
/// property that is `None` creates it, whereas clearing one is an error.
fn write<T: Content>(
    working: &mut Node,
    path: &NodePath,
    items: Vec<T>,
    presence: Presence,
) -> MergeResult<()> {
    let mut context = PatchContext::default();

    let append = PatchOp::Append(items.into_iter().map(T::into_patch_value).collect());
    let ops = match presence {
        Presence::Present => vec![PatchOp::Clear, append],
        Presence::Absent => vec![append],
    };

    for op in ops {
        let mut at = path.clone();
        working
            .apply(&mut at, op, &mut context)
            .map_err(|error| MergeError::Apply {
                path: path.clone(),
                message: error.to_string(),
            })?;
    }

    Ok(())
}

/// A copy of a range of items, clamped to what the container actually holds
fn slice<T: Clone>(items: &[T], range: std::ops::Range<usize>) -> Vec<T> {
    let start = range.start.min(items.len());
    let end = range.end.min(items.len());
    items[start..end].to_vec()
}

/// Sequential identifiers for the suggestions of one merge
///
/// Allocated in application order, which is deterministic, rather than randomly, so
/// that merging the same two documents twice produces the same document.
pub(crate) struct Ids {
    prefix: String,
    next: usize,
}

impl Ids {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            next: 0,
        }
    }

    fn next(&mut self) -> String {
        let id = format!("{}{}", self.prefix, self.next);
        self.next += 1;
        id
    }
}

/// Content that a suggestion can wrap
///
/// Implemented for `Block` and `Inline`, the two unions that have a suggestion variant.
/// Written as a trait so that rebuilding a container is one function rather than two
/// near-identical ones.
pub(crate) trait Content: Clone + Sized {
    /// What to call the type when a value turns out not to be one
    const NAME: &'static str;

    /// Narrow a node to this kind of content
    fn from_node(node: Node) -> Option<Self>;

    /// Wrap the content in a suggestion of this kind
    fn suggestion(
        suggestion_type: SuggestionType,
        content: Vec<Self>,
        original: Option<Vec<Self>>,
        options: &MergeOptions,
        ids: &mut Ids,
    ) -> Self;

    /// Carry the value into a patch operation
    fn into_patch_value(self) -> PatchValue;

    /// Append the text of one occurrence, with only the runs that differ marked
    ///
    /// Only inline content can be split this way, so the block implementation stands in
    /// for it by marking the text whole. Splitting is what keeps a two-character
    /// correction from reading as a rewrite of the paragraph around it.
    fn extend_with_text_runs(
        into: &mut Vec<Self>,
        left: &str,
        right: &str,
        options: &MergeOptions,
        ids: &mut Ids,
    );
}

impl Content for Block {
    const NAME: &'static str = "Block";

    fn from_node(node: Node) -> Option<Self> {
        Block::try_from(node).ok()
    }

    fn suggestion(
        suggestion_type: SuggestionType,
        content: Vec<Self>,
        original: Option<Vec<Self>>,
        options: &MergeOptions,
        ids: &mut Ids,
    ) -> Self {
        Block::SuggestionBlock(SuggestionBlock {
            id: Some(ids.next()),
            suggestion_type: Some(suggestion_type),
            suggestion_status: options.suggestion_status,
            authors: options.authors.clone(),
            content,
            original,
            ..Default::default()
        })
    }

    fn into_patch_value(self) -> PatchValue {
        PatchValue::Block(self)
    }

    fn extend_with_text_runs(
        into: &mut Vec<Self>,
        left: &str,
        right: &str,
        options: &MergeOptions,
        ids: &mut Ids,
    ) {
        // A block container never holds a `Text`, so this is unreachable in practice;
        // marking the whole of it keeps the fallback honest rather than silent.
        into.push(Block::SuggestionBlock(SuggestionBlock {
            id: Some(ids.next()),
            suggestion_type: Some(SuggestionType::Replace),
            suggestion_status: options.suggestion_status,
            authors: options.authors.clone(),
            content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                Text::from(right),
            )]))],
            original: Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                Text::from(left),
            )]))]),
            ..Default::default()
        }));
    }
}

impl Content for Inline {
    const NAME: &'static str = "Inline";

    fn from_node(node: Node) -> Option<Self> {
        Inline::try_from(node).ok()
    }

    fn suggestion(
        suggestion_type: SuggestionType,
        content: Vec<Self>,
        original: Option<Vec<Self>>,
        options: &MergeOptions,
        ids: &mut Ids,
    ) -> Self {
        Inline::SuggestionInline(SuggestionInline {
            id: Some(ids.next()),
            suggestion_type: Some(suggestion_type),
            suggestion_status: options.suggestion_status,
            authors: options.authors.clone(),
            content,
            original,
            ..Default::default()
        })
    }

    fn into_patch_value(self) -> PatchValue {
        PatchValue::Inline(self)
    }

    fn extend_with_text_runs(
        into: &mut Vec<Self>,
        left: &str,
        right: &str,
        options: &MergeOptions,
        ids: &mut Ids,
    ) {
        for run in text_runs(left, right) {
            match run {
                TextRun::Unchanged(text) => into.push(Inline::Text(Text::from(text.as_str()))),
                TextRun::Changed { before, after } => {
                    let (suggestion_type, content, original) =
                        match (before.is_empty(), after.is_empty()) {
                            (true, false) => (
                                SuggestionType::Insert,
                                vec![Inline::Text(Text::from(after.as_str()))],
                                None,
                            ),
                            (false, true) => (
                                SuggestionType::Delete,
                                vec![Inline::Text(Text::from(before.as_str()))],
                                None,
                            ),
                            _ => (
                                SuggestionType::Replace,
                                vec![Inline::Text(Text::from(after.as_str()))],
                                Some(vec![Inline::Text(Text::from(before.as_str()))]),
                            ),
                        };

                    into.push(Inline::SuggestionInline(SuggestionInline {
                        id: Some(ids.next()),
                        suggestion_type: Some(suggestion_type),
                        suggestion_status: options.suggestion_status,
                        authors: options.authors.clone(),
                        content,
                        original,
                        ..Default::default()
                    }));
                }
            }
        }
    }
}
