//! Conservative recognition of paragraph merges and splits.
//!
//! A group replaces one local pair and its adjacent gaps (or just gaps). Complete
//! inline content must agree, including formatting and reference targets. Only the
//! space joining paragraphs and segmentation of otherwise plain Text nodes may
//! differ. No prefix, truncated matching feature, or fuzzy score proves a group.

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use stencila_node_type::{NodeProperty, NodeType};

use crate::{
    align::Aligned,
    alignment::{Alignment, Correspondence, ParagraphGroup},
    error::CompareResult,
    projection::{Item, Occurrence, OccurrenceId, Presence, ProjectedProperty, Projection},
    references::TargetIndex,
    scalar::ScalarValue,
    sequence::Step,
};

/// Bound the search to small local edits. Larger groups retain ordinary alignment.
const MAX_PARAGRAPHS: usize = 8;

struct Group {
    /// The positions of the paragraphs within each side's property
    pub left_items: Range<usize>,
    pub right_items: Range<usize>,
    pub left: Vec<OccurrenceId>,
    pub right: Vec<OccurrenceId>,
}

impl Group {
    /// Whether a step aligns one of the group's paragraphs
    fn covers(&self, step: &Step) -> bool {
        match *step {
            Step::Pair { left, right } => {
                self.left_items.contains(&left) || self.right_items.contains(&right)
            }
            Step::LeftGap { left } => self.left_items.contains(&left),
            Step::RightGap { right } => self.right_items.contains(&right),
        }
    }
}

/// Recognize groups only after target identity and cross-parent moves are known.
pub(crate) fn recognize(
    left: &Projection,
    right: &Projection,
    aligned: &mut Aligned,
) -> CompareResult<()> {
    let mut found = Vec::new();
    for (index, property) in aligned.properties.iter().enumerate() {
        if property
            .steps
            .iter()
            .all(|step| matches!(step, Step::Pair { .. }))
        {
            continue;
        }
        let left_property = find_property(left, property.left_parent, property.property)?;
        let right_property = find_property(right, property.right_parent, property.property)?;
        let groups = find(left, right, left_property, right_property, &property.steps)?;
        if !groups.is_empty() {
            found.push((index, groups));
        }
    }
    // Most comparisons have no candidates, so they need none of the lookups below
    if found.is_empty() {
        return Ok(());
    }

    let mut left_covered = HashSet::new();
    let mut right_covered = HashSet::new();
    let mut records = Vec::new();
    let mut inline_pairs = Vec::new();
    let pairing: HashMap<_, _> = aligned.pairs.iter().copied().collect();
    let reverse_pairing: HashMap<_, _> = aligned
        .pairs
        .iter()
        .map(|(left, right)| (*right, *left))
        .collect();
    let left_targets = TargetIndex::new(left);
    let right_targets = TargetIndex::new(right);

    for (index, groups) in found {
        let mut accepted = Vec::new();
        for group in groups {
            let left_ids = descendants(left, &group.left)?;
            let right_ids = descendants(right, &group.right)?;
            // A descendant paired outside this run is evidence of a separate move.
            if left_ids.iter().any(|id| {
                pairing
                    .get(id)
                    .is_some_and(|other| !right_ids.contains(other))
            }) || right_ids.iter().any(|id| {
                reverse_pairing
                    .get(id)
                    .is_some_and(|other| !left_ids.contains(other))
            }) {
                continue;
            }
            let Some(inlines) = inline_pairs_of(left, right, &group)? else {
                continue;
            };
            let left_refs = targets(left, &group.left, &left_targets)?;
            let right_refs = targets(right, &group.right, &right_targets)?;
            if left_refs.len() != right_refs.len()
                || !left_refs
                    .iter()
                    .zip(&right_refs)
                    .all(|(l, r)| match (l, r) {
                        (Some(l), Some(r)) => {
                            pairing.get(l) == Some(r) || inlines.contains(&(*l, *r))
                        }
                        (None, None) => true,
                        _ => false,
                    })
            {
                continue;
            }
            left_covered.extend(left_ids);
            right_covered.extend(right_ids);
            inline_pairs.extend(inlines);
            records.push(Correspondence::ParagraphGroup {
                left: references(left, &group.left)?,
                right: references(right, &group.right)?,
            });
            accepted.push(group);
        }
        aligned.properties[index]
            .steps
            .retain(|step| !accepted.iter().any(|group| group.covers(step)));
    }
    if records.is_empty() {
        return Ok(());
    }
    let left_paths: HashSet<_> = left_covered
        .iter()
        .map(|id| left.occurrence(*id).map(|node| &node.path))
        .collect::<CompareResult<_>>()?;
    let right_paths: HashSet<_> = right_covered
        .iter()
        .map(|id| right.occurrence(*id).map(|node| &node.path))
        .collect::<CompareResult<_>>()?;
    records.extend(
        aligned
            .alignment
            .correspondences()
            .iter()
            .filter(|item| {
                !item
                    .left()
                    .is_some_and(|node| left_paths.contains(&node.path))
                    && !item
                        .right()
                        .is_some_and(|node| right_paths.contains(&node.path))
            })
            .cloned(),
    );
    aligned.alignment = Alignment::new(aligned.alignment.algorithm().clone(), records)?;
    aligned
        .pairs
        .retain(|(l, r)| !left_covered.contains(l) && !right_covered.contains(r));
    aligned
        .reference_pairs
        .retain(|(l, r)| !left_covered.contains(l) && !right_covered.contains(r));
    aligned.reference_pairs.extend(inline_pairs);
    aligned.properties.retain(|property| {
        !left_covered.contains(&property.left_parent)
            && !right_covered.contains(&property.right_parent)
    });
    Ok(())
}

fn find_property(
    projection: &Projection,
    parent: OccurrenceId,
    name: NodeProperty,
) -> CompareResult<&ProjectedProperty> {
    projection
        .occurrence(parent)?
        .properties
        .iter()
        .find(|property| property.decl.property == name)
        .ok_or_else(|| crate::CompareError::Invariant {
            message: "Missing aligned property".into(),
        })
}

fn descendants(
    projection: &Projection,
    roots: &[OccurrenceId],
) -> CompareResult<HashSet<OccurrenceId>> {
    let mut ids = HashSet::new();
    for root in roots {
        let node = projection.occurrence(*root)?;
        ids.extend(
            (root.index()..root.index() + node.subtree_size as usize).map(OccurrenceId::new),
        );
    }
    Ok(ids)
}

/// Pair the inline occurrences within a group one-for-one
///
/// Every occurrence beneath the paragraphs, other than plain text, contributes a node
/// token in document order, so equal content has equal sequences of them. Pairing
/// them keeps references to identified inlines resolvable once the group replaces
/// their individual correspondences. `None` if the sequences unexpectedly differ.
fn inline_pairs_of(
    left: &Projection,
    right: &Projection,
    group: &Group,
) -> CompareResult<Option<HashSet<(OccurrenceId, OccurrenceId)>>> {
    let left_inlines = inlines(left, &group.left)?;
    let right_inlines = inlines(right, &group.right)?;
    if left_inlines.len() != right_inlines.len()
        || left_inlines
            .iter()
            .zip(&right_inlines)
            .any(|(l, r)| l.node_type != r.node_type)
    {
        return Ok(None);
    }
    Ok(Some(
        left_inlines
            .iter()
            .zip(&right_inlines)
            .map(|(l, r)| (l.id, r.id))
            .collect(),
    ))
}

/// The occurrences beneath paragraphs that are not plain text, in document order
fn inlines<'p>(
    projection: &'p Projection,
    roots: &[OccurrenceId],
) -> CompareResult<Vec<&'p Occurrence>> {
    let mut nodes = Vec::new();
    for root in roots {
        let node = projection.occurrence(*root)?;
        nodes.extend(
            projection
                .occurrences()
                .iter()
                .skip(root.index() + 1)
                .take((node.subtree_size as usize).saturating_sub(1))
                .filter(|node| plain_text(node).is_none()),
        );
    }
    Ok(nodes)
}

fn targets(
    projection: &Projection,
    roots: &[OccurrenceId],
    index: &TargetIndex,
) -> CompareResult<Vec<Option<OccurrenceId>>> {
    let mut targets = Vec::new();
    for root in roots {
        let node = projection.occurrence(*root)?;
        for node in projection
            .occurrences()
            .iter()
            .skip(root.index())
            .take(node.subtree_size as usize)
        {
            if !matches!(node.node_type, NodeType::Citation | NodeType::Link) {
                continue;
            }
            if let Some(property) = node
                .properties
                .iter()
                .find(|property| property.decl.property == NodeProperty::Target)
                && let [Item::Scalar(value)] = property.items.as_slice()
            {
                targets.push(index.resolve(node.node_type, value));
            }
        }
    }
    Ok(targets)
}

/// Find non-overlapping, unambiguous groups in a completed sibling alignment.
///
/// Candidates depend only on sibling positions and pairings, never on the order of
/// steps, which reconciling reorders can change, so that recognition is the same with
/// the sides swapped.
fn find(
    left: &Projection,
    right: &Projection,
    left_property: &ProjectedProperty,
    right_property: &ProjectedProperty,
    steps: &[Step],
) -> CompareResult<Vec<Group>> {
    let mut left_partners = vec![None; left_property.items.len()];
    let mut right_partners = vec![None; right_property.items.len()];
    for step in steps {
        if let Step::Pair { left, right } = *step {
            left_partners[left] = Some(right);
            right_partners[right] = Some(left);
        }
    }
    let left_siblings = Siblings::new(left, &left_property.items, left_partners)?;
    let right_siblings = Siblings::new(right, &right_property.items, right_partners)?;

    let candidates: Vec<(Range<usize>, Range<usize>)> = left_siblings
        .joins(&right_siblings)
        .into_iter()
        .map(|(index, run)| (index..index + 1, run))
        .chain(
            right_siblings
                .joins(&left_siblings)
                .into_iter()
                .map(|(index, run)| (run, index..index + 1)),
        )
        .collect();

    // Repeated text can offer more than one explanation. Keep the original
    // alignment for every conflicting candidate, symmetrically in both directions.
    let mut left_uses = vec![0usize; left_property.items.len()];
    let mut right_uses = vec![0usize; right_property.items.len()];
    for (left_items, right_items) in &candidates {
        left_items.clone().for_each(|index| left_uses[index] += 1);
        right_items.clone().for_each(|index| right_uses[index] += 1);
    }
    let ids = |items: &[Item], range: &Range<usize>| {
        items[range.clone()]
            .iter()
            .filter_map(|item| match item {
                Item::Structured(id) => Some(*id),
                Item::Scalar(..) => None,
            })
            .collect()
    };
    Ok(candidates
        .into_iter()
        .filter(|(left_items, right_items)| {
            left_items.clone().all(|index| left_uses[index] == 1)
                && right_items.clone().all(|index| right_uses[index] == 1)
        })
        .map(|(left_items, right_items)| Group {
            left: ids(&left_property.items, &left_items),
            right: ids(&right_property.items, &right_items),
            left_items,
            right_items,
        })
        .collect())
}

/// One side's items of an aligned property
struct Siblings {
    /// The content of each item that is a paragraph able to join a group
    contents: Vec<Option<Vec<Token>>>,

    /// The size of each item's content, to reject most runs without joining them
    sizes: Vec<usize>,

    /// The position of the other side's item that each item is paired with
    partners: Vec<Option<usize>>,

    /// The partner of the nearest paired item before each item
    previous: Vec<Option<usize>>,

    /// The partner of the nearest paired item after each item
    next: Vec<Option<usize>>,
}

impl Siblings {
    fn new(
        projection: &Projection,
        items: &[Item],
        partners: Vec<Option<usize>>,
    ) -> CompareResult<Self> {
        let contents = contents(projection, items)?;
        let sizes = contents
            .iter()
            .map(|tokens| tokens.as_deref().map_or(0, size))
            .collect();
        let mut previous = Vec::with_capacity(partners.len());
        let mut last = None;
        for partner in &partners {
            previous.push(last);
            last = partner.or(last);
        }
        let mut next = vec![None; partners.len()];
        let mut last = None;
        for (index, partner) in partners.iter().enumerate().rev() {
            next[index] = last;
            last = partner.or(last);
        }
        Ok(Self {
            contents,
            sizes,
            partners,
            previous,
            next,
        })
    }

    fn len(&self) -> usize {
        self.contents.len()
    }

    /// Whether an item is a paragraph without a counterpart
    fn unpaired(&self, index: usize) -> bool {
        self.partners[index].is_none() && self.contents[index].is_some()
    }

    /// Runs of the other side's paragraphs that join to one paragraph here
    ///
    /// A paired paragraph extends its partner over that partner's unpaired neighbors.
    /// An unpaired one may join a run of unpaired paragraphs only where each lies
    /// between the partners of the other's nearest paired neighbors, which keeps
    /// the group local even beside reordered siblings.
    fn joins(&self, other: &Self) -> Vec<(usize, Range<usize>)> {
        let mut found = Vec::new();
        for (index, content) in self.contents.iter().enumerate() {
            let Some(content) = content else {
                continue;
            };
            let mut runs = Vec::new();
            if let Some(partner) = self.partners[index] {
                if other.contents[partner].is_none() {
                    continue;
                }
                let mut first = partner;
                while first > 0 && partner - first + 1 < MAX_PARAGRAPHS && other.unpaired(first - 1)
                {
                    first -= 1;
                }
                let mut last = partner;
                while last + 1 < other.len()
                    && last - partner + 1 < MAX_PARAGRAPHS
                    && other.unpaired(last + 1)
                {
                    last += 1;
                }
                for start in first..=partner {
                    for end in partner..=last {
                        runs.push(start..end + 1);
                    }
                }
            } else {
                let start = self.previous[index].map_or(0, |partner| partner + 1);
                let end = self.next[index].unwrap_or(other.len());
                for first in start..end {
                    if !other.unpaired(first)
                        || other.previous[first].is_some_and(|partner| partner >= index)
                    {
                        continue;
                    }
                    for last in first + 1..end.min(first + MAX_PARAGRAPHS) {
                        if !other.unpaired(last) {
                            break;
                        }
                        if other.next[last].is_none_or(|partner| partner > index) {
                            runs.push(first..last + 1);
                        }
                    }
                }
            }
            for run in runs {
                if (2..=MAX_PARAGRAPHS).contains(&run.len())
                    && size(content)
                        == other.sizes[run.clone()].iter().sum::<usize>() + run.len() - 1
                    && joined(&other.contents, run.clone()).as_ref() == Some(content)
                {
                    found.push((index, run));
                }
            }
        }
        found
    }
}

/// Characters of text plus one for every other token
///
/// Joining paragraphs adds one space between each, so equal content has equal size.
fn size(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .map(|token| match token {
            Token::Text(text) => text.len(),
            _ => 1,
        })
        .sum()
}

fn references(projection: &Projection, ids: &[OccurrenceId]) -> CompareResult<ParagraphGroup> {
    let nodes = ids
        .iter()
        .map(|id| Ok(projection.occurrence(*id)?.node_ref()))
        .collect::<CompareResult<Vec<_>>>()?;
    let Some((first, rest)) = nodes.split_first() else {
        return Err(crate::CompareError::Invariant {
            message: "An empty paragraph group".into(),
        });
    };
    Ok(ParagraphGroup {
        first: first.clone(),
        rest: rest.to_vec(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text(String),
    Node(NodeType),
    Property(NodeProperty, Presence),
    Scalar(ScalarValue),
    End,
}

fn contents(projection: &Projection, items: &[Item]) -> CompareResult<Vec<Option<Vec<Token>>>> {
    items
        .iter()
        .map(|item| {
            let Item::Structured(id) = item else {
                return Ok(None);
            };
            let node = projection.occurrence(*id)?;
            if node.node_type != NodeType::Paragraph
                || node.properties.iter().any(|property| {
                    property.decl.property != NodeProperty::Content
                        && property.presence == Presence::Present
                })
            {
                return Ok(None);
            }
            let Some(content) = node
                .properties
                .iter()
                .find(|property| property.decl.property == NodeProperty::Content)
            else {
                return Ok(None);
            };
            let mut tokens = Vec::new();
            for item in &content.items {
                append_item(projection, item, &mut tokens)?;
            }
            // Empty and purely non-text paragraphs provide no evidence for a boundary.
            let has_text = tokens
                .iter()
                .any(|token| matches!(token, Token::Text(text) if !text.trim().is_empty()));
            Ok(has_text.then_some(tokens))
        })
        .collect()
}

fn append_item(projection: &Projection, item: &Item, tokens: &mut Vec<Token>) -> CompareResult<()> {
    let Item::Structured(id) = item else {
        if let Item::Scalar(value) = item {
            tokens.push(Token::Scalar(value.clone()));
        }
        return Ok(());
    };
    let node = projection.occurrence(*id)?;
    if let Some(value) = plain_text(node) {
        append_text(tokens, value);
        return Ok(());
    }
    tokens.push(Token::Node(node.node_type));
    for property in &node.properties {
        tokens.push(Token::Property(property.decl.property, property.presence));
        for item in &property.items {
            append_item(projection, item, tokens)?;
        }
        tokens.push(Token::End);
    }
    tokens.push(Token::End);
    Ok(())
}

/// The string of a Text occurrence with no property other than its value
fn plain_text(node: &Occurrence) -> Option<&str> {
    if node.node_type != NodeType::Text
        || node.properties.iter().any(|property| {
            property.decl.property != NodeProperty::Value && property.presence != Presence::Absent
        })
    {
        return None;
    }
    let value = node
        .properties
        .iter()
        .find(|property| property.decl.property == NodeProperty::Value)?;
    match value.items.as_slice() {
        [Item::Scalar(ScalarValue::String { value })] => Some(value),
        _ => None,
    }
}

fn append_text(tokens: &mut Vec<Token>, text: &str) {
    if let Some(Token::Text(previous)) = tokens.last_mut() {
        previous.push_str(text);
    } else if !text.is_empty() {
        tokens.push(Token::Text(text.into()));
    }
}

fn joined(contents: &[Option<Vec<Token>>], indices: Range<usize>) -> Option<Vec<Token>> {
    let mut joined = Vec::new();
    for (position, index) in indices.enumerate() {
        let tokens = contents.get(index)?.as_ref()?;
        if position > 0 {
            append_text(&mut joined, " ");
        }
        for token in tokens {
            match token {
                Token::Text(text) => append_text(&mut joined, text),
                _ => joined.push(token.clone()),
            }
        }
    }
    Some(joined)
}
