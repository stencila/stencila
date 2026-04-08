use std::collections::HashMap;

use pandoc_types::definition::{self as pandoc};

use stencila_codec::{
    DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::*,
};

use crate::{
    blocks::{blocks_from_pandoc, blocks_to_pandoc},
    inlines::pandoc_inlines_to_blocks,
    meta::{
        inlines_from_meta_inlines, inlines_to_meta_inlines, string_from_meta_value,
        string_to_meta_value,
    },
    shared::{PandocDecodeContext, PandocEncodeContext},
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

    let blocks = blocks_to_pandoc(NodeProperty::Content, &article.content, context);

    Ok(pandoc::Pandoc { meta, blocks })
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

    // Build Comment nodes from pending comments collected during inline decoding
    let comments = if context.pending_comments.is_empty() {
        None
    } else {
        let pending: Vec<_> = context.pending_comments.drain(..).collect();

        // Collect parent_pandoc_id and pandoc_id before consuming pending
        let parent_ids: Vec<Option<String>> =
            pending.iter().map(|c| c.parent_pandoc_id.clone()).collect();
        let pandoc_ids: Vec<String> = pending.iter().map(|c| c.pandoc_id.clone()).collect();

        // Collect the set of reply comment IDs — these don't get their own boundaries
        let reply_ids: Vec<String> = parent_ids
            .iter()
            .zip(pandoc_ids.iter())
            .filter_map(|(parent, id)| parent.as_ref().map(|_| id.clone()))
            .collect();

        // Build a map from pandoc_id -> index
        let id_to_index: HashMap<String, usize> = pandoc_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        // Build all comments initially as a flat list
        let mut comments: Vec<Comment> = pending
            .into_iter()
            .enumerate()
            .map(|(i, pending)| {
                let body_blocks = pandoc_inlines_to_blocks(pending.body_inlines, context);
                let is_reply = parent_ids[i].is_some();

                let authors = pending.author.map(|name| {
                    vec![Author::Person(Person {
                        given_names: Some(vec![name]),
                        ..Default::default()
                    })]
                });

                let date_published = pending.date.and_then(|d| d.parse().ok());

                Comment {
                    content: body_blocks,
                    authors,
                    date_published,
                    options: Box::new(CommentOptions {
                        // Reply comments share the parent's range — no location of their own
                        start_location: if is_reply {
                            None
                        } else {
                            Some(format!("#comment-{}-start", pending.pandoc_id))
                        },
                        end_location: if is_reply {
                            None
                        } else {
                            Some(format!("#comment-{}-end", pending.pandoc_id))
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            })
            .collect();

        // Nest reply comments under their parent's `comments` property.
        // Collect reply indices in forward order, then remove in reverse
        // (to keep indices stable), then re-reverse so siblings stay ordered.
        let reply_indices: Vec<(usize, usize)> = (0..comments.len())
            .filter_map(|i| {
                let pid = parent_ids[i].as_ref()?;
                let &parent_idx = id_to_index.get(pid)?;
                Some((i, parent_idx))
            })
            .collect();

        let mut replies: Vec<(usize, Comment)> = Vec::new();
        for &(i, parent_idx) in reply_indices.iter().rev() {
            replies.push((parent_idx, comments.remove(i)));
        }
        replies.reverse();

        // Insert each reply into its parent, searching recursively so that
        // deeply nested replies (A → B → C) find their parent even after
        // it has already been nested.
        for (original_parent_idx, reply) in replies {
            let parent_pandoc_id = &pandoc_ids[original_parent_idx];
            let target_loc = format!("#comment-{parent_pandoc_id}-start");
            // If nest_reply returns Err the parent wasn't found — reply is dropped
            let _ = nest_reply(&mut comments, &target_loc, reply);
        }

        // Remove Boundary nodes for reply comments from inline content
        if !reply_ids.is_empty() {
            let reply_boundary_ids: Vec<String> = reply_ids
                .iter()
                .map(|id| format!("comment-{id}-start"))
                .collect();
            strip_boundaries(&mut content, &reply_boundary_ids);
        }

        Some(comments)
    };

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

/// Recursively find a comment whose `start_location` matches `target_loc`
/// and append `reply` to its `comments`. Returns `Err(reply)` if not found
/// so ownership is returned to the caller.
fn nest_reply(comments: &mut [Comment], target_loc: &str, reply: Comment) -> Result<(), Comment> {
    // Two-pass: first check direct children, then recurse.
    // This avoids borrow conflicts from iterating while recursing.
    for comment in comments.iter_mut() {
        if comment.options.start_location.as_deref() == Some(target_loc) {
            comment
                .options
                .comments
                .get_or_insert_with(Vec::new)
                .push(reply);
            return Ok(());
        }
    }
    let mut reply = reply;
    for comment in comments.iter_mut() {
        if let Some(nested) = &mut comment.options.comments {
            reply = match nest_reply(nested, target_loc, reply) {
                Ok(()) => return Ok(()),
                Err(r) => r,
            };
        }
    }
    Err(reply)
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
