use roxmltree::{Document, ParsingOptions};

use stencila_codec::{
    DecodeInfo, DecodeOptions, Losses,
    eyre::{OptionExt, Result},
    stencila_schema::{Article, Block, Node, Section},
};

mod back;
mod body;
mod front;
mod utilities;

use back::decode_back;
use body::decode_blocks;
use front::decode_front;

use self::utilities::{extend_path, record_node_lost};

/// Decode a JATS XML string to a Stencila Schema [`Node`]
///
/// This is the main entry point for decoding. It parses the XML, and then traverses the
/// XML DOM, building an [`Article`] from it (JATS is always treated as an article, not any other
/// type of `CreativeWork`).
pub fn decode(jats: &str, _options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    let mut article = Article::default();
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

    let path = "//article";
    let mut content = Vec::new();
    let mut notes = Vec::new();
    for child in root.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "front" => {
                decode_front(&child_path, &child, &mut article, &mut losses);
                // Take any content added by the front matter so it can be appended after main content
                notes.append(&mut article.content);
            }
            "body" => {
                content = decode_blocks(&child_path, child.children(), &mut losses, 0);
            }
            "back" => {
                decode_back(&child_path, &child, &mut article, &mut losses);
                // Take any content added by the back matter to notes
                notes.append(&mut article.content);
            }
            _ => record_node_lost(path, &child, &mut losses),
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

    let node = Node::Article(article);

    let info = DecodeInfo {
        losses,
        ..Default::default()
    };

    Ok((node, info))
}
