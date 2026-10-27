use std::path::Path;

use roxmltree::{Document, NS_XML_URI, ParsingOptions};

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::{OptionExt, Result},
    stencila_schema::{
        Article, ArticleOptions, Block, CreativeWorkVariant, Node, Object, Primitive, Section,
    },
};

mod back;
mod body;
mod front;
mod media;
mod names;
mod utilities;

use media::DecodeContext;

use back::decode_back;
use body::decode_blocks;
use front::{decode_article_meta, decode_front};

use self::utilities::{extend_path, record_attrs_lost, record_node_lost};

/// Decode a JATS XML string to a Stencila Schema [`Node`]
///
/// This is the main entry point for decoding. It parses the XML, and then traverses the
/// XML DOM, building an [`Article`] from it (JATS is always treated as an article, not any other
/// type of `CreativeWork`).
pub fn decode(jats: &str, _options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    decode_with_directory(jats, None)
}

pub(super) fn decode_with_directory(
    jats: &str,
    directory: Option<&Path>,
) -> Result<(Node, DecodeInfo)> {
    let dom = Document::parse_with_options(
        jats,
        ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    let mut losses = DecodeContext::new(directory, &dom.root());

    // Find the <article> element
    let root = if dom.root_element().has_tag_name("article") {
        // <article> is the root node
        dom.root_element()
    } else {
        // Search for <article> in DOM (e.g. within a <pmc-articleset>). Only
        // one article is decoded, so any others are lost.
        let mut articles = dom
            .root()
            .descendants()
            .filter(|elem| elem.tag_name().name() == "article");
        let first = articles
            .next()
            .ok_or_eyre("XML document does not have an <article> element")?;
        for _ in articles {
            losses.add(format!(
                "//{}/article",
                dom.root_element().tag_name().name()
            ));
        }
        first
    };

    let article = decode_article("//article", &root, &mut losses);

    let node = Node::Article(article);

    let info = DecodeInfo {
        losses: losses.losses,
        ..Default::default()
    };

    Ok((node, info))
}

/// Decode an `<article>` or `<sub-article>` element to a Stencila [`Article`]
///
/// Used for both so that a sub-article gets the same front matter, body, back
/// matter and nesting treatment as the article containing it.
fn decode_article(path: &str, root: &roxmltree::Node, losses: &mut DecodeContext) -> Article {
    // The JATS `article-type` is retained as the first `genre` of the article
    // because the Stencila Schema has no equivalent property and the type (e.g.
    // `research-article`, `referee-report`) is needed to encode it again
    let mut article = Article {
        id: root.attribute("id").map(String::from),
        options: Box::new(ArticleOptions {
            genre: root
                .attribute("article-type")
                .map(|article_type| vec![article_type.to_string()]),
            extra: root
                .attribute((NS_XML_URI, "lang"))
                .filter(|language| !language.is_empty())
                .map(|language| {
                    Object::from([(
                        "inLanguage",
                        Primitive::String(language.to_ascii_lowercase()),
                    )])
                }),
            ..Default::default()
        }),
        ..Default::default()
    };

    // The `dtd-version` describes the JATS rather than the article
    record_attrs_lost(
        path,
        root,
        ["id", "article-type", "lang", "dtd-version"],
        losses,
    );

    // References are scoped to each article so that those of a sub-article are
    // not mixed with those of the article containing it
    let outer_references = std::mem::take(&mut losses.references);

    let mut content = Vec::new();
    let mut notes = Vec::new();
    let mut parts = Vec::new();

    for child in root.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            // A sub-article uses <front-stub>, whose children are those of an
            // <article-meta> rather than of a <front>
            "front" | "front-stub" => {
                if tag == "front" {
                    decode_front(&child_path, &child, &mut article, losses);
                } else {
                    decode_article_meta(&child_path, &child, &mut article, losses);
                }
                // Take any content added by the front matter so it can be appended after main content
                notes.append(&mut article.content);
            }
            "body" => {
                content = decode_blocks(&child_path, child.children(), losses, 0);
            }
            "floats-group" | "floats-wrap" => {
                content.extend(decode_blocks(&child_path, child.children(), losses, 0));
            }
            "back" => {
                decode_back(&child_path, &child, &mut article, losses);
                // Take any content added by the back matter to notes
                notes.append(&mut article.content);
            }
            "sub-article" => {
                parts.push(CreativeWorkVariant::Article(decode_article(
                    &child_path,
                    &child,
                    losses,
                )));
            }
            _ => record_node_lost(path, &child, losses),
        }
    }

    // Front and back matter sometimes repeat a body section. Only discard an
    // identical section: a shared type does not make two appendices or
    // acknowledgements duplicates, and distinct ids must remain addressable.
    for block in notes {
        if matches!(
            &block,
            Block::Section(Section {
                section_type: Some(_),
                ..
            })
        ) && content.contains(&block)
        {
            continue;
        }
        content.push(block);
    }
    article.content = content;

    if !parts.is_empty() {
        article.options.parts.get_or_insert_default().extend(parts);
    }

    let references = std::mem::replace(&mut losses.references, outer_references);
    if !references.is_empty() {
        article
            .references
            .get_or_insert_default()
            .extend(references);
    }

    article
}
