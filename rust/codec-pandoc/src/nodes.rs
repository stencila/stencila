use std::collections::HashMap;

use itertools::Itertools;
use pandoc_types::definition::{self as pandoc};

use stencila_codec::{
    DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::*,
};

use crate::{
    blocks::{blocks_from_pandoc, blocks_to_pandoc},
    inlines::{comment_blocks_to_pandoc_inlines, pandoc_inlines_to_blocks},
    meta::{
        inlines_from_meta_inlines, inlines_to_meta_inlines, string_from_meta_value,
        string_to_meta_value,
    },
    shared::{PandocDecodeContext, PandocEncodeContext, PendingComment},
};

pub fn root_to_pandoc(
    root: &Node,
    format: Format,
    options: &Option<EncodeOptions>,
) -> Result<(pandoc::Pandoc, EncodeInfo)> {
    let options = options.clone().unwrap_or_default();
    let mut context = PandocEncodeContext::new(
        format,
        options.render.unwrap_or_default(),
        options.highlight.unwrap_or_default(),
        options.reproducible.unwrap_or_default(),
    );
    let pandoc = node_to_pandoc(root, &mut context)?;

    Ok((
        pandoc,
        EncodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

/// Normalize reply parent ids collected from Pandoc comment spans.
///
/// For DOCX imports, Pandoc can represent several sibling replies as a deeply
/// nested chain of `comment-end` spans even when the source DOCX stores them as
/// direct replies to the same top-level comment. Those nested spans do not
/// encode enough information to recover the original sibling relationships at
/// parse time, so any reply that still has no explicit parent after inline
/// decoding is attached to the single anchored top-level comment.
fn normalize_reply_parents(pending_comments: &mut [PendingComment]) {
    let Some(root_id) = pending_comments
        .iter()
        .find(|comment| comment.parent_pandoc_id.is_none())
        .map(|comment| comment.pandoc_id.clone())
    else {
        return;
    };

    for pending_comment in pending_comments.iter_mut() {
        if pending_comment.pandoc_id != root_id && pending_comment.parent_pandoc_id.is_none() {
            pending_comment.parent_pandoc_id = Some(root_id.clone());
        }
    }
}

fn comment_end_attrs(pandoc_id: &str, parent_pandoc_id: Option<&str>) -> pandoc::Attr {
    let mut attrs = pandoc::Attr::default();
    attrs.classes.push("comment-end".into());
    attrs.attributes.push(("id".into(), pandoc_id.to_string()));
    if let Some(parent_id) = parent_pandoc_id {
        attrs
            .attributes
            .push(("parent".into(), parent_id.to_string()));
    }
    attrs
}

fn prepare_reply_end_spans(
    replies: &Option<Vec<Comment>>,
    parent_pandoc_id: &str,
    next_id: &mut usize,
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Inline> {
    let mut reply_end_spans = Vec::new();

    if let Some(replies) = replies {
        for reply in replies {
            if let Some((_, reply_end)) =
                prepare_comment_for_pandoc(reply, Some(parent_pandoc_id), next_id, context)
            {
                reply_end_spans.push(reply_end);
            }
        }
    }

    reply_end_spans
}

fn comments_from_pending(
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

    let reply_ids = reply_ids(&parent_ids, &pandoc_ids);
    let id_to_index: HashMap<String, usize> = pandoc_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.clone(), index))
        .collect();

    let mut comments = build_flat_comments(pending, &parent_ids, context);
    nest_reply_comments(&mut comments, &parent_ids, &pandoc_ids, &id_to_index);

    if !reply_ids.is_empty() {
        strip_reply_boundaries(content, &reply_ids);
    }

    Some(comments)
}

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
}

fn reply_ids(parent_ids: &[Option<String>], pandoc_ids: &[String]) -> Vec<String> {
    parent_ids
        .iter()
        .zip(pandoc_ids.iter())
        .filter_map(|(parent, id)| parent.as_ref().map(|_| id.clone()))
        .collect()
}

fn build_flat_comments(
    pending_comments: Vec<PendingComment>,
    parent_ids: &[Option<String>],
    context: &mut PandocDecodeContext,
) -> Vec<Comment> {
    pending_comments
        .into_iter()
        .enumerate()
        .map(|(index, pending_comment)| {
            let is_reply = parent_ids[index].is_some();
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
                    start_location: (!is_reply).then(|| format!("#comment-{pandoc_id}-start")),
                    end_location: (!is_reply).then(|| format!("#comment-{pandoc_id}-end")),
                    ..Default::default()
                }),
                ..Default::default()
            }
        })
        .collect()
}

fn nest_reply_comments(
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
        let _ = nest_reply(comments, parent_pandoc_id, reply);
    }
}

fn strip_reply_boundaries(content: &mut [Block], reply_ids: &[String]) {
    let reply_boundary_ids: Vec<String> = reply_ids
        .iter()
        .map(|id| format!("comment-{id}-start"))
        .collect();

    strip_boundaries(content, &reply_boundary_ids);
}

pub fn root_from_pandoc(
    pandoc: pandoc::Pandoc,
    format: Format,
    _options: &Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let mut context = PandocDecodeContext {
        format,
        ..Default::default()
    };
    let node = node_from_pandoc(pandoc, &mut context)?;

    Ok((
        node,
        DecodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

fn node_to_pandoc(node: &Node, context: &mut PandocEncodeContext) -> Result<pandoc::Pandoc> {
    match node {
        Node::Article(article) => article_to_pandoc(article, context),
        _ => bail!("Unsupported node type: {}", node.node_type()),
    }
}

fn node_from_pandoc(pandoc: pandoc::Pandoc, context: &mut PandocDecodeContext) -> Result<Node> {
    let article = article_from_pandoc(pandoc, context);
    Ok(Node::Article(article))
}

fn article_to_pandoc(
    article: &Article,
    context: &mut PandocEncodeContext,
) -> Result<pandoc::Pandoc> {
    // Set repository and commit in context for file link URL encoding
    context.repository = article.options.repository.clone();
    context.commit = article.options.commit.clone();

    let mut meta = HashMap::new();

    if let Some(title) = &article.title {
        meta.insert(
            "title".into(),
            inlines_to_meta_inlines(NodeProperty::Title, title, context),
        );
    }

    if let Some(date) = &article.date_published {
        meta.insert("date".into(), string_to_meta_value(&date.value.to_string()));
    }

    if let Some(keywords) = &article.options.keywords {
        let mut keywords_meta = Vec::new();
        for keyword in keywords {
            keywords_meta.push(string_to_meta_value(keyword));
        }
        meta.insert(
            "keywords".into(),
            pandoc::MetaValue::MetaList(keywords_meta),
        );
    }

    if let Some(r#abstract) = &article.r#abstract
        && let Some(Block::Paragraph(paragraph)) = &r#abstract.first()
    {
        meta.insert(
            "abstract".into(),
            inlines_to_meta_inlines(NodeProperty::Content, &paragraph.content, context),
        );
    }

    prepare_comments_for_pandoc(article, context);

    let blocks = blocks_to_pandoc(NodeProperty::Content, &article.content, context);

    Ok(pandoc::Pandoc { meta, blocks })
}

fn prepare_comments_for_pandoc(article: &Article, context: &mut PandocEncodeContext) {
    context.comment_start_spans.clear();
    context.comment_end_spans.clear();

    let Some(comments) = &article.options.comments else {
        return;
    };

    let mut next_id = 0usize;
    for comment in comments {
        prepare_comment_for_pandoc(comment, None, &mut next_id, context);
    }
}

fn prepare_comment_for_pandoc(
    comment: &Comment,
    parent_pandoc_id: Option<&str>,
    next_id: &mut usize,
    context: &mut PandocEncodeContext,
) -> Option<(String, pandoc::Inline)> {
    let start_location = comment.options.start_location.as_deref();
    let end_location = comment.options.end_location.as_deref();

    let pandoc_id = if let Some(start_location) = start_location {
        comment_pandoc_id(start_location, next_id)?
    } else if let Some(id) = &comment.id {
        id.clone()
    } else {
        let id = next_id.to_string();
        *next_id += 1;
        id
    };

    if start_location.is_none() || end_location.is_none() {
        let end_attrs = comment_end_attrs(&pandoc_id, parent_pandoc_id);
        let mut end_inlines = comment_blocks_to_pandoc_inlines(&comment.content, context);
        end_inlines.append(&mut prepare_reply_end_spans(
            &comment.options.comments,
            &pandoc_id,
            next_id,
            context,
        ));

        return Some((pandoc_id, pandoc::Inline::Span(end_attrs, end_inlines)));
    }

    let start_location = start_location?;
    let end_location = end_location?;

    let start_boundary_id = start_location.strip_prefix('#')?.to_string();
    let end_boundary_id = end_location.strip_prefix('#')?.to_string();

    let mut start_attrs = pandoc::Attr::default();
    start_attrs.classes.push("comment-start".into());
    start_attrs
        .attributes
        .push(("id".into(), pandoc_id.clone()));

    if let Some(author) = comment
        .authors
        .as_ref()
        .map(|authors| authors.iter().map(|author| author.name()).join(";"))
    {
        start_attrs.attributes.push(("author".into(), author));
    }

    if let Some(date) = &comment.date_published {
        start_attrs
            .attributes
            .push(("date".into(), date.value.to_string()));
    }

    let body_inlines = comment_blocks_to_pandoc_inlines(&comment.content, context);
    context.comment_start_spans.insert(
        start_boundary_id,
        pandoc::Inline::Span(start_attrs, body_inlines),
    );

    let reply_end_spans =
        prepare_reply_end_spans(&comment.options.comments, &pandoc_id, next_id, context);

    let end_attrs = comment_end_attrs(&pandoc_id, parent_pandoc_id);

    context.comment_end_spans.insert(
        end_boundary_id.clone(),
        pandoc::Inline::Span(end_attrs, reply_end_spans),
    );

    let end_span = context.comment_end_spans.get(&end_boundary_id)?.clone();

    Some((pandoc_id, end_span))
}

fn comment_pandoc_id(start_location: &str, next_id: &mut usize) -> Option<String> {
    let boundary_id = start_location.strip_prefix('#')?;
    let comment_id = boundary_id.strip_prefix("comment-")?;
    let comment_id = comment_id.strip_suffix("-start")?;

    if let Ok(numeric) = comment_id.parse::<usize>() {
        *next_id = (*next_id).max(numeric + 1);
    }

    Some(comment_id.to_string())
}

fn article_from_pandoc(pandoc: pandoc::Pandoc, context: &mut PandocDecodeContext) -> Article {
    let mut title = None;
    let mut date_published = None;
    let mut keywords = None;
    let mut r#abstract = None;

    for (key, value) in pandoc.meta {
        if key == "title" {
            title = Some(inlines_from_meta_inlines(value, context));
        } else if key == "date" {
            date_published = string_from_meta_value(value).parse().ok();
        } else if key == "keywords" {
            if let Some(pandoc::MetaValue::MetaList(meta_keywords)) = Some(value) {
                keywords = Some(
                    meta_keywords
                        .iter()
                        .map(|keyword| string_from_meta_value(keyword.clone()))
                        .collect(),
                );
            }
        } else if key == "abstract" {
            r#abstract = Some(vec![Block::Paragraph(Paragraph {
                content: inlines_from_meta_inlines(value, context),
                ..Default::default()
            })]);
        }
    }

    let mut content = blocks_from_pandoc(pandoc.blocks, context);
    let comments = comments_from_pending(context, &mut content);

    Article {
        title,
        date_published,
        content,
        r#abstract,
        options: Box::new(ArticleOptions {
            keywords,
            comments,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Recursively find a comment whose `id` matches `target_id`
/// and append `reply` to its `comments`. Returns `Err(reply)` if not found
/// so ownership is returned to the caller.
fn nest_reply(comments: &mut [Comment], target_id: &str, reply: Comment) -> Option<Comment> {
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
            reply = nest_reply(nested, target_id, reply)?;
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
