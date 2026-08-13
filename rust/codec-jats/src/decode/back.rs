use itertools::Itertools;
use roxmltree::Node;

use stencila_codec::{
    Losses,
    stencila_schema::{
        Article, Author, Block, CreativeWorkType, Date, Inline, IntegerOrString, Organization,
        OrganizationOptions, Person, PersonOrOrganization, PostalAddressOrString,
        PropertyValueOrString, Reference, ReferenceOptions, Section, SectionType, StringOrNumber,
    },
};
use stencila_codec_biblio::decode::text_to_reference;

use crate::decode::{
    body::{decode_blocks, decode_sec},
    front::{decode_notes, push_identifier},
};

use super::{
    body::decode_inlines,
    utilities::{XLINK, extend_path, record_attrs_lost, record_node_lost, split_given_names},
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
            "fn-group" => {
                let mut blocks = decode_blocks(path, std::iter::once(child), losses, 1);
                article.content.append(&mut blocks);
            }
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
///
/// # Bibliographic reference contract
///
/// A JATS citation is flat: it spells the title of a work, the title of the
/// journal or book containing it, and the volume and issue of that container,
/// as siblings. A [`Reference`] is nested. Both sides of this codec follow one
/// mapping between the two so that nothing is written by one and then not read
/// by the other:
///
/// | JATS                              | Schema                              |
/// |-----------------------------------|-------------------------------------|
/// | `@publication-type`               | `workType`                          |
/// | `article-title`, `chapter-title`  | `title`                             |
/// | `source`, with a title present    | `isPartOf.title`                    |
/// | `source`, with no title           | `title`                             |
/// | `volume`, `issue`                 | container's, else the reference's    |
/// | `fpage`, `lpage`, `page-range`    | the reference's                     |
/// | `person-group[@type='editor']`    | container's for a book, else the reference's |
/// | `publisher-name`, `publisher-loc` | container's `publisher`, else the reference's |
/// | `edition`                         | `version`                           |
/// | `pub-id[@pub-id-type='doi']`      | `doi`                               |
/// | other `pub-id`, `elocation-id`    | `identifiers`, keyed by their type  |
/// | `ext-link`, `uri`                 | `url`                               |
///
/// Fields decoded from elements are authoritative. Parsing the text of a
/// citation only fills in fields that no element supplied, and the text itself
/// is kept only when the reference can not be rendered without it.
fn decode_ref_list(path: &str, ref_list: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, ref_list, [], losses);

    let mut references = Vec::new();
    for ref_elem in ref_list.children() {
        if ref_elem.tag_name().name() != "ref" {
            record_node_lost(path, &ref_elem, losses);
            continue;
        }

        let ref_path = &extend_path(path, "ref");
        record_attrs_lost(ref_path, &ref_elem, ["id"], losses);

        let Some(id) = ref_elem.attribute("id") else {
            // Without an ID the reference cannot be the target of a citation
            losses.add(format!("{ref_path}/@id"));
            continue;
        };

        // The <ref> may have <citation>, <element-citation> and/or
        // <mixed-citation> elements within it, or those elements may be nested
        // within a <citation-alternatives>. The most structured of them is used
        // for the fields of the reference and, when a different one has richer
        // raw text, that text is used to fill in any fields that are missing.
        let citations = ref_elem
            .descendants()
            .filter(|elem| {
                matches!(
                    elem.tag_name().name(),
                    "element-citation" | "citation" | "mixed-citation"
                )
            })
            .collect_vec();

        let structured = citations
            .iter()
            .find(|elem| elem.tag_name().name() == "element-citation")
            .or_else(|| {
                citations
                    .iter()
                    .find(|elem| elem.tag_name().name() == "citation")
            })
            .or_else(|| {
                citations
                    .iter()
                    .find(|elem| elem.tag_name().name() == "mixed-citation")
            });

        let Some(structured) = structured.copied() else {
            losses.add(ref_path.clone());
            continue;
        };

        // A <mixed-citation> keeps the punctuation and labelling that a text
        // parser needs, so is preferred as the source of raw citation text
        let raw_text = citations
            .iter()
            .find(|elem| elem.tag_name().name() == "mixed-citation")
            .copied()
            .unwrap_or(structured);

        // Report any citation that contributes neither fields nor text
        for other in citations
            .iter()
            .filter(|elem| **elem != structured && **elem != raw_text)
        {
            record_node_lost(ref_path, other, losses);
        }

        // Report the citation against its own path, rather than that of the
        // surrounding <ref>
        let citation_path = &extend_path(ref_path, structured.tag_name().name());
        let reference = decode_citation(citation_path, id, &structured, &raw_text, losses);
        references.push(reference);
    }

    article.references = (!references.is_empty()).then_some(references);
}

/// The bibliographic fields that the children of a citation element can supply
///
/// These are collected before being assembled into a [`Reference`] because
/// where a field belongs depends upon fields that may only be seen later. In
/// particular, whether the citation has a title of its own decides whether
/// `<source>` is the title of the reference or of a container that the volume,
/// issue and publisher then also belong to.
#[derive(Default)]
struct CitationFields {
    work_type: Option<CreativeWorkType>,
    doi: Option<String>,
    url: Option<String>,
    authors: Vec<Author>,
    editors: Vec<Person>,
    date: Option<Date>,
    title: Option<Vec<Inline>>,
    title_kind: CitationTitleKind,
    source: Option<Vec<Inline>>,
    volume_number: Option<IntegerOrString>,
    issue_number: Option<IntegerOrString>,
    page_start: Option<IntegerOrString>,
    page_end: Option<IntegerOrString>,
    pagination: Option<String>,
    version: Option<StringOrNumber>,
    publisher_name: Option<String>,
    publisher_location: Option<String>,
    identifiers: Option<Vec<PropertyValueOrString>>,
}

/// How an explicit citation title relates to its source
#[derive(Clone, Copy, Default)]
enum CitationTitleKind {
    /// The title is the primary cited work, such as an article
    #[default]
    Work,
    /// The title is a part of a book, such as a chapter
    BookPart,
}

/// Convert the text of a citation element into an integer or a string
///
/// Only uses an integer when doing so does not change the value, so that a
/// value such as the page "056708" keeps its leading zero.
fn integer_or_string(value: &str) -> IntegerOrString {
    match IntegerOrString::from(value) {
        IntegerOrString::Integer(integer) if integer.to_string() == value => {
            IntegerOrString::Integer(integer)
        }
        _ => IntegerOrString::String(value.to_string()),
    }
}

/// Map a citation's `publication-type` to the type of work referenced
fn work_type(publication_type: &str, title_kind: CitationTitleKind) -> Option<CreativeWorkType> {
    Some(match publication_type {
        "journal" | "preprint" | "eprint" => CreativeWorkType::Article,
        "book" => match title_kind {
            CitationTitleKind::Work => CreativeWorkType::Book,
            CitationTitleKind::BookPart => CreativeWorkType::Chapter,
        },
        "data" | "database" => CreativeWorkType::Dataset,
        "report" => CreativeWorkType::Report,
        "thesis" => CreativeWorkType::Thesis,
        "software" => CreativeWorkType::SoftwareApplication,
        "webpage" | "website" | "web" => CreativeWorkType::WebPage,
        _ => return None,
    })
}

/// Decode a `<citation>`, `<element-citation>` or `<mixed-citation>` element
///
/// `text_node` is the citation element to fall back to for raw citation text.
/// It is usually `node` itself but, when a `<citation-alternatives>` pairs a
/// structured citation with a `<mixed-citation>`, it is the latter because it
/// retains the punctuation that text parsing needs.
///
/// Fields decoded from the XML are authoritative: text parsing is only used to
/// fill in fields that no element supplied.
fn decode_citation(
    path: &str,
    id: &str,
    node: &Node,
    text_node: &Node,
    losses: &mut Losses,
) -> Reference {
    record_attrs_lost(path, node, ["publication-type"], losses);

    let mut fields = CitationFields::default();
    for child in node.children() {
        if !child.is_element() {
            // Text between elements of a mixed citation is punctuation and
            // labelling that is regenerated when encoding
            continue;
        }
        decode_citation_child(path, &child, &mut fields, losses);
    }

    if let Some(publication_type) = node.attribute("publication-type") {
        fields.work_type = work_type(publication_type, fields.title_kind);
        // A type that says the work is of no particular type is not a loss
        if fields.work_type.is_none()
            && !matches!(publication_type, "other" | "miscellaneous" | "unknown" | "")
        {
            losses.add(format!("{path}/@publication-type"));
        }
    }

    let mut reference = assemble_reference(id, fields);

    // Fields that no element supplied may still be recoverable from the text of
    // the citation, which for a <mixed-citation> is a complete rendering of the
    // reference. Nothing already decoded is overwritten by the weaker result.
    let text = text_node
        .descendants()
        .filter_map(|node| {
            if !node.is_text() {
                return None;
            }
            let text = node.text()?;
            Some(text.split_whitespace().join(" "))
        })
        .filter(|text| !text.is_empty())
        .join(" ");

    if !text.is_empty() {
        let parsed = text_to_reference(&text);
        if reference.authors.is_none() || reference.title.is_none() {
            fill_missing(&mut reference, parsed);
        } else {
            // Parsing citation prose is heuristic. When the structured
            // citation already has its defining fields, only accept values
            // whose textual forms have unambiguous parsers; otherwise prose
            // such as a URL can be mistaken for a container title or page.
            reference.doi = reference.doi.take().or(parsed.doi);
            reference.date = reference.date.take().or(parsed.date);
            reference.url = reference.url.take().or(parsed.url);
        }
    }

    reference
}

/// Decode one child element of a citation element into [`CitationFields`]
fn decode_citation_child(
    path: &str,
    child: &Node,
    fields: &mut CitationFields,
    losses: &mut Losses,
) {
    let tag = child.tag_name().name();
    let child_path = &extend_path(path, tag);

    let text = || child.text().map(str::trim).filter(|text| !text.is_empty());

    match tag {
        "name" | "string-name" => {
            let person = decode_person(child_path, child, losses);
            fields.authors.push(Author::Person(person));
        }
        "collab" => {
            // A collaboration is a group, rather than a person, that authored
            // the work
            record_attrs_lost(child_path, child, [], losses);
            if let Some(name) = text() {
                fields.authors.push(Author::Organization(Organization {
                    name: Some(name.to_string()),
                    ..Default::default()
                }));
            }
        }
        "etal" => {
            // There is no way to represent "and others" without fabricating
            // the people that it stands for
            record_attrs_lost(child_path, child, [], losses);
            losses.add(child_path.clone());
        }
        "person-group" => {
            record_attrs_lost(child_path, child, ["person-group-type"], losses);

            let is_authors = matches!(child.attribute("person-group-type"), Some("author") | None);
            for grandchild in child.children() {
                if !grandchild.is_element() {
                    continue;
                }
                match grandchild.tag_name().name() {
                    "name" | "string-name" => {
                        let grandchild_path =
                            &extend_path(child_path, grandchild.tag_name().name());
                        let person = decode_person(grandchild_path, &grandchild, losses);
                        if is_authors {
                            fields.authors.push(Author::Person(person))
                        } else {
                            fields.editors.push(person)
                        }
                    }
                    "collab" if is_authors => {
                        let grandchild_path = &extend_path(child_path, "collab");
                        record_attrs_lost(grandchild_path, &grandchild, [], losses);
                        if let Some(name) =
                            grandchild.text().map(str::trim).filter(|it| !it.is_empty())
                        {
                            fields.authors.push(Author::Organization(Organization {
                                name: Some(name.to_string()),
                                ..Default::default()
                            }));
                        }
                    }
                    "etal" => losses.add(extend_path(child_path, "etal")),
                    _ => record_node_lost(child_path, &grandchild, losses),
                }
            }
        }
        "article-title" | "trans-title" => {
            record_attrs_lost(child_path, child, [], losses);
            fields.title = Some(decode_inlines(child_path, child.children(), losses));
        }
        "chapter-title" | "part-title" | "data-title" => {
            record_attrs_lost(child_path, child, [], losses);
            fields.title = Some(decode_inlines(child_path, child.children(), losses));
            fields.title_kind = CitationTitleKind::BookPart;
        }
        "source" => {
            record_attrs_lost(child_path, child, [], losses);
            fields.source = Some(decode_inlines(child_path, child.children(), losses));
        }
        "year" => {
            record_attrs_lost(child_path, child, ["iso-8601-date"], losses);
            fields.date = child
                .attribute("iso-8601-date")
                .map(String::from)
                .or_else(|| text().map(String::from))
                .map(Date::new);
        }
        "volume" | "issue" | "fpage" | "lpage" | "page-range" | "edition" | "publisher-name"
        | "publisher-loc" => {
            record_attrs_lost(child_path, child, [], losses);
            let Some(value) = text() else { return };
            match tag {
                "volume" => fields.volume_number = Some(integer_or_string(value)),
                "issue" => fields.issue_number = Some(integer_or_string(value)),
                "fpage" => fields.page_start = Some(integer_or_string(value)),
                "lpage" => fields.page_end = Some(integer_or_string(value)),
                "page-range" => fields.pagination = Some(value.to_string()),
                "edition" => fields.version = Some(StringOrNumber::String(value.to_string())),
                "publisher-name" => fields.publisher_name = Some(value.to_string()),
                _ => fields.publisher_location = Some(value.to_string()),
            }
        }
        "elocation-id" => {
            // An electronic location identifier stands in for a page range but
            // is an identifier, not a page, so is kept as one
            record_attrs_lost(child_path, child, [], losses);
            if let Some(value) = text() {
                push_identifier(
                    &mut fields.identifiers,
                    Some("elocation-id".to_string()),
                    None,
                    value,
                );
            }
        }
        "pub-id" => {
            record_attrs_lost(child_path, child, ["pub-id-type", "specific-use"], losses);
            let Some(value) = text() else { return };
            match child.attribute("pub-id-type") {
                Some(id_type) if id_type.eq_ignore_ascii_case("doi") => {
                    fields.doi = Some(value.to_string())
                }
                property_id => push_identifier(
                    &mut fields.identifiers,
                    property_id.map(String::from),
                    child.attribute("specific-use").map(String::from),
                    value,
                ),
            }
        }
        "ext-link" | "uri" => {
            record_attrs_lost(child_path, child, ["ext-link-type", "href"], losses);
            let url = child
                .attribute((XLINK, "href"))
                .or_else(|| child.attribute("href"))
                .map(str::trim)
                .filter(|url| !url.is_empty())
                .map(String::from)
                .or_else(|| text().map(String::from));
            if fields.url.is_none() {
                fields.url = url;
            }
        }
        "comment" => {
            // Comments are often just a label for a following identifier, such
            // as "doi:", which is regenerated when encoding
            record_attrs_lost(child_path, child, [], losses);
            let content = child
                .descendants()
                .filter_map(|node| node.text())
                .join(" ")
                .trim()
                .to_string();
            if !content.is_empty() && !content.ends_with(':') {
                losses.add(child_path.clone());
            }
        }
        _ => record_node_lost(path, child, losses),
    }
}

/// Assemble the fields decoded from a citation into a [`Reference`]
///
/// Metadata of the container that the referenced work is part of, such as the
/// volume and issue of the journal that an article appeared in, is placed on
/// `isPartOf`. When the citation is of a whole work, so that `<source>` is the
/// title of the reference itself, that metadata stays on the reference.
fn assemble_reference(id: &str, fields: CitationFields) -> Reference {
    let CitationFields {
        work_type,
        doi,
        url,
        authors,
        editors,
        date,
        mut title,
        title_kind,
        source,
        volume_number,
        issue_number,
        page_start,
        page_end,
        pagination,
        version,
        publisher_name,
        publisher_location,
        identifiers,
    } = fields;

    let authors = (!authors.is_empty()).then_some(authors);
    let mut editors = (!editors.is_empty()).then_some(editors);

    // A citation can name where a work was published without naming its
    // publisher, so the location alone is enough to record one
    let publisher = (publisher_name.is_some() || publisher_location.is_some()).then(|| {
        PersonOrOrganization::Organization(Organization {
            name: publisher_name,
            options: Box::new(OrganizationOptions {
                address: publisher_location.map(PostalAddressOrString::String),
                ..Default::default()
            }),
            ..Default::default()
        })
    });

    let mut reference = Reference {
        id: Some(id.into()),
        work_type,
        doi,
        authors,
        date,
        url,
        options: Box::new(ReferenceOptions {
            page_start,
            page_end,
            pagination,
            version,
            identifiers,
            ..Default::default()
        }),
        ..Default::default()
    };

    match source {
        // A whole journal, book or other work is being referenced
        Some(source) if title.is_none() => {
            reference.title = Some(source);
            reference.options.editors = editors;
            reference.options.publisher = publisher;
            reference.options.volume_number = volume_number;
            reference.options.issue_number = issue_number;
        }
        // A part of a container work is being referenced
        Some(source) => {
            reference.title = title;
            reference.is_part_of = Some(Box::new(Reference {
                work_type: Some(match title_kind {
                    CitationTitleKind::Work => CreativeWorkType::Periodical,
                    CitationTitleKind::BookPart => CreativeWorkType::Book,
                }),
                title: Some(source),
                options: Box::new(ReferenceOptions {
                    // Editors of a book edited the container, not the chapter,
                    // so are taken from the reference itself
                    editors: matches!(title_kind, CitationTitleKind::BookPart)
                        .then(|| editors.take())
                        .flatten(),
                    publisher,
                    volume_number,
                    issue_number,
                    ..Default::default()
                }),
                ..Default::default()
            }));
            reference.options.editors = editors;
        }
        None => {
            reference.title = title.take();
            reference.options.editors = editors;
            reference.options.publisher = publisher;
            reference.options.volume_number = volume_number;
            reference.options.issue_number = issue_number;
        }
    }

    reference
}

/// Fill fields that a citation's elements did not supply from a reference
/// parsed from the citation's text
///
/// Only empty fields are filled: a field decoded from an element is always more
/// reliable than one recovered by parsing text.
fn fill_missing(reference: &mut Reference, parsed: Reference) {
    let Reference {
        work_type,
        doi,
        authors,
        date,
        title,
        is_part_of,
        url,
        options,
        ..
    } = parsed;

    reference.work_type = reference.work_type.or(work_type);
    reference.doi = reference.doi.take().or(doi);
    reference.authors = reference.authors.take().or(authors);
    reference.date = reference.date.take().or(date);
    reference.title = reference.title.take().or(title);
    reference.is_part_of = reference.is_part_of.take().or(is_part_of);
    reference.url = reference.url.take().or(url);

    let ReferenceOptions {
        editors,
        publisher,
        volume_number,
        issue_number,
        page_start,
        page_end,
        pagination,
        version,
        text,
        ..
    } = *options;

    let reference_options = &mut reference.options;
    reference_options.editors = reference_options.editors.take().or(editors);
    reference_options.publisher = reference_options.publisher.take().or(publisher);
    reference_options.volume_number = reference_options.volume_number.take().or(volume_number);
    reference_options.issue_number = reference_options.issue_number.take().or(issue_number);
    reference_options.page_start = reference_options.page_start.take().or(page_start);
    reference_options.page_end = reference_options.page_end.take().or(page_end);
    reference_options.pagination = reference_options.pagination.take().or(pagination);
    reference_options.version = reference_options.version.take().or(version);

    // The raw text is only worth keeping when the reference can not be
    // rendered from its fields, which without a title it can not. When it is
    // kept, it is encoded as an alternative to, not as well as, those fields.
    if reference.title.is_none() {
        reference_options.text = reference_options.text.take().or(text);
    }
}

/// Decode a `<name>` or `<string-name>`
fn decode_person(path: &str, node: &Node, losses: &mut Losses) -> Person {
    record_attrs_lost(path, node, [], losses);

    let mut family_names = Vec::new();
    let mut given_names = Vec::new();

    for child in node.children() {
        let tag = child.tag_name().name();
        if tag == "surname" {
            if let Some(value) = child.text() {
                family_names.push(value.to_string());
            }
        } else if tag == "given-names" {
            if let Some(value) = child.text() {
                given_names.append(&mut split_given_names(value));
            }
        } else {
            record_node_lost(path, &child, losses);
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
