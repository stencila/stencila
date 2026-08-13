use roxmltree::{Document, ParsingOptions};

use stencila_codec::{
    DecodeInfo, DecodeOptions, Losses,
    eyre::{OptionExt, Result},
    stencila_schema::{Article, Block, CreativeWorkVariant, Node, Section},
};

mod back;
mod body;
mod front;
mod utilities;

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
    let mut losses = Losses::none();

    let dom = Document::parse_with_options(
        jats,
        ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    // Find the <article> element
    let root = if dom.root_element().has_tag_name("article") {
        // <article> is the root node
        dom.root_element()
    } else {
        // Search for <article> in DOM (e.g. within a <pmc-articleset>)
        dom.root()
            .descendants()
            .find(|elem| elem.tag_name().name() == "article")
            .ok_or_eyre("XML document does not have an <article> element")?
    };

    let article = decode_article("//article", &root, &mut losses);

    let node = Node::Article(article);

    let info = DecodeInfo {
        losses,
        ..Default::default()
    };

    Ok((node, info))
}

/// Decode an `<article>` or `<sub-article>` element to a Stencila [`Article`]
///
/// Used for both so that a sub-article gets the same front matter, body, back
/// matter and nesting treatment as the article containing it.
fn decode_article(path: &str, root: &roxmltree::Node, losses: &mut Losses) -> Article {
    let mut article = Article {
        id: root.attribute("id").map(String::from),
        ..Default::default()
    };
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
            "back" => {
                decode_back(&child_path, &child, &mut article, losses);
                // Take any content added by the back matter to notes
                notes.append(&mut article.content);
            }
            "sub-article" => {
                parts.push(CreativeWorkVariant::Article(decode_sub_article(
                    &child_path,
                    &child,
                    losses,
                )));
            }
            _ => record_node_lost(path, &child, losses),
        }
    }

    // Append any front or back matter content (e.g. <notes>) but not if the same section
    // already exists (e.g. sometime conflict of interest section is in frontmatter notes and body)
    for block in notes {
        if let Block::Section(Section {
            section_type: Some(section_type),
            ..
        }) = block
        {
            if !content.iter().any(|block| match block {
                Block::Section(section) => section.section_type == Some(section_type),
                _ => false,
            }) {
                content.push(block);
            }
        } else {
            content.push(block);
        }
    }
    article.content = content;

    if !parts.is_empty() {
        match &mut article.options.parts {
            Some(existing) => existing.extend(parts),
            None => article.options.parts = Some(parts),
        }
    }

    article
}

/// Decode a `<sub-article>` element to a Stencila [`Article`]
///
/// The JATS `article-type` is retained as the first `genre` of the article
/// because the Stencila Schema has no equivalent property and the type (e.g.
/// `referee-report`, `author-comment`) is needed to encode the sub-article
/// again.
fn decode_sub_article(path: &str, node: &roxmltree::Node, losses: &mut Losses) -> Article {
    record_attrs_lost(path, node, ["id", "article-type"], losses);

    let mut article = decode_article(path, node, losses);

    article.id = node.attribute("id").map(String::from);
    article.options.genre = node
        .attribute("article-type")
        .map(|article_type| vec![article_type.to_string()]);

    article
}
