use std::collections::HashMap;

use itertools::Itertools;
use pandoc_types::definition::{self as pandoc};

use stencila_codec::stencila_schema::*;

use crate::{
    inlines::{comment_blocks_to_pandoc_inlines, pandoc_inlines_to_blocks},
    shared::{PandocDecodeContext, PandocEncodeContext, PendingComment},
};

/// Normalize reply parent ids collected from Pandoc comment spans.
///
/// For some DOCX imports, Pandoc can represent sibling replies as a nested
/// chain of `comment-end` spans with no explicit `parent` attributes on the
/// nested replies. Apply the fallback only to comments observed solely in
/// nested `comment-end` spans. When several anchored top-level comments are
/// present, attach those unresolved nested-only replies to the first anchored
/// thread rather than flattening them into extra top-level comments.
fn normalize_reply_parents(pending_comments: &mut [PendingComment]) {
    let anchored_top_level_ids = pending_comments
        .iter()
        .filter(|comment| {
            comment.has_start_span && !comment.nested_end_only && comment.parent_pandoc_id.is_none()
        })
        .map(|comment| comment.pandoc_id.clone())
        .collect_vec();

    if anchored_top_level_ids.len() > 1 {
        let Some(root_id) = anchored_top_level_ids.first().cloned() else {
            return;
        };

        for pending_comment in pending_comments.iter_mut() {
            if pending_comment.nested_end_only && pending_comment.parent_pandoc_id.is_none() {
                pending_comment.parent_pandoc_id = Some(root_id.clone());
            }
        }

        return;
    }

    let root_ids = pending_comments
        .iter()
        .filter(|comment| comment.has_start_span && !comment.nested_end_only)
        .map(|comment| comment.pandoc_id.clone())
        .collect_vec();

    let [root_id] = root_ids.as_slice() else {
        return;
    };

    for pending_comment in pending_comments.iter_mut() {
        if pending_comment.pandoc_id != *root_id
            && pending_comment.parent_pandoc_id.is_none()
            && pending_comment.nested_end_only
        {
            pending_comment.parent_pandoc_id = Some(root_id.clone());
        }
    }
}

/// Create the Pandoc attributes for a comment end span.
fn comment_end_attrs(pandoc_id: &str, parent_pandoc_id: Option<&str>) -> pandoc::Attr {
    let mut attrs = comment_span_attrs("comment-end", pandoc_id);
    if let Some(parent_id) = parent_pandoc_id {
        attrs
            .attributes
            .push(("parent".into(), parent_id.to_string()));
    }
    attrs
}

/// Allocate integer Pandoc comment ids for DOCX flavors.
///
/// Word comment ids must be decimal integers. Preserve existing numeric ids
/// from anchored comments when possible, and allocate new ids for replies or
/// other comments with non-numeric Stencila ids.
struct DocxCommentIdAllocator {
    reserved_ids: HashMap<usize, usize>,
    next_id: usize,
}

impl DocxCommentIdAllocator {
    /// Create a new allocator seeded with numeric ids already present in the tree.
    fn new(comments: &[Comment]) -> Self {
        let mut reserved_ids = HashMap::new();
        collect_docx_comment_ids(comments, &mut reserved_ids);

        let next_id = reserved_ids.keys().max().map_or(0, |id| id + 1);

        Self {
            reserved_ids,
            next_id,
        }
    }

    /// Assign a numeric Pandoc id, preserving unique existing numeric ids when possible.
    fn assign(&mut self, comment: &Comment, start_location: Option<&str>) -> String {
        if let Some(numeric_id) = docx_comment_numeric_id(comment, start_location)
            && self.reserved_ids.get(&numeric_id) == Some(&1)
        {
            return numeric_id.to_string();
        }

        self.allocate()
    }

    /// Allocate the next unused numeric Pandoc comment id.
    fn allocate(&mut self) -> String {
        while self.reserved_ids.contains_key(&self.next_id) {
            self.next_id += 1;
        }

        let id = self.next_id;
        self.next_id += 1;

        id.to_string()
    }
}

/// Collect all numeric comment ids already used anywhere in the comment tree.
fn collect_docx_comment_ids(comments: &[Comment], reserved_ids: &mut HashMap<usize, usize>) {
    for comment in comments {
        if let Some(numeric_id) =
            docx_comment_numeric_id(comment, comment.options.start_location.as_deref())
        {
            *reserved_ids.entry(numeric_id).or_default() += 1;
        }

        if let Some(replies) = &comment.options.comments {
            collect_docx_comment_ids(replies, reserved_ids);
        }
    }
}

/// Resolve a numeric DOCX/Pandoc comment id from a boundary location or comment id.
fn docx_comment_numeric_id(comment: &Comment, start_location: Option<&str>) -> Option<usize> {
    start_location
        .and_then(comment_boundary_id)
        .and_then(|id| id.parse().ok())
        .or_else(|| comment.id.as_ref().and_then(|id| id.parse().ok()))
}

struct PreparedComment {
    start_inlines: Vec<pandoc::Inline>,
    end_inline: pandoc::Inline,
}

struct CommentEncodingPlan<'a> {
    pandoc_id: &'a str,
    parent_pandoc_id: Option<&'a str>,
    should_anchor_reply: bool,
    start_location: Option<&'a str>,
    end_location: Option<&'a str>,
}

/// Create the shared Pandoc attributes used by comment start and end spans.
fn comment_span_attrs(class: &str, pandoc_id: &str) -> pandoc::Attr {
    let mut attrs = pandoc::Attr::default();
    attrs.classes.push(class.into());
    attrs.attributes.push(("id".into(), pandoc_id.to_string()));
    attrs
}

fn comment_start_boundary_id(pandoc_id: &str) -> String {
    format!("comment-{pandoc_id}-start")
}

fn comment_end_boundary_id(pandoc_id: &str) -> String {
    format!("comment-{pandoc_id}-end")
}

fn comment_boundary_location(boundary_id: String) -> String {
    format!("#{boundary_id}")
}

/// Recursively prepare encoded Pandoc spans for nested reply comments.
fn prepare_nested_reply_inlines(
    replies: &Option<Vec<Comment>>,
    parent_pandoc_id: &str,
    next_id: &mut usize,
    docx_ids: &mut Option<DocxCommentIdAllocator>,
    context: &mut PandocEncodeContext,
) -> (Vec<pandoc::Inline>, Vec<pandoc::Inline>) {
    let mut reply_start_inlines = Vec::new();
    let mut reply_end_inlines = Vec::new();

    if let Some(replies) = replies {
        for reply in replies {
            if let Some(prepared) = prepare_comment_for_pandoc(
                reply,
                Some(parent_pandoc_id),
                next_id,
                docx_ids,
                context,
            ) {
                reply_start_inlines.extend(prepared.start_inlines);
                reply_end_inlines.push(prepared.end_inline);
            }
        }
    }

    (reply_start_inlines, reply_end_inlines)
}

/// Build article-level comments from decoded pending Pandoc comment spans.
///
/// This merges duplicate sightings of the same Pandoc comment id, normalizes
/// missing reply parents seen in DOCX-style nested `comment-end` spans,
/// converts the flattened pending comments into Stencila `Comment`s while
/// preserving explicit reply `parent_item` ids, nests replies under their
/// parents, and strips reply boundary markers from content.
pub(super) fn comments_from_pending(
    context: &mut PandocDecodeContext,
    content: &mut [Block],
) -> Option<Vec<Comment>> {
    if context.pending_comments.is_empty() {
        return None;
    }

    let mut pending = merge_pending_comments(context.pending_comments.drain(..).collect());
    normalize_reply_parents(&mut pending);
    let parent_ids: Vec<Option<String>> = pending
        .iter()
        .map(|comment| comment.parent_pandoc_id.clone())
        .collect();
    let pandoc_ids: Vec<String> = pending
        .iter()
        .map(|comment| comment.pandoc_id.clone())
        .collect();

    let reply_ids = reply_pandoc_ids(&parent_ids, &pandoc_ids);
    let id_to_index: HashMap<String, usize> = pandoc_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.clone(), index))
        .collect();

    let mut comments = pending_comments_to_comments(pending, &parent_ids, context);
    attach_reply_comments(&mut comments, &parent_ids, &pandoc_ids, &id_to_index);

    if !reply_ids.is_empty() {
        strip_reply_boundaries(content, &reply_ids);
    }

    Some(comments)
}

/// Merge repeated pending comment records for the same Pandoc id.
fn merge_pending_comments(pending_comments: Vec<PendingComment>) -> Vec<PendingComment> {
    let mut merged: Vec<PendingComment> = Vec::new();

    for pending_comment in pending_comments {
        if let Some(existing) = merged
            .iter_mut()
            .find(|existing| existing.pandoc_id == pending_comment.pandoc_id)
        {
            merge_pending_comment(existing, pending_comment);
        } else {
            merged.push(pending_comment);
        }
    }

    merged
}

/// Combine two sightings of the same pending Pandoc comment.
///
/// Pandoc comment data may arrive from separate start and end spans, or from
/// nested end spans during DOCX decoding. Prefer the first non-empty metadata
/// fields while unioning the anchor flags.
fn merge_pending_comment(existing: &mut PendingComment, pending_comment: PendingComment) {
    if existing.author.is_none() {
        existing.author = pending_comment.author;
    }
    if existing.date.is_none() {
        existing.date = pending_comment.date;
    }
    if existing.body_inlines.is_empty() {
        existing.body_inlines = pending_comment.body_inlines;
    }
    if existing.parent_pandoc_id.is_none() {
        existing.parent_pandoc_id = pending_comment.parent_pandoc_id;
    }
    existing.has_start_span |= pending_comment.has_start_span;
    existing.nested_end_only &= pending_comment.nested_end_only;
}

fn reply_pandoc_ids(parent_ids: &[Option<String>], pandoc_ids: &[String]) -> Vec<String> {
    parent_ids
        .iter()
        .zip(pandoc_ids.iter())
        .filter_map(|(parent, id)| parent.as_ref().map(|_| id.clone()))
        .collect()
}

fn pending_comments_to_comments(
    pending_comments: Vec<PendingComment>,
    parent_ids: &[Option<String>],
    context: &mut PandocDecodeContext,
) -> Vec<Comment> {
    pending_comments
        .into_iter()
        .enumerate()
        .map(|(index, pending_comment)| {
            let parent_id = parent_ids[index].clone();
            let is_reply = parent_id.is_some();
            let pandoc_id = pending_comment.pandoc_id;

            Comment {
                id: Some(pandoc_id.clone()),
                content: pandoc_inlines_to_blocks(pending_comment.body_inlines, context),
                authors: pending_comment.author.map(|names| {
                    names
                        .split(';')
                        .map(|name| Author::Person(Person::from(name.trim())))
                        .collect_vec()
                }),
                date_published: pending_comment.date.map(DateTime::new),
                options: Box::new(CommentOptions {
                    parent_item: parent_id,
                    start_location: (!is_reply)
                        .then(|| comment_boundary_location(comment_start_boundary_id(&pandoc_id))),
                    end_location: (!is_reply)
                        .then(|| comment_boundary_location(comment_end_boundary_id(&pandoc_id))),
                    ..Default::default()
                }),
                ..Default::default()
            }
        })
        .collect()
}

/// Attach reply comments to their parents while preserving their original order.
fn attach_reply_comments(
    comments: &mut Vec<Comment>,
    parent_ids: &[Option<String>],
    pandoc_ids: &[String],
    id_to_index: &HashMap<String, usize>,
) {
    let reply_indices: Vec<(usize, usize)> = (0..comments.len())
        .filter_map(|index| {
            let parent_id = parent_ids[index].as_ref()?;
            let &parent_index = id_to_index.get(parent_id)?;
            Some((index, parent_index))
        })
        .collect();

    let mut replies = Vec::new();
    for &(index, parent_index) in reply_indices.iter().rev() {
        replies.push((parent_index, comments.remove(index)));
    }
    replies.reverse();

    for (original_parent_index, reply) in replies {
        let parent_pandoc_id = &pandoc_ids[original_parent_index];
        let _ = attach_reply_to_parent(comments, parent_pandoc_id, reply);
    }
}

/// Remove reply-only boundary markers from decoded article content.
fn strip_reply_boundaries(content: &mut [Block], reply_ids: &[String]) {
    let reply_boundary_ids: Vec<String> = reply_ids
        .iter()
        .map(|id| comment_start_boundary_id(id))
        .collect();

    strip_boundaries(content, &reply_boundary_ids);
}

/// Prepare all article comments for later boundary-based Pandoc encoding.
pub(super) fn prepare_comments_for_pandoc(article: &Article, context: &mut PandocEncodeContext) {
    context.comment_start_spans.clear();
    context.comment_end_spans.clear();

    let Some(comments) = &article.options.comments else {
        return;
    };

    let mut next_id = 0usize;
    let mut docx_ids = context
        .is_docx_flavor()
        .then(|| DocxCommentIdAllocator::new(comments));
    for comment in comments {
        prepare_comment_for_pandoc(comment, None, &mut next_id, &mut docx_ids, context);
    }
}

/// Prepare Pandoc comment spans for a Stencila comment and its nested replies.
///
/// Comments with explicit start and end boundary locations are encoded as a
/// start span placed at the start boundary and an end span placed at the end
/// boundary. Unanchored comments are encoded as end-only spans containing the
/// comment body and nested reply end spans. DOCX reply comments are anchored at
/// their parent comment span because Pandoc represents DOCX replies that way.
fn prepare_comment_for_pandoc(
    comment: &Comment,
    parent_pandoc_id: Option<&str>,
    next_id: &mut usize,
    docx_ids: &mut Option<DocxCommentIdAllocator>,
    context: &mut PandocEncodeContext,
) -> Option<PreparedComment> {
    let start_location = comment.options.start_location.as_deref();
    let end_location = comment.options.end_location.as_deref();
    let should_anchor_reply = parent_pandoc_id.is_some() && context.is_docx_flavor();

    let pandoc_id = assign_comment_pandoc_id(comment, start_location, next_id, docx_ids)?;

    if !should_anchor_reply && (start_location.is_none() || end_location.is_none()) {
        return Some(prepare_unanchored_comment(
            comment,
            &pandoc_id,
            parent_pandoc_id,
            next_id,
            docx_ids,
            context,
        ));
    }

    Some(prepare_anchored_comment(
        comment,
        CommentEncodingPlan {
            pandoc_id: &pandoc_id,
            parent_pandoc_id,
            should_anchor_reply,
            start_location,
            end_location,
        },
        next_id,
        docx_ids,
        context,
    ))
}

/// Choose the Pandoc id to use for a comment during encoding.
fn assign_comment_pandoc_id(
    comment: &Comment,
    start_location: Option<&str>,
    next_id: &mut usize,
    docx_ids: &mut Option<DocxCommentIdAllocator>,
) -> Option<String> {
    if let Some(docx_ids) = docx_ids.as_mut() {
        Some(docx_ids.assign(comment, start_location))
    } else if let Some(start_location) = start_location {
        comment_pandoc_id(start_location, next_id)
    } else if let Some(id) = &comment.id {
        Some(id.clone())
    } else {
        let id = next_id.to_string();
        *next_id += 1;
        Some(id)
    }
}

/// Prepare an unanchored comment as an end-only Pandoc comment span.
fn prepare_unanchored_comment(
    comment: &Comment,
    pandoc_id: &str,
    parent_pandoc_id: Option<&str>,
    next_id: &mut usize,
    docx_ids: &mut Option<DocxCommentIdAllocator>,
    context: &mut PandocEncodeContext,
) -> PreparedComment {
    let end_attrs = comment_end_attrs(pandoc_id, parent_pandoc_id);
    let mut end_inlines = comment_blocks_to_pandoc_inlines(&comment.content, context);
    let (_, mut reply_end_inlines) = prepare_nested_reply_inlines(
        &comment.options.comments,
        pandoc_id,
        next_id,
        docx_ids,
        context,
    );
    end_inlines.append(&mut reply_end_inlines);

    PreparedComment {
        start_inlines: Vec::new(),
        end_inline: pandoc::Inline::Span(end_attrs, end_inlines),
    }
}

/// Prepare an anchored comment with separate start and end Pandoc spans.
fn prepare_anchored_comment(
    comment: &Comment,
    plan: CommentEncodingPlan,
    next_id: &mut usize,
    docx_ids: &mut Option<DocxCommentIdAllocator>,
    context: &mut PandocEncodeContext,
) -> PreparedComment {
    let start_attrs = comment_start_attrs(
        comment,
        plan.pandoc_id,
        plan.parent_pandoc_id,
        plan.should_anchor_reply,
    );

    let body_inlines = comment_blocks_to_pandoc_inlines(&comment.content, context);
    let (reply_start_inlines, reply_end_inlines) = prepare_nested_reply_inlines(
        &comment.options.comments,
        plan.pandoc_id,
        next_id,
        docx_ids,
        context,
    );

    let end_attrs = comment_end_attrs(plan.pandoc_id, plan.parent_pandoc_id);
    let start_inline = pandoc::Inline::Span(start_attrs, body_inlines);
    let mut start_inlines = vec![start_inline];
    start_inlines.extend(reply_start_inlines);
    let end_inline = pandoc::Inline::Span(end_attrs, reply_end_inlines);

    if let (Some(start_location), Some(end_location)) = (plan.start_location, plan.end_location)
        && !plan.should_anchor_reply
    {
        register_comment_boundaries(
            context,
            start_location,
            end_location,
            &start_inlines,
            &end_inline,
        );
    }

    PreparedComment {
        start_inlines,
        end_inline,
    }
}

/// Create the Pandoc attributes for a comment start span.
fn comment_start_attrs(
    comment: &Comment,
    pandoc_id: &str,
    parent_pandoc_id: Option<&str>,
    include_parent: bool,
) -> pandoc::Attr {
    let mut attrs = comment_span_attrs("comment-start", pandoc_id);

    if let Some(author) = comment
        .authors
        .as_ref()
        .map(|authors| authors.iter().map(|author| author.name()).join(";"))
    {
        attrs.attributes.push(("author".into(), author));
    }

    if let Some(date) = &comment.date_published {
        attrs
            .attributes
            .push(("date".into(), date.value.to_string()));
    }

    if include_parent && let Some(parent_pandoc_id) = parent_pandoc_id {
        attrs
            .attributes
            .push(("parent".into(), parent_pandoc_id.to_string()));
    }

    attrs
}

/// Register encoded comment spans against their boundary ids in the encode context.
fn register_comment_boundaries(
    context: &mut PandocEncodeContext,
    start_location: &str,
    end_location: &str,
    start_inlines: &[pandoc::Inline],
    end_inline: &pandoc::Inline,
) {
    let Some(start_boundary_id) = start_location.strip_prefix('#') else {
        return;
    };
    let Some(end_boundary_id) = end_location.strip_prefix('#') else {
        return;
    };

    context
        .comment_start_spans
        .insert(start_boundary_id.to_string(), start_inlines.to_vec());
    context
        .comment_end_spans
        .insert(end_boundary_id.to_string(), vec![end_inline.clone()]);
}

/// Extract the logical comment id from a `#comment-<id>-start` boundary location.
fn comment_boundary_id(start_location: &str) -> Option<&str> {
    let boundary_id = start_location.strip_prefix('#')?;
    let comment_id = boundary_id.strip_prefix("comment-")?;
    comment_id.strip_suffix("-start")
}

/// Derive a Pandoc comment id from a start boundary location.
///
/// If the extracted id is numeric, advance `next_id` so that subsequently
/// allocated ids remain unique.
fn comment_pandoc_id(start_location: &str, next_id: &mut usize) -> Option<String> {
    let comment_id = comment_boundary_id(start_location)?;

    if let Ok(numeric) = comment_id.parse::<usize>() {
        *next_id = (*next_id).max(numeric + 1);
    }

    Some(comment_id.to_string())
}

/// Recursively find a comment whose `id` matches `target_id`
/// and append `reply` to its `comments`.
///
/// Returns `None` when the reply is attached, or `Some(reply)` when no matching
/// parent is found so ownership can be returned to the caller.
fn attach_reply_to_parent(
    comments: &mut [Comment],
    target_id: &str,
    reply: Comment,
) -> Option<Comment> {
    // Two-pass: first check direct children, then recurse.
    // This avoids borrow conflicts from iterating while recursing.
    for comment in comments.iter_mut() {
        if comment.id.as_deref() == Some(target_id) {
            comment
                .options
                .comments
                .get_or_insert_with(Vec::new)
                .push(reply);
            return None;
        }
    }
    let mut reply = reply;
    for comment in comments.iter_mut() {
        if let Some(nested) = &mut comment.options.comments {
            reply = attach_reply_to_parent(nested, target_id, reply)?;
        }
    }
    Some(reply)
}

/// Remove [`Boundary`] nodes with any of the given IDs from block content.
fn strip_boundaries(blocks: &mut [Block], ids: &[String]) {
    fn strip_inlines(inlines: &mut Vec<Inline>, ids: &[String]) {
        inlines.retain(|inline| {
            !matches!(
                inline,
                Inline::Boundary(b) if b.id.as_ref().is_some_and(|id| ids.contains(id))
            )
        });
    }

    // This is intentionally scoped to the inline-bearing/container blocks that
    // Pandoc DOCX decoding emits into article content.
    for block in blocks.iter_mut() {
        match block {
            // Blocks with inline content
            Block::Paragraph(node) => strip_inlines(&mut node.content, ids),
            Block::Heading(node) => strip_inlines(&mut node.content, ids),
            Block::InlinesBlock(node) => strip_inlines(&mut node.content, ids),

            // Blocks with block content — recurse
            Block::Admonition(node) => strip_boundaries(&mut node.content, ids),
            Block::Claim(node) => strip_boundaries(&mut node.content, ids),
            Block::Figure(node) => strip_boundaries(&mut node.content, ids),
            Block::ForBlock(node) => strip_boundaries(&mut node.content, ids),
            Block::IfBlock(node) => {
                for clause in &mut node.clauses {
                    strip_boundaries(&mut clause.content, ids);
                }
            }
            Block::QuoteBlock(node) => strip_boundaries(&mut node.content, ids),
            Block::Section(node) => strip_boundaries(&mut node.content, ids),
            Block::StyledBlock(node) => strip_boundaries(&mut node.content, ids),

            // List items have block content
            Block::List(list) => {
                for item in &mut list.items {
                    strip_boundaries(&mut item.content, ids);
                }
            }

            // Table cells have block content
            Block::Table(table) => {
                for row in &mut table.rows {
                    for cell in &mut row.cells {
                        strip_boundaries(&mut cell.content, ids);
                    }
                }
            }

            _ => {}
        }
    }
}
