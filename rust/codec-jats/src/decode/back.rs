use itertools::Itertools;
use roxmltree::Node;

use stencila_codec::{
    Losses,
    stencila_schema::{
        Article, Author, Block, CreativeWorkType, Date, Person, Reference, ReferenceOptions,
        Section, SectionType, shortcuts::t,
    },
};
use stencila_codec_biblio::decode::text_to_reference;

use crate::decode::{
    body::{decode_blocks, decode_sec},
    front::decode_notes,
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
            "ack" => {
                let section = decode_ack(&child_path, &child, losses);
                article.content.push(section);
            }
            "app-group" => {
                let sections = decode_app_group(&child_path, &child, losses);
                article.content.extend(sections);
            }
            "notes" => decode_notes(&child_path, &child, article, losses),
            "ref-list" => decode_ref_list(&child_path, &child, article, losses),
            "sec" => {
                let mut blocks = decode_sec(&child_path, &child, losses, 1);
                article.content.append(&mut blocks);
            }
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode an `<ack>` (acknowledgements section)
fn decode_ack(path: &str, node: &Node, losses: &mut Losses) -> Block {
    record_attrs_lost(path, node, [], losses);

    let content = decode_blocks(path, node.children(), losses, 1);

    Block::Section(Section {
        section_type: Some(SectionType::Acknowledgements),
        content,
        ..Default::default()
    })
}

/// Decode an `<app-group>` (appendix group)
fn decode_app_group(path: &str, node: &Node, losses: &mut Losses) -> Vec<Block> {
    record_attrs_lost(path, node, [], losses);

    let mut secs = Vec::new();

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "app" | "sec" => {
                let block = decode_app(&child_path, &child, losses);
                secs.push(block);
            }
            _ => record_node_lost(path, &child, losses),
        };
    }

    secs
}

/// Decode an `<app>` (appendix) or a `<sec>` in  an `<app-group>`
fn decode_app(path: &str, node: &Node, losses: &mut Losses) -> Block {
    record_attrs_lost(path, node, [], losses);

    let content = decode_blocks(path, node.children(), losses, 1);

    Block::Section(Section {
        section_type: Some(SectionType::Appendix),
        content,
        ..Default::default()
    })
}

/// Decode an `<ref-list>` element
fn decode_ref_list(path: &str, ref_list: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, ref_list, [], losses);

    let mut references = Vec::new();
    for ref_elem in ref_list.children() {
        if ref_elem.tag_name().name() != "ref" {
            continue;
        }
        let Some(id) = ref_elem.attribute("id") else {
            continue;
        };

        let child_path = &extend_path(path, "ref");

        // The <ref> may has <citation>, <element-citation> and/or
        // <mixed-citation> elements within it, or those elements may be nested
        // within a <citation-alternatives>. This attempts to get the an
        // <element-citation> first, falling back to a <citation> and <mixed-citation>
        if let Some(element_citation) = ref_elem
            .descendants()
            .find(|elem| elem.tag_name().name() == "element-citation")
            .or_else(|| {
                ref_elem
                    .descendants()
                    .find(|elem| elem.tag_name().name() == "citation")
            })
            .or_else(|| {
                ref_elem
                    .descendants()
                    .find(|elem| elem.tag_name().name() == "mixed-citation")
            })
        {
            let reference = decode_citation(child_path, id, &element_citation, losses);
            references.push(reference);
        }
    }

    article.references = (!references.is_empty()).then_some(references);
}

/// Decode a `<citation>`, `<element-citation>` or `<mixed-citation>` element
fn decode_citation(path: &str, id: &str, node: &Node, losses: &mut Losses) -> Reference {
    let work_type = node.attribute("publication-type").and_then(|pt| {
        Some(match pt {
            "journal" => CreativeWorkType::Article,
            "book" => CreativeWorkType::Book,
            "data" => CreativeWorkType::Dataset,
            "website" => CreativeWorkType::WebPage,
            _ => return None,
        })
    });

    record_attrs_lost(path, node, ["publication-type"], losses);

    let mut doi = None;
    let mut authors = Vec::new();
    let mut editors = Vec::new();
    let mut date = None;
    let mut title = None;
    let mut source = None;
    let mut volume_number = None;
    let mut issue_number = None;
    let mut page_start = None;
    let mut page_end = None;

    for child in node.children() {
        let child_tag = child.tag_name().name();
        if matches!(child_tag, "name" | "string-name") {
            let person = decode_person(path, &child, losses);
            authors.push(Author::Person(person));
        } else if child_tag == "person-group" {
            let is_authors = matches!(child.attribute("person-group-type"), Some("author") | None);
            for grandchild in child.children() {
                if matches!(grandchild.tag_name().name(), "name" | "string-name") {
                    let person = decode_person(path, &grandchild, losses);
                    if is_authors {
                        authors.push(Author::Person(person))
                    } else {
                        editors.push(person)
                    }
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

    let id = Some(id.into());

    let authors = (!authors.is_empty()).then_some(authors);
    let mut editors = (!editors.is_empty()).then_some(editors);

    let is_part_of = if let Some(source) = source {
        if title.is_none() {
            title = Some(vec![t(source)]);

            None
        } else {
            Some(Box::new(Reference {
                work_type: Some(CreativeWorkType::Periodical),
                title: Some(vec![t(source)]),
                options: Box::new(ReferenceOptions {
                    // Note that we take() here so they are None of the main reference
                    editors: editors.take(),
                    volume_number,
                    issue_number,
                    ..Default::default()
                }),
                ..Default::default()
            }))
        }
    } else {
        None
    };

    // If at least authors and title are not present (can be common for <mixed-citation>)
    // then parse the text of the element as a citation and use that
    if authors.is_none() || title.is_none() {
        let text = node
            .descendants()
            .filter_map(|node| {
                if !node.is_text() {
                    return None;
                }
                let text = node.text()?;
                Some(text.split_whitespace().join(" "))
            })
            .join(" ");

        Reference {
            id,
            ..text_to_reference(&text)
        }
    } else {
        Reference {
            id,
            work_type,
            doi,
            authors,
            date,
            title,
            is_part_of,
            options: Box::new(ReferenceOptions {
                editors,
                page_start,
                page_end,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

/// Decode a `<name>` or `<string-name>`
fn decode_person(path: &str, node: &Node, losses: &mut Losses) -> Person {
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

    Person {
        family_names,
        given_names,
        ..Default::default()
    }
}
