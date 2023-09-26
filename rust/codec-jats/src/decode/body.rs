use roxmltree::Node;

use codec::{
    common::itertools::Itertools,
    schema::{
        shortcuts::{em, p, s, strong, sub, sup, text, u},
        Article, AudioObject, AudioObjectOptions, Block, ImageObject, ImageObjectOptions, Inline,
        Inlines, MediaObject, MediaObjectOptions, ThematicBreak,
    },
    Loss, LossDirection, Losses,
};

/// Decode the `<body>` of an `<article>`
///
/// Iterates over all child elements and either decodes them (by delegating to
/// the corresponding `decode_*` function for the element name), or adds them to
/// losses.
pub(super) fn decode_body(node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let block = match tag {
            "p" => decode_p(&child, losses),
            "hr" => decode_hr(&child, losses),
            _ => {
                if child.is_element() {
                    losses.add(Loss::of_type(LossDirection::Decode, tag))
                }
                continue;
            }
        };
        article.content.push(block)
    }
}

/// Decode a `<p>` to a [`Block::Paragraph`]
fn decode_p(node: &Node, losses: &mut Losses) -> Block {
    record_attributes_lost(node, losses, []);

    p(decode_inlines(node, losses))
}

/// Decode a `<hr>` to a [`Block::ThematicBreak`]
fn decode_hr(node: &Node, losses: &mut Losses) -> Block {
    record_attributes_lost(node, losses, []);

    Block::ThematicBreak(ThematicBreak::new())
}

/// Decode inline content nodes
///
/// Iterates over all child elements and either decodes them, or adds them to
/// losses.
fn decode_inlines(node: &Node, losses: &mut Losses) -> Inlines {
    let mut inlines = Inlines::new();
    for child in node.children() {
        let inline = if child.is_text() {
            text(child.text().unwrap_or_default())
        } else {
            let tag = child.tag_name().name();
            match tag {
                "inline-media" | "inline-graphic" => decode_inline_media(&child, losses),
                _ => {
                    record_attributes_lost(&child, losses, []);
                    match tag {
                        "bold" => strong(decode_inlines(&child, losses)),
                        "italic" => em(decode_inlines(&child, losses)),
                        "strike" => s(decode_inlines(&child, losses)),
                        "sub" => sub(decode_inlines(&child, losses)),
                        "sup" => sup(decode_inlines(&child, losses)),
                        "underline" => u(decode_inlines(&child, losses)),
                        _ => {
                            if child.is_element() {
                                losses.add(Loss::of_type(LossDirection::Decode, tag))
                            }
                            continue;
                        }
                    }
                }
            }
        };
        inlines.push(inline);
    }
    inlines
}

/// Decode a `<inline-media>` to a [`Inline::AudioObject`], [`Inline::ImageObject`],
/// or [`Inline::VideoObject`]
///
/// Resolves the destination type based on the `mimetype` attribute of the element.
fn decode_inline_media(node: &Node, losses: &mut Losses) -> Inline {
    let content_url = node.attribute("href").map(String::from).unwrap_or_default();

    let mime_type = node.attribute("mimetype").map(String::from);
    let mime_subtype = node.attribute("mime-subtype").map(String::from);
    let media_type = match (&mime_type, &mime_subtype) {
        (Some(r#type), Some(subtype)) => Some(format!("{type}/{subtype}")),
        (Some(r#type), None) => Some(r#type.clone()),
        _ => None,
    };

    record_attributes_lost(node, losses, ["href", "mimetype", "mime-subtype"]);

    let mut alternate_names = None;
    let mut description = None;
    for child in node.children() {
        let tag = child.tag_name().name();
        match tag {
            "alt-text" => alternate_names = child.text().map(|content| vec![content.to_string()]),
            "long-desc" => description = child.text().map(|content| vec![p([text(content)])]),
            _ => {
                if child.is_element() {
                    losses.add(Loss::of_type(LossDirection::Decode, tag))
                }
            }
        }
    }

    match mime_type.as_deref() {
        Some("audio") => Inline::AudioObject(AudioObject {
            content_url,
            media_type,
            options: Box::new(AudioObjectOptions {
                alternate_names,
                description,
                ..Default::default()
            }),
            ..Default::default()
        }),
        Some("inline") => Inline::ImageObject(ImageObject {
            content_url,
            media_type,
            options: Box::new(ImageObjectOptions {
                alternate_names,
                description,
                ..Default::default()
            }),
            ..Default::default()
        }),
        Some("video") => Inline::AudioObject(AudioObject {
            content_url,
            media_type,
            options: Box::new(AudioObjectOptions {
                alternate_names,
                description,
                ..Default::default()
            }),
            ..Default::default()
        }),
        _ => Inline::MediaObject(MediaObject {
            content_url,
            media_type,
            options: Box::new(MediaObjectOptions {
                alternate_names,
                description,
                ..Default::default()
            }),
            ..Default::default()
        }),
    }
}

/// Record the attributes of a node that are lost when decoding
///
/// Pass the names of the of the attributes (not namespaced) that _are_
/// decoded in the `not_lost` parameter.
fn record_attributes_lost<'lt, I>(node: &Node, losses: &mut Losses, not_lost: I)
where
    I: IntoIterator<Item = &'lt str>,
{
    let not_lost = not_lost.into_iter().collect_vec();
    for attribute in node.attributes() {
        let name = attribute.name();
        if not_lost.contains(&name) {
            losses.push(Loss::of_property(
                LossDirection::Decode,
                node.tag_name().name(),
                name,
            ));
        }
    }
}
