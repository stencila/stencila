use roxmltree::Document;

use codec::{
    common::eyre::{bail, Result},
    schema::{self, Article},
    DecodeOptions, Loss, LossDirection, Losses,
};

mod back;
mod body;
mod front;

use back::decode_back;
use body::decode_body;
use front::decode_front;

/// Decode a JATS XML string to a Stencila Schema [`Node`]
///
/// This is the main entry point for decoding. It parses the XML, and then traverses the
/// XML DOM, building an [`Article`] from it (JATS is always treated as an article, not any other
/// type of `CreativeWork`).
pub(super) fn decode(str: &str, _options: Option<DecodeOptions>) -> Result<(schema::Node, Losses)> {
    let mut article = Article::default();
    let mut losses = Losses::default();

    let dom = Document::parse(str)?;
    let root = if !dom.root_element().has_tag_name("article") {
        bail!("XML document does not have an <article> root element")
    } else {
        dom.root_element()
    };

    for child in root.children() {
        let tag = child.tag_name().name();
        match tag {
            "front" => decode_front(&child, &mut article, &mut losses),
            "body" => decode_body(&child, &mut article, &mut losses),
            "back" => decode_back(&child, &mut article, &mut losses),
            _ => {
                if child.is_element() {
                    losses.add(Loss::of_type(LossDirection::Decode, tag))
                }
            }
        }
    }

    Ok((schema::Node::Article(article), losses))
}
