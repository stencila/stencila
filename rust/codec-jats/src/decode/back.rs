use roxmltree::Node;

use codec::{
    schema::{
        Article, ArticleOptions, Author, CreativeWorkTypeOrText, IntegerOrString, Organization,
        Person, PersonOrOrganization, Primitive, PropertyValue, PropertyValueOrString,
        StringOrNumber,
    },
    Losses,
};

use crate::decode::body::decode_inlines;

use super::utilities::{extend_path, record_attrs_lost, record_node_lost};

/// Decode the `<back>` of an `<article>`
pub(super) fn decode_back(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "ref-list" => decode_ref_list(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode an `<ref-list>` element
fn decode_ref_list(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let references: Vec<CreativeWorkTypeOrText> = node
        .children()
        .filter(|child| child.tag_name().name() == "ref")
        .flat_map(|child| {
            let child_path = &extend_path(path, "ref");
            child
                .children()
                .filter(|grandchild| {
                    grandchild
                        .tag_name()
                        .name()
                        .to_string()
                        .contains("citation")
                })
                .map(|grandchild| decode_citation(child_path, &grandchild, losses))
                .collect::<Vec<CreativeWorkTypeOrText>>()
        })
        .collect();

    record_attrs_lost(path, node, [], losses);

    let references = (!references.is_empty()).then_some(references);

    article.references = references;
}

/// Decode any node that contains `<citation>` element
fn decode_citation(path: &str, node: &Node, losses: &mut Losses) -> CreativeWorkTypeOrText {
    record_attrs_lost(path, node, [], losses);

    let mut authors = Vec::new();
    let mut identifiers = Vec::new();
    let mut title = None;
    let mut publisher = None;
    let mut version = None;
    let mut page_start = None;
    let mut page_end = None;

    for child in node.children() {
        let child_tag = child.tag_name().name();
        if child_tag == "string-name" {
            authors.push(decode_person(path, &child, losses))
        } else if child_tag == "person-group" {
            for grandchild in child.children() {
                let grandchild_tag = grandchild.tag_name().name();
                if grandchild_tag == "name" {
                    authors.push(decode_person(path, &grandchild, losses))
                }
            }
        } else if child_tag.to_string().contains("title") {
            title = Some(decode_inlines(path, child.children(), losses));
        } else if child_tag == "source" {
            publisher = Some(PersonOrOrganization::Organization(Organization {
                name: child.text().map(str::to_string),
                ..Default::default()
            }));
        } else if child_tag == "volume" {
            if let Some(value) = child.text() {
                if let Ok(num) = value.parse::<f64>() {
                    version = Some(StringOrNumber::Number(num))
                } else {
                    version = Some(StringOrNumber::String(value.into()))
                }
            }
        } else if child_tag == "fpage" {
            if let Some(value) = child.text() {
                if let Ok(num) = value.parse::<i64>() {
                    page_start = Some(IntegerOrString::Integer(num))
                } else {
                    page_start = Some(IntegerOrString::String(value.into()))
                }
            }
        } else if child_tag == "lpage" {
            if let Some(value) = child.text() {
                if let Ok(num) = value.parse::<i64>() {
                    page_end = Some(IntegerOrString::Integer(num))
                } else {
                    page_end = Some(IntegerOrString::String(value.into()))
                }
            }
        } else if child_tag == "pub-id" {
            if let Some(value) = child.text() {
                let child_type = child.attribute("pub-id-type");
                if child_type == Some("doi") {
                    identifiers.push(PropertyValueOrString::PropertyValue(PropertyValue {
                        property_id: Some("https://registry.identifiers.org/registry/doi".into()),
                        value: Primitive::String(value.into()),
                        ..Default::default()
                    }))
                } else if child_type == Some("pmid") {
                    identifiers.push(PropertyValueOrString::PropertyValue(PropertyValue {
                        property_id: Some("https://registry.identifiers.org/registry/pubmed".into()),
                        value: Primitive::String(value.into()),
                        ..Default::default()
                    }))
                }
            }
        } else if child_tag == "year" {
            //TODO
        }
    }

    let authors = (!authors.is_empty()).then_some(authors);
    let identifiers = (!identifiers.is_empty()).then_some(identifiers);

    CreativeWorkTypeOrText::CreativeWorkType(codec::schema::CreativeWorkType::Article(Article {
        authors,
        title,
        options: Box::new(ArticleOptions {
            identifiers,
            publisher,
            version,
            page_start,
            page_end,
            ..Default::default()
        }),
        ..Default::default()
    }))
}

/// Decode an `<name> and <string-name>`
fn decode_person(path: &str, node: &Node, losses: &mut Losses) -> Author {
    record_attrs_lost(path, node, [], losses);

    let mut family_names = Vec::new();
    let mut given_names = Vec::new();

    for node in node.children() {
        let tag = node.tag_name().name();
        if tag == "surname" {
            if let Some(value) = node.text() {
                family_names.push(value.to_string());
            }
        } else if tag == "given-names" {
            if let Some(value) = node.text() {
                given_names.push(value.to_string());
            }
        }
    }

    let family_names = (!family_names.is_empty()).then_some(family_names);
    let given_names = (!given_names.is_empty()).then_some(given_names);

    Author::Person(Person {
        family_names,
        given_names,
        ..Default::default()
    })
}
