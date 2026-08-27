//! Attaching comments to the merged document
//!
//! A `Comment` is a `CreativeWork`, not a block or an inline, so it does not live in
//! the content: it hangs off the `comments` property of the work it is about, and
//! points back into the content by identifier. The schema allows `startLocation` to be
//! `#id` referencing any node with that identifier, which is what is used here — the
//! target occurrence is given an identifier if it does not already have one.
//!
//! The alternative the schema also supports is a pair of `Boundary` inlines bracketing
//! a region, which is what the Markdown and Word codecs round-trip. That is a better
//! fit for commenting on *part* of a paragraph, and worse for commenting on a table or
//! a code block, which have no inline content to bracket. Since every comment a merge
//! produces is about a whole occurrence rather than a span within one, identifiers
//! cover every case uniformly and boundaries would not.

use std::collections::{BTreeMap, BTreeSet};

use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::NodeProperty;
use stencila_schema::{
    Block, Comment, CommentOptions, Inline, Node, NodeSet, Paragraph, PatchContext, PatchNode,
    PatchOp, PatchValue, Text, get,
};

use crate::{
    error::{MergeError, MergeResult},
    options::MergeOptions,
    plan::PendingComment,
};

/// Identify the occurrences the planned comments are about
///
/// Runs *before* the container rewrites, while the comparison's paths still address
/// the document they were derived from. Afterwards a commented occurrence may have
/// been wrapped in a suggestion, and the same path would reach the wrapper instead —
/// so the comment would end up pointing at the change rather than at what changed.
/// Identifying first also means the identifier travels with the occurrence into
/// whichever slot of the suggestion it ends up in.
///
/// Returns the location string for each planned comment, in order, which is `None`
/// where the target could not be identified.
pub(crate) fn identify_targets(
    working: &mut Node,
    pending: &[PendingComment],
    options: &MergeOptions,
) -> Vec<Option<String>> {
    let mut used = identifiers(working);
    let mut identified: BTreeMap<NodePath, String> = BTreeMap::new();
    let mut next = 0;

    pending
        .iter()
        .map(|comment| {
            let target = comment.target.as_ref()?;

            if let Some(id) = identified.get(target) {
                return Some(format!("#{id}"));
            }

            let id = if let Some(id) = existing_id(working, target) {
                id
            } else {
                let id = next_available_id(&mut used, &options.id_prefix, "t", &mut next);
                identify(working, target, &id).ok()?;
                id
            };
            identified.insert(target.clone(), id.clone());
            Some(format!("#{id}"))
        })
        .collect()
}

/// Attach the planned comments to the merged document
///
/// Returns the number attached.
pub(crate) fn attach(
    working: &mut Node,
    pending: &[PendingComment],
    locations: &[Option<String>],
    options: &MergeOptions,
) -> MergeResult<usize> {
    if pending.is_empty() {
        return Ok(0);
    }

    let Node::Article(..) = working else {
        return Err(MergeError::UnsupportedRoot {
            node_type: working.node_type(),
        });
    };

    let mut comments = Vec::with_capacity(pending.len());
    let mut used = identifiers(working);
    let mut next = 0;

    for (index, comment) in pending.iter().enumerate() {
        let id = next_available_id(&mut used, &options.id_prefix, "c", &mut next);
        let location = locations.get(index).cloned().flatten();

        comments.push(Comment {
            id: Some(id),
            content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                Text::from(comment.message.as_str()),
            )]))],
            authors: options.authors.clone(),
            options: Box::new(CommentOptions {
                start_location: location.clone(),
                end_location: location,
                ..Default::default()
            }),
            ..Default::default()
        });
    }

    let attached = comments.len();

    if let Node::Article(article) = working {
        match &mut article.options.comments {
            Some(existing) => existing.extend(comments),
            none => *none = Some(comments),
        }
    }

    Ok(attached)
}

/// Give the occurrence at a path an identifier
///
/// Only sets one where the occurrence has none, so a document that already identifies
/// its nodes keeps the identifiers it came with.
fn identify(working: &mut Node, target: &NodePath, id: &str) -> MergeResult<()> {
    let mut path = target.clone();
    path.push_back(NodeSlot::Property(NodeProperty::Id));

    let mut context = PatchContext::default();
    let mut at = path.clone();

    working
        .apply(
            &mut at,
            PatchOp::Set(PatchValue::String(id.to_string())),
            &mut context,
        )
        .map_err(|error| MergeError::Apply {
            path,
            message: error.to_string(),
        })
}

/// The existing identifier of a target occurrence, if it has one
fn existing_id(working: &Node, target: &NodePath) -> Option<String> {
    let mut path = target.clone();
    path.push_back(NodeSlot::Property(NodeProperty::Id));

    match get(working, path).ok()? {
        NodeSet::One(Node::String(id)) if !id.is_empty() => Some(id),
        _ => None,
    }
}

/// Allocate a deterministic identifier that is not already present in the document
fn next_available_id(
    used: &mut BTreeSet<String>,
    prefix: &str,
    role: &str,
    next: &mut usize,
) -> String {
    loop {
        let id = format!("{prefix}{role}{next}");
        *next += 1;
        if used.insert(id.clone()) {
            return id;
        }
    }
}

/// Collect serialized `id` properties without maintaining a list of schema node types
fn identifiers(node: &Node) -> BTreeSet<String> {
    let mut identifiers = BTreeSet::new();
    if let Ok(value) = serde_json::to_value(node) {
        collect_identifiers(&value, &mut identifiers);
    }
    identifiers
}

fn collect_identifiers(value: &serde_json::Value, identifiers: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_identifiers(item, identifiers);
            }
        }
        serde_json::Value::Object(entries) => {
            if let Some(id) = entries.get("id").and_then(serde_json::Value::as_str) {
                identifiers.insert(id.to_string());
            }
            for value in entries.values() {
                collect_identifiers(value, identifiers);
            }
        }
        _ => {}
    }
}
