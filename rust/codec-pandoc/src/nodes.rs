use std::collections::HashMap;

use pandoc_types::definition::{self as pandoc};

use codec::{
    common::eyre::{bail, Result},
    format::Format,
    schema::*,
    DecodeInfo, EncodeInfo,
};

use crate::{
    blocks::{blocks_from_pandoc, blocks_to_pandoc},
    meta::{
        inlines_from_meta_inlines, inlines_to_meta_inlines, string_from_meta_value,
        string_to_meta_value,
    },
    shared::{PandocDecodeContext, PandocEncodeContext},
};

pub fn root_to_pandoc(root: &Node, format: Format) -> Result<(pandoc::Pandoc, EncodeInfo)> {
    let mut context = PandocEncodeContext {
        format,
        ..Default::default()
    };
    let pandoc = node_to_pandoc(root, &mut context)?;

    Ok((
        pandoc,
        EncodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

pub fn root_from_pandoc(pandoc: pandoc::Pandoc, format: Format) -> Result<(Node, DecodeInfo)> {
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
    let mut meta = HashMap::new();

    if let Some(title) = &article.title {
        meta.insert("title".into(), inlines_to_meta_inlines(title, context));
    }

    if let Some(date) = &article.date_published {
        meta.insert("date".into(), string_to_meta_value(&date.value.to_string()));
    }
    if let Some(keywords) = &article.keywords {
        let mut keywords_meta = Vec::new();
        for keyword in keywords {
            keywords_meta.push(string_to_meta_value(keyword));
        }
        meta.insert(
            "keywords".into(),
            pandoc::MetaValue::MetaList(keywords_meta),
        );
    }
    if let Some(r#abstract) = &article.r#abstract {
        if let Some(Block::Paragraph(paragraph)) = &r#abstract.first() {
            meta.insert(
                "abstract".into(),
                inlines_to_meta_inlines(&paragraph.content, context),
            );
        }
    }

    let blocks = blocks_to_pandoc(&article.content, context);

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

    let content = blocks_from_pandoc(pandoc.blocks, context);

    Article {
        title,
        date_published,
        content,
        keywords,
        r#abstract,
        ..Default::default()
    }
}
