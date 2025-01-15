use roxmltree::{Document, ParsingOptions};

use codec::{
    common::eyre::{bail, Result},
    schema::{Article, Node},
    DecodeInfo, DecodeOptions, Losses,
};

mod back;
mod body;
mod front;
mod utilities;

use back::decode_back;
use body::decode_body;
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
    let root = if !dom.root_element().has_tag_name("article") {
        bail!("XML document does not have an <article> root element")
    } else {
        dom.root_element()
    };

    let path = "//article";
    for child in root.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "front" => decode_front(&child_path, &child, &mut article, &mut losses),
            "body" => decode_body(&child_path, &child, &mut article, &mut losses),
            "back" => decode_back(&child_path, &child, &mut article, &mut losses),
            _ => record_node_lost(path, &child, &mut losses),
        }
    }

    let node = Node::Article(article);

    let info = DecodeInfo {
        losses,
        ..Default::default()
    };

    Ok((node, info))
}
