use itertools::Itertools;
use roxmltree::Node;

use stencila_codec::{
    Losses,
    stencila_schema::{Article, Author, Date, Person, Reference, ReferenceOptions, shortcuts::t},
};

use super::{
    body::decode_inlines,
    utilities::{extend_path, record_attrs_lost, record_node_lost, split_given_names},
};

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
    let references = node
        .children()
        .filter_map(|child| {
            if let ("ref", Some(id)) = (child.tag_name().name(), child.attribute("id")) {
                Some((id, child))
            } else {
                None
            }
        })
        .flat_map(|(id, child)| {
            let child_path = &extend_path(path, "ref");
            child
                .children()
                .filter(|grandchild| grandchild.tag_name().name().contains("citation"))
                .map(|grandchild| decode_citation(child_path, id, &grandchild, losses))
                .collect_vec()
        })
        .collect_vec();

    record_attrs_lost(path, node, [], losses);

    article.references = (!references.is_empty()).then_some(references);
}

/// Decode any node that contains `<citation>` element
fn decode_citation(path: &str, id: &str, node: &Node, losses: &mut Losses) -> Reference {
    record_attrs_lost(path, node, [], losses);

    let mut doi = None;
    let mut authors = Vec::new();
    let mut date = None;
    let mut title = None;
    let mut source = None;
    let mut volume_number = None;
    let mut issue_number = None;
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
        } else if child_tag == "year" {
            date = child.text().map(|year| Date::new(year.to_string()));
        } else if child_tag.to_string().contains("title") {
            title = Some(decode_inlines(path, child.children(), losses))
        } else if child_tag == "source" {
            source = child.text().map(String::from);
        } else if child_tag == "volume" {
            volume_number = child.text().map(Into::into);
        } else if child_tag == "issue" {
            issue_number = child.text().map(Into::into);
        } else if child_tag == "fpage" {
            page_start = child.text().map(Into::into);
        } else if child_tag == "lpage" {
            page_end = child.text().map(Into::into);
        } else if child_tag == "pub-id" {
            let id_type = child.attribute("pub-id-type");
            if id_type == Some("doi") {
                doi = child.text().map(String::from);
            }
        }
    }

    let authors = (!authors.is_empty()).then_some(authors);

    let is_part_of = source
        .map(|title| Reference {
            title: Some(vec![t(title)]),
            options: Box::new(ReferenceOptions {
                volume_number,
                issue_number,
                ..Default::default()
            }),
            ..Default::default()
        })
        .map(Box::new);

    Reference {
        id: Some(id.into()),
        doi,
        authors,
        date,
        title,
        is_part_of,
        options: Box::new(ReferenceOptions {
            page_start,
            page_end,
            ..Default::default()
        }),
        ..Default::default()
    }
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
        } else if tag == "given-names"
            && let Some(value) = node.text()
        {
            given_names.append(&mut split_given_names(value));
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
