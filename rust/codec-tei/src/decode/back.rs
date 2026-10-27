use std::iter::once;

use roxmltree::Node;

use stencila_codec::stencila_schema::{
    Article, Block, GrantOrMonetaryGrant, MonetaryGrant, MonetaryGrantOptions, Section, SectionType,
};

use super::{
    Context,
    bibl::{decode_list_bibl, push_identifier},
    non_empty_text, path,
    text::{Divisions, decode_blocks, decode_divisions},
    xml_id,
};

/// Decode the `<back>` of a `<text>`
///
/// Returns the sections of the back matter, to be appended to the content of
/// the article, and decodes the reference list and grants onto the article.
pub(super) fn decode_back(node: &Node, article: &mut Article, context: &mut Context) -> Vec<Block> {
    context.lost_attrs(node, &[]);

    let mut blocks = Vec::new();
    for child in node.children() {
        match child.tag_name().name() {
            "div" if child.attribute("type") == Some("references") => {
                decode_references(&child, context)
            }
            "div" => blocks.append(&mut decode_back_div(&child, context)),
            "listOrg" => decode_list_org(&child, article, context),
            _ if child.is_element() => blocks.append(&mut decode_blocks(once(child), 0, context)),
            _ => context.lost(&child),
        }
    }

    blocks
}

/// Decode a `<div>` of the back matter
///
/// GROBID types the divisions of the back matter, and each typed division
/// becomes a [`Section`] of the corresponding type, holding the headings and
/// content of the divisions within it, as the back matter sections decoded
/// from JATS do. An untyped division is decoded as a division of the body is.
fn decode_back_div(node: &Node, context: &mut Context) -> Vec<Block> {
    let Some(div_type) = node.attribute("type") else {
        return decode_divisions(once(*node), 0, Divisions::Sections, context);
    };

    let section_type = match div_type {
        "acknowledgement" | "acknowledgment" | "acknowledgements" => {
            Some(SectionType::Acknowledgements)
        }
        "annex" => Some(SectionType::Appendix),
        "availability" => Some(SectionType::DataAvailability),
        "conflict" => Some(SectionType::CompetingInterests),
        "contribution" => Some(SectionType::AuthorContributions),
        "funding" => Some(SectionType::Funding),
        _ => SectionType::from_text(div_type).ok(),
    };
    if section_type.is_none() {
        context.lost_label(format!("{}[@type='{div_type}']", path(node)));
    }

    context.lost_attrs(node, &["id", "coords", "type"]);

    let content = decode_divisions(node.children(), 0, Divisions::Flattened, context);
    if content.is_empty() {
        context.lost_attrs_named(node, &["id", "coords"]);
        return Vec::new();
    }

    vec![Block::Section(context.located(
        node,
        Section {
            id: xml_id(node).map(String::from),
            section_type,
            content,
            ..Default::default()
        },
    ))]
}

/// Decode the reference list
///
/// The `<head>` of the references (e.g. "References") is regenerated when
/// rendering so is not a loss.
fn decode_references(node: &Node, context: &mut Context) {
    context.lost_attrs(node, &["type"]);

    for child in node.children() {
        match child.tag_name().name() {
            "listBibl" => {
                let mut references = decode_list_bibl(&child, context);
                context.references.append(&mut references);
            }
            "head" => {}
            _ => context.lost(&child),
        }
    }
}

/// Decode a `<listOrg type="funding">` to the grants that funded the article
///
/// Each `<org>` is a grant, with its number in an `<idno type="grant-number">`,
/// kept as an `award-id` identifier as it is when decoding a JATS `<award-id>`.
/// The funders of a grant are those `<funder>`s of the header that refer to it.
fn decode_list_org(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &["type"]);

    for child in node.children() {
        if !child.has_tag_name("org") {
            context.lost(&child);
            continue;
        }

        context.lost_attrs(&child, &["id", "type"]);

        let id = xml_id(&child).map(String::from);
        let mut identifiers = None;
        for grandchild in child.children() {
            if grandchild.has_tag_name("idno") {
                context.lost_attrs(&grandchild, &["type"]);
                if let Some(value) = non_empty_text(&grandchild) {
                    let id_type = match grandchild.attribute("type") {
                        Some("grant-number") | None => "award-id",
                        Some(id_type) => id_type,
                    };
                    push_identifier(&mut identifiers, Some(id_type.into()), &value);
                }
            } else {
                context.lost(&grandchild);
            }
        }

        let funders = id
            .as_ref()
            .map(|id| {
                context
                    .funders
                    .iter()
                    .filter(|(funder_ref, ..)| funder_ref == id)
                    .map(|(.., funder)| funder.clone())
                    .collect::<Vec<_>>()
            })
            .filter(|funders| !funders.is_empty());

        if identifiers.is_none() && funders.is_none() {
            continue;
        }

        article.options.funded_by.get_or_insert_default().push(
            GrantOrMonetaryGrant::MonetaryGrant(MonetaryGrant {
                id,
                options: Box::new(MonetaryGrantOptions {
                    identifiers,
                    funders,
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );
    }
}
