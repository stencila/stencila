use roxmltree::Node;

use stencila_codec::stencila_schema::{
    Author, ContactPoint, CreativeWorkType, Date, Inline, IntegerOrString, Organization,
    OrganizationOptions, Person, PersonOrOrganization, PostalAddressOrString, Primitive,
    PropertyValue, PropertyValueOptions, PropertyValueOrString, Reference, ReferenceOptions,
};

use stencila_codec_utils::{split_given_names, strip_doi_prefix, strip_ror_prefix};

use super::{
    Context, child, collapse_whitespace, non_empty_text, path,
    text::{decode_inlines, normalize},
    text_of, typed_path, xml_id,
};

/// Decode a `<listBibl>` to a list of [`Reference`]s
pub(super) fn decode_list_bibl(node: &Node, context: &mut Context) -> Vec<Reference> {
    context.lost_attrs(node, &[]);

    node.children()
        .filter_map(|child| match child.tag_name().name() {
            "biblStruct" => Some(decode_bibl_struct(&child, context)),
            _ => {
                context.lost(&child);
                None
            }
        })
        .collect()
}

/// The fields of a `<biblStruct>`
///
/// Collected before being assembled into a [`Reference`] because where a
/// field belongs depends on fields that may only be seen later, as it does
/// when decoding a JATS citation.
#[derive(Default)]
struct BiblFields {
    /// The fields of the cited work
    cited: WorkFields,
    /// The fields of the work that the cited work is part of, if it is
    container: WorkFields,
    /// Whether there is an `<analytic>`, even if it has no title
    has_analytic: bool,
    editors: Vec<Person>,
    date: Option<Date>,
    title: Option<Vec<Inline>>,
    source: Option<Vec<Inline>>,
    source_level: SourceLevel,
    is_meeting: bool,
    scope: Scope,
    publisher_name: Option<String>,
    publisher_location: Option<String>,
    text: Option<String>,
}

/// The fields of a work that the `<analytic>` and the `<monogr>` of a
/// `<biblStruct>` can each have
#[derive(Default)]
struct WorkFields {
    authors: Vec<Author>,
    doi: Option<String>,
    url: Option<String>,
    identifiers: Option<Vec<PropertyValueOrString>>,
}

/// Which of the works of a `<biblStruct>` a field is of
#[derive(Clone, Copy)]
enum Work {
    /// The cited work
    Cited,
    /// The work that the cited work is part of
    Container,
}

impl BiblFields {
    fn work(&mut self, work: Work) -> &mut WorkFields {
        match work {
            Work::Cited => &mut self.cited,
            Work::Container => &mut self.container,
        }
    }
}

/// The bibliographic level of the title of the `<monogr>` of a `<biblStruct>`
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SourceLevel {
    /// A journal (`level="j"`)
    Journal,
    /// A monograph, such as a book or the proceedings of a meeting (`level="m"`)
    #[default]
    Monograph,
}

/// Decode a `<biblStruct>` to a [`Reference`]
///
/// # Bibliographic reference contract
///
/// A `<biblStruct>` separates the cited work (`<analytic>`) from the work it
/// is part of (`<monogr>`), which is close to the nesting of a [`Reference`].
/// The mapping is the same as for a JATS citation, so that both decode to the
/// same nodes:
///
/// | TEI                                    | Schema                                 |
/// |----------------------------------------|----------------------------------------|
/// | `analytic/title`                       | `title`                                |
/// | `monogr/title`, with an analytic title | `isPartOf.title`                       |
/// | `monogr/title`, with no analytic title | `title`                                |
/// | `monogr/title[@level='j']`             | `workType` article, part of a periodical |
/// | `monogr/title[@level='m']`             | `workType` chapter, part of a book (unless of a meeting) |
/// | `analytic/author`                      | `authors`                              |
/// | `monogr/author`, `idno`, `ptr`         | container's, with an analytic title, else the reference's |
/// | `editor`                               | `editors`, of the container for a book |
/// | `biblScope[@unit='volume' or 'issue']` | container's, else the reference's      |
/// | `biblScope[@unit='page']`              | `pageStart`, `pageEnd`                 |
/// | `imprint/date`, preferring `type='published'` | `date`                           |
/// | `imprint/publisher`, `pubPlace`        | container's `publisher`, else the reference's |
/// | `idno[@type='DOI']`                    | `doi`                                  |
/// | other `idno`s                          | `identifiers`, keyed by their type     |
/// | `ptr/@target`, `ref[@type='url']` (its target, else its text) | `url`, the first |
/// | `note[@type='raw_reference']`          | `text`, only if there is no title      |
///
/// A raw reference that is not kept, because the reference has a title, is
/// recorded as lost (counted rather than reported for each reference).
pub(super) fn decode_bibl_struct(node: &Node, context: &mut Context) -> Reference {
    context.lost_attrs(node, &["id", "coords", "status", "type"]);

    // The `<monogr>` is of a container only when both it and the `<analytic>`
    // have a title, as in `assemble_reference`
    let has_title = |tag: &str| {
        child(node, tag).is_some_and(|work| {
            work.children()
                .any(|child| child.has_tag_name("title") && non_empty_text(&child).is_some())
        })
    };
    let monogr_work = if has_title("analytic") && has_title("monogr") {
        Work::Container
    } else {
        Work::Cited
    };

    let mut fields = BiblFields::default();
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "analytic" => decode_analytic(&child, &mut fields, context),
            "monogr" => decode_monogr(&child, &mut fields, monogr_work, context),
            "idno" | "ptr" | "ref" => decode_work_child(&child, &mut fields.cited, context),
            "note" if child.attribute("type") == Some("raw_reference") => {
                if fields.text.is_some() {
                    context.lost(&child);
                } else {
                    fields.text = non_empty_text(&child);
                }
            }
            "note" => context.lost_label(typed_path(&child, "type")),
            _ => context.lost(&child),
        }
    }

    let has_text = fields.text.is_some();
    let reference = assemble_reference(xml_id(node).map(String::from), fields);
    if has_text && reference.options.text.is_none() {
        context.lost_label(format!(
            "{}/note[@type='raw_reference'] (redundant with fields)",
            path(node)
        ));
    }

    context.located(node, reference)
}

/// Decode a child of a `<biblStruct>`, `<analytic>` or `<monogr>` that is an
/// identifier or URL of a work, recording any other child as lost
fn decode_work_child(node: &Node, work: &mut WorkFields, context: &mut Context) {
    match node.tag_name().name() {
        "idno" => decode_idno(node, &mut work.doi, &mut work.identifiers, context),
        "ptr" => decode_url(node, &mut work.url, context),
        "ref" if node.attribute("type") == Some("url") => decode_url(node, &mut work.url, context),
        _ => context.lost(node),
    }
}

/// Decode the `<analytic>` of a `<biblStruct>`: the cited work when it is a
/// part of another work, such as an article in a journal
fn decode_analytic(node: &Node, fields: &mut BiblFields, context: &mut Context) {
    context.lost_attrs(node, &[]);
    fields.has_analytic = true;

    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "title" => decode_title(&child, &mut fields.title, context),
            "author" => fields.cited.authors.extend(decode_author(&child, context)),
            "editor" => fields.editors.extend(decode_editor(&child, context)),
            _ => decode_work_child(&child, &mut fields.cited, context),
        }
    }
}

/// Decode the `<monogr>` of a `<biblStruct>`: the work that the cited work is
/// part of, or the cited work itself when there is no `<analytic>`
///
/// The authors, identifiers and URLs of the `<monogr>` are of the work given
/// by `work`, because they are of the container only when there is one.
fn decode_monogr(node: &Node, fields: &mut BiblFields, work: Work, context: &mut Context) {
    context.lost_attrs(node, &[]);

    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "title" => {
                // The first title is the main title; others, such as an
                // abbreviated journal title, add nothing that can be kept
                if fields.source.is_some() {
                    context.lost(&child);
                    continue;
                }
                fields.source_level = match child.attribute("level") {
                    Some("j") => SourceLevel::Journal,
                    _ => SourceLevel::Monograph,
                };
                decode_title(&child, &mut fields.source, context);
            }
            "author" => fields
                .work(work)
                .authors
                .extend(decode_author(&child, context)),
            "editor" => fields.editors.extend(decode_editor(&child, context)),
            "idno" | "ptr" | "ref" => decode_work_child(&child, fields.work(work), context),
            "meeting" => {
                // The name and place of a meeting has nowhere to go other than
                // the title of the proceedings, which GROBID gives separately
                fields.is_meeting = true;
                context.lost(&child);
            }
            "imprint" => decode_imprint(&child, fields, context),
            _ => context.lost(&child),
        }
    }
}

/// Decode the `<imprint>` of a `<monogr>`
///
/// Prefer a publication date to other dates, retaining the last non-empty
/// date when there is more than one with the same priority, and recording an
/// earlier, different one as lost. Only the first publisher and place of
/// publication are kept.
fn decode_imprint(node: &Node, fields: &mut BiblFields, context: &mut Context) {
    context.lost_attrs(node, &[]);

    let mut published_date = None;
    let mut fallback_date = None;
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "biblScope" => decode_bibl_scope(&child, &mut fields.scope, context),
            "date" => {
                if let Some(date) = decode_date(&child, context) {
                    let target = if child.attribute("type") == Some("published") {
                        &mut published_date
                    } else {
                        &mut fallback_date
                    };
                    if let Some((previous, element)) = target.replace((date.clone(), child))
                        && previous != date
                    {
                        context.lost(&element);
                    }
                }
            }
            "publisher" => {
                context.lost_attrs(&child, &[]);
                keep_first(&mut fields.publisher_name, &child, context);
            }
            "pubPlace" => {
                context.lost_attrs(&child, &[]);
                keep_first(&mut fields.publisher_location, &child, context);
            }
            _ => context.lost(&child),
        }
    }

    if let Some((date, ..)) = published_date.or(fallback_date) {
        fields.date = Some(Date::new(date));
    }
}

/// Keep the text of an element in a field, unless it is empty or the field
/// already has text, in which case different text is recorded as lost
fn keep_first(field: &mut Option<String>, node: &Node, context: &mut Context) {
    let Some(text) = non_empty_text(node) else {
        return;
    };
    match field {
        None => *field = Some(text),
        Some(kept) if *kept != text => context.lost(node),
        Some(..) => {}
    }
}

/// The volume, issue and pages of a work, from its `<biblScope>`s
#[derive(Default)]
pub(super) struct Scope {
    pub volume_number: Option<IntegerOrString>,
    pub issue_number: Option<IntegerOrString>,
    pub page_start: Option<IntegerOrString>,
    pub page_end: Option<IntegerOrString>,
}

/// Decode a `<biblScope>` onto the volume, issue or pages of a work
///
/// The pages are in `from` and `to` or, if neither is present, in the text,
/// which may be a range (e.g. "12-20"). Only the parts of the scope that a
/// `<biblScope>` gives are set, so that, for example, one with only a first
/// page does not remove the last page given by an earlier one.
pub(super) fn decode_bibl_scope(node: &Node, scope: &mut Scope, context: &mut Context) {
    let unit = node.attribute("unit");
    let from = node.attribute("from").map(str::trim);
    let to = node.attribute("to").map(str::trim);
    let text = non_empty_text(node);

    let (start, end) = match (from, to) {
        (None, None) => match text.as_deref().and_then(|text| text.split_once(['-', '–'])) {
            Some((start, end)) => (Some(start.trim()), Some(end.trim())),
            None => (text.as_deref(), None),
        },
        from_to => from_to,
    };

    let set = |field: &mut Option<IntegerOrString>, value: Option<&str>| {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            *field = Some(integer_or_string(value));
        }
    };

    // A volume or issue is in the text or, if there is none, in `from` and
    // `to` (e.g. a combined issue "3-4")
    let number = text.clone().or_else(|| match (from, to) {
        (Some(from), Some(to)) if from != to => Some([from, "-", to].concat()),
        (Some(from), _) => Some(from.to_string()),
        (None, to) => to.map(String::from),
    });

    match unit {
        Some("volume") => set(&mut scope.volume_number, number.as_deref()),
        Some("issue") => set(&mut scope.issue_number, number.as_deref()),
        Some("page") => {
            set(&mut scope.page_start, start);
            set(&mut scope.page_end, end);
        }
        _ => {
            context.lost_label(typed_path(node, "unit"));
            return;
        }
    }

    context.lost_attrs(node, &["unit", "from", "to"]);
}

/// Decode a `<title>` to inlines, if it has any text
///
/// A title that is not the main title (e.g. `type="sub"`) when there already is
/// one is recorded as lost. Whether a title is kept is decided before its
/// content is decoded, so that nothing within it, such as a footnote, is
/// decoded only to be dropped.
fn decode_title(node: &Node, title: &mut Option<Vec<Inline>>, context: &mut Context) {
    context.lost_attrs(node, &["level", "type"]);

    if non_empty_text(node).is_none() {
        return;
    }

    if title.is_some() {
        context.lost(node);
        return;
    }

    *title = Some(normalize(decode_inlines(node.children(), context)));
}

/// Decode an `<idno>` as a DOI, or as another identifier keyed by its type
pub(super) fn decode_idno(
    node: &Node,
    doi: &mut Option<String>,
    identifiers: &mut Option<Vec<PropertyValueOrString>>,
    context: &mut Context,
) {
    context.lost_attrs(node, &["type"]);

    let Some(value) = non_empty_text(node) else {
        return;
    };

    let id_type = node.attribute("type");
    if id_type.is_some_and(|id_type| id_type.eq_ignore_ascii_case("doi")) && doi.is_none() {
        *doi = Some(strip_doi_prefix(&value));
        return;
    }

    push_identifier(identifiers, id_type.map(String::from), &value);
}

/// Decode an `<idno>` as an identifier keyed by its type, including a DOI,
/// for things that have no `doi` property
pub(super) fn decode_idno_as_identifier(
    node: &Node,
    identifiers: &mut Option<Vec<PropertyValueOrString>>,
    context: &mut Context,
) {
    let mut doi = None;
    decode_idno(node, &mut doi, identifiers, context);
    if let Some(doi) = doi {
        push_identifier(identifiers, Some("DOI".into()), &doi);
    }
}

/// Decode a `<ptr>` or `<ref type="url">` as a URL, keeping the first
///
/// Use the target when present, otherwise the text of a `<ref>`. A later,
/// different URL, and the text of a `<ref>` that differs from its target,
/// have nowhere to go, so are recorded as lost.
///
/// A pointer to a part of the TEI itself (e.g. GROBID's `#string-range(...)`
/// pointers from a reference to where a patent cites it) is not a URL of the
/// work, so is recorded as lost.
fn decode_url(node: &Node, url: &mut Option<String>, context: &mut Context) {
    let is_ref = node.has_tag_name("ref");
    let target = node
        .attribute("target")
        .map(str::trim)
        .filter(|target| !target.is_empty());
    let text = if is_ref { non_empty_text(node) } else { None };
    let value = target.map(String::from).or_else(|| text.clone());

    if value.as_deref().is_some_and(|value| value.starts_with('#')) {
        context.lost(node);
        return;
    }

    // Only the first URL is kept, so a different one is lost
    if let (Some(kept), Some(value)) = (url.as_deref(), value.as_deref())
        && kept != value
    {
        context.lost(node);
        return;
    }

    context.lost_attrs(
        node,
        if is_ref {
            &["target", "type"]
        } else {
            &["target"]
        },
    );

    // The text of a `<ref>` with a target has nowhere to go, unless it is the target
    if let (Some(target), Some(text)) = (target, text)
        && target != text
    {
        context.lost_label([&path(node), "/text()"].concat());
    }

    if url.is_none() {
        *url = value;
    }
}

/// Decode a `<date>` to an ISO 8601 date string
///
/// GROBID gives the date in `when` (e.g. "2024-07-08", "1997-10") and the
/// date as written as the text, which is used only if there is no `when`.
pub(super) fn decode_date(node: &Node, context: &mut Context) -> Option<String> {
    context.lost_attrs(node, &["type", "when"]);

    node.attribute("when")
        .map(str::trim)
        .filter(|when| !when.is_empty())
        .map(String::from)
        .or_else(|| non_empty_text(node))
}

/// Assemble the fields of a `<biblStruct>` into a [`Reference`]
///
/// As when decoding a JATS citation, metadata of the container that the cited
/// work is part of, such as the volume and issue of the journal that an
/// article appeared in, is placed on `isPartOf`. When the work is not part of
/// another (there is no `<analytic>` title) that metadata stays on the
/// reference. GROBID does not say what type a monograph is, so a reference
/// with only a monograph title has no `workType`. A reference with an
/// `<analytic>` but only a journal title is of an article, whose title is not
/// known, in that journal.
fn assemble_reference(id: Option<String>, fields: BiblFields) -> Reference {
    let BiblFields {
        cited,
        container,
        has_analytic,
        editors,
        date,
        title,
        source,
        source_level,
        is_meeting,
        scope:
            Scope {
                volume_number,
                issue_number,
                page_start,
                page_end,
            },
        publisher_name,
        publisher_location,
        text,
    } = fields;

    let editors = (!editors.is_empty()).then_some(editors);

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
        id,
        date,
        ..cited.into_reference()
    };
    reference.options.page_start = page_start;
    reference.options.page_end = page_end;

    match (title, source) {
        // A part of a container work is being referenced
        (Some(title), Some(source)) => {
            let is_book = source_level == SourceLevel::Monograph;
            reference.work_type = match source_level {
                SourceLevel::Journal => Some(CreativeWorkType::Article),
                SourceLevel::Monograph if !is_meeting => Some(CreativeWorkType::Chapter),
                SourceLevel::Monograph => None,
            };
            reference.title = Some(title);

            // Editors of a book edited the container, not the chapter
            let (container_editors, editors) = if is_book {
                (editors, None)
            } else {
                (None, editors)
            };
            reference.options.editors = editors;

            let mut part_of = Reference {
                work_type: Some(match source_level {
                    SourceLevel::Journal => CreativeWorkType::Periodical,
                    SourceLevel::Monograph => CreativeWorkType::Book,
                }),
                title: Some(source),
                ..container.into_reference()
            };
            part_of.options.editors = container_editors;
            part_of.options.publisher = publisher;
            part_of.options.volume_number = volume_number;
            part_of.options.issue_number = issue_number;
            reference.is_part_of = Some(Box::new(part_of));
        }
        // An article, without a title, in a journal is being referenced
        (None, Some(source)) if has_analytic && source_level == SourceLevel::Journal => {
            reference.work_type = Some(CreativeWorkType::Article);
            reference.options.editors = editors;
            reference.is_part_of = Some(Box::new(Reference {
                work_type: Some(CreativeWorkType::Periodical),
                title: Some(source),
                options: Box::new(ReferenceOptions {
                    publisher,
                    volume_number,
                    issue_number,
                    ..Default::default()
                }),
                ..Default::default()
            }));
        }
        // A whole work is being referenced
        (title, source) => {
            reference.title = title.or(source);
            reference.options.editors = editors;
            reference.options.publisher = publisher;
            reference.options.volume_number = volume_number;
            reference.options.issue_number = issue_number;
        }
    }

    // The raw text is only worth keeping when the reference can not be
    // rendered from its fields, which without a title it can not
    if reference.title.is_none() {
        reference.options.text = text;
    }

    reference
}

impl WorkFields {
    /// Create a reference to a work with these fields
    fn into_reference(self) -> Reference {
        Reference {
            authors: (!self.authors.is_empty()).then_some(self.authors),
            doi: self.doi,
            url: self.url,
            options: Box::new(ReferenceOptions {
                identifiers: self.identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

/// Decode an `<author>` to an [`Author`]
///
/// A person's names are in `<persName>`, along with their `<email>`, ORCID
/// (`<idno type="ORCID">`) and `<affiliation>`s. Only the first `<persName>`
/// with a name is decoded: others, such as a transliteration, are recorded as
/// lost. An author with no personal name but an `<orgName>` is an
/// organization, such as a consortium.
/// GROBID's dummy authors are omitted, with each orphan affiliation reported
/// as a named loss because it has no person to attach to.
pub(super) fn decode_author(node: &Node, context: &mut Context) -> Option<Author> {
    if node
        .children()
        .any(|child| child.has_tag_name("note") && child.attribute("type") == Some("dummy_author"))
    {
        context.lost_attrs(node, &[]);
        for child in node.children() {
            match child.tag_name().name() {
                "note" if child.attribute("type") == Some("dummy_author") => {
                    context.lost_attrs(&child, &["type"]);
                }
                "affiliation" => {
                    // Decoded, though never kept, so that it is named as it
                    // would be for a person, and its id and coordinates are
                    // recorded as lost
                    let label = match decode_affiliation(&child, context).name {
                        Some(name) => format!("{} (orphan affiliation: {name})", path(&child)),
                        None => format!("{} (orphan affiliation)", path(&child)),
                    };
                    context.lost_label(label);
                }
                _ => context.lost(&child),
            }
        }
        return None;
    }

    // The corresponding author is marked with `role="corresp"`, which says no
    // more than their email address already does
    let decoded: &[&str] = if node.attribute("role") == Some("corresp") {
        &["id", "coords", "role"]
    } else {
        &["id", "coords"]
    };
    context.lost_attrs(node, decoded);

    let mut person = Person {
        id: xml_id(node).map(String::from),
        ..Default::default()
    };
    let mut pers_names = Vec::new();
    let mut org_names = Vec::new();
    for child in node.children() {
        if !child.is_element() {
            context.lost(&child);
            continue;
        }
        match child.tag_name().name() {
            "persName" => {
                if !has_name(&person) {
                    decode_pers_name(&child, &mut person, context);
                    pers_names.push(child);
                } else if non_empty_text(&child).is_some() {
                    context.lost(&child);
                } else {
                    context.lost_attrs(&child, &[]);
                }
            }
            "email" => {
                context.lost_attrs(&child, &[]);
                if let Some(email) = non_empty_text(&child) {
                    person.options.emails.get_or_insert_default().push(email);
                }
            }
            "idno"
                if child
                    .attribute("type")
                    .is_some_and(|id_type| id_type.eq_ignore_ascii_case("orcid")) =>
            {
                context.lost_attrs(&child, &["type"]);
                person.orcid = non_empty_text(&child).map(|orcid| {
                    orcid
                        .trim_start_matches("https://orcid.org/")
                        .trim_start_matches("http://orcid.org/")
                        .to_string()
                });
            }
            "idno" => decode_idno_as_identifier(&child, &mut person.options.identifiers, context),
            "affiliation" => person
                .affiliations
                .get_or_insert_default()
                .push(decode_affiliation(&child, context)),
            "orgName" => {
                context.lost_attrs(&child, &["id", "coords", "type"]);
                org_names.push(child);
            }
            _ => context.lost(&child),
        }
    }

    // An `orgName` is the name of the author only when it has no personal
    // name, so its id and coordinates are only kept then
    if !has_name(&person) {
        // Any `persName`s are empty, so nothing has their ids or coordinates
        for pers_name in &pers_names {
            context.lost_attrs_named(pers_name, &["id", "coords"]);
        }

        // The first `orgName` with text is the name, and others are lost
        let mut org_names = org_names.into_iter();
        let mut named = None;
        for org_name in org_names.by_ref() {
            if let Some(name) = non_empty_text(&org_name) {
                named = Some((org_name, name));
                break;
            }
            context.lost_attrs_named(&org_name, &["id", "coords"]);
        }
        for org_name in org_names {
            context.lost(&org_name);
        }

        let Some((org_name, name)) = named else {
            // An author with no name of either kind is not decoded
            context.lost(node);
            return None;
        };

        if person.id.is_some() {
            context.lost_attrs_named(&org_name, &["id"]);
        }

        let organization = person_to_organization(person, &org_name, name, node, context);
        let organization = context.located(node, organization);
        context.locate(&org_name, &organization, Some("orgName"));

        return Some(Author::Organization(organization));
    }

    if !org_names.is_empty() {
        context.lost_label([&path(node), "/orgName"].concat());
    }

    // The author's own id takes precedence over that of the first `persName`
    // with one
    for pers_name in &pers_names {
        if xml_id(pers_name).is_none() {
            continue;
        }
        if person.id.is_none() {
            person.id = xml_id(pers_name).map(String::from);
        } else {
            context.lost_attrs_named(pers_name, &["id"]);
        }
    }

    let person = context.located(node, person);
    for pers_name in pers_names {
        context.locate(&pers_name, &person, Some("persName"));
    }

    Some(Author::Person(person))
}

/// Whether a person has a name
fn has_name(person: &Person) -> bool {
    person.family_names.is_some() || person.given_names.is_some() || person.options.name.is_some()
}

/// Convert a person, decoded from an `<author>` with no personal name, into
/// an organization, named by one of its `<orgName>`s
///
/// Only what an organization can have is transferred: its emails become a
/// contact point. A person's affiliations and ORCID have nowhere to go, so
/// are recorded as lost.
fn person_to_organization(
    person: Person,
    org_name: &Node,
    name: String,
    author: &Node,
    context: &mut Context,
) -> Organization {
    let Person {
        id,
        affiliations,
        orcid,
        options,
        ..
    } = person;

    for _ in affiliations.iter().flatten() {
        context.lost_label([&path(author), "/affiliation"].concat());
    }
    if orcid.is_some() {
        context.lost_label([&path(author), "/idno[@type='ORCID']"].concat());
    }

    let contact_points = options.emails.map(|emails| {
        vec![ContactPoint {
            emails: Some(emails),
            ..Default::default()
        }]
    });

    Organization {
        id: id.or_else(|| xml_id(org_name).map(String::from)),
        name: Some(name),
        options: Box::new(OrganizationOptions {
            identifiers: options.identifiers,
            contact_points,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Decode an `<editor>` to a [`Person`]
///
/// Keep an unstructured editor's text as its name, as for an unstructured
/// `<persName>`. It may contain several names, whose boundaries are ambiguous.
fn decode_editor(node: &Node, context: &mut Context) -> Option<Person> {
    if !node.children().any(|child| child.is_element())
        && let Some(name) = non_empty_text(node)
    {
        context.lost_attrs(node, &["id", "coords"]);
        let mut person = Person {
            id: xml_id(node).map(String::from),
            ..Default::default()
        };
        person.options.name = Some(name);
        return Some(context.located(node, person));
    }

    match decode_author(node, context)? {
        Author::Person(person) => Some(person),
        _ => {
            // `editors` is a list of people
            context.lost(node);
            None
        }
    }
}

/// Decode a `<persName>` onto a [`Person`]
fn decode_pers_name(node: &Node, person: &mut Person, context: &mut Context) {
    // Its id is given to the person, and its coordinates are those of a part
    // of the person, by `decode_author` once it knows the person has a name
    context.lost_attrs(node, &["id", "coords"]);

    let mut given_names = Vec::new();
    let mut family_names = Vec::new();
    for child in node.children().filter(Node::is_element) {
        let tag = child.tag_name().name();
        if !matches!(tag, "forename" | "surname" | "roleName" | "genName") {
            context.lost(&child);
            continue;
        }
        let Some(text) = non_empty_text(&child) else {
            continue;
        };
        match tag {
            "forename" => {
                context.lost_attrs(&child, &["type"]);
                given_names.append(&mut split_given_names(&text));
            }
            "surname" => {
                context.lost_attrs(&child, &[]);
                family_names.push(text);
            }
            "roleName" => {
                context.lost_attrs(&child, &[]);
                person.options.honorific_prefix = Some(text);
            }
            "genName" => {
                context.lost_attrs(&child, &[]);
                person.options.honorific_suffix = Some(text);
            }
            _ => context.lost(&child),
        }
    }

    if given_names.is_empty() && family_names.is_empty() {
        // A name written as running text has no parts to separate
        person.options.name = non_empty_text(node);
    } else {
        // Text between the parts of a structured name (e.g. the ", Jr." of
        // `<surname>Smith</surname>, Jr.`) is not in any of them
        for child in node.children().filter(Node::is_text) {
            context.lost(&child);
        }
    }

    person.given_names = (!given_names.is_empty()).then_some(given_names);
    person.family_names = (!family_names.is_empty()).then_some(family_names);
}

/// Decode an `<affiliation>` to an [`Organization`]
///
/// As when decoding a JATS `<aff>`, the names of the organization and its
/// divisions (`<orgName>`s, in the order given) are joined to form its name,
/// and the parts of its `<address>` are joined to form its address. Text
/// directly within the affiliation is its name if it has no `<orgName>`, else
/// its address if it has no `<address>`, else it is recorded as lost.
pub(super) fn decode_affiliation(node: &Node, context: &mut Context) -> Organization {
    // The `key` (e.g. "aff0") only identifies the affiliation within the TEI
    context.lost_attrs(node, &["id", "coords", "key"]);

    const TRIM_CHARS: &[char] = &[',', '.', ';', ' '];

    let trim_part = |text: &str| {
        collapse_whitespace(text)
            .trim_matches(TRIM_CHARS)
            .to_string()
    };

    let mut name = Vec::new();
    let mut address = Vec::new();
    let mut ror = None;
    let mut identifiers = None;
    let mut parts = Vec::new();
    let mut raw = None;
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "orgName" => {
                context.lost_attrs(&child, &["coords", "type", "key"]);
                parts.push((child, "orgName"));
                if let Some(text) = non_empty_text(&child) {
                    name.push(text.trim_matches(TRIM_CHARS).to_string());
                }
            }
            "address" => {
                context.lost_attrs(&child, &["coords"]);
                parts.push((child, "address"));
                // Addresses can mix text with structured parts, or consist
                // entirely of text. Text split only by comments, or processing
                // instructions, is one part.
                let mut text = String::new();
                for part in child.children() {
                    if part.is_text() {
                        text.push_str(part.text().unwrap_or_default());
                    } else if part.is_element() {
                        address.push(trim_part(&std::mem::take(&mut text)));
                        // The `key` of a <country> is its ISO 3166 code, which
                        // is not kept because the address is a string
                        context.lost_attrs(&part, &["key"]);
                        address.push(trim_part(&text_of(&part)));
                    }
                }
                address.push(trim_part(&text));
            }
            "idno" => {
                context.lost_attrs(&child, &["type"]);
                if let Some(value) = non_empty_text(&child) {
                    match child.attribute("type") {
                        Some(id_type) if id_type.eq_ignore_ascii_case("ror") => {
                            ror = Some(strip_ror_prefix(&value))
                        }
                        id_type => {
                            push_identifier(&mut identifiers, id_type.map(String::from), &value)
                        }
                    }
                }
            }
            // The affiliation as written, which GROBID includes if asked to
            "note" if child.attribute("type") == Some("raw_affiliation") => {
                raw = non_empty_text(&child);
            }
            _ => context.lost(&child),
        }
    }
    name.retain(|name| !name.is_empty());
    address.retain(|part| !part.is_empty());

    let text = node
        .children()
        .filter(Node::is_text)
        .filter_map(|child| child.text())
        .collect::<String>();
    let text = trim_part(&text);
    if !text.is_empty() {
        if name.is_empty() && raw.is_none() {
            // An affiliation written as plain text
            raw = Some(text);
        } else if address.is_empty() {
            address.push(text);
        } else {
            context.lost_label([&path(node), "/text()"].concat());
        }
    }

    let name = if name.is_empty() {
        raw.or_else(|| (!address.is_empty()).then(|| std::mem::take(&mut address).join(", ")))
    } else {
        Some(name.join(", "))
    };

    let organization = context.located(
        node,
        Organization {
            id: xml_id(node).map(String::from),
            name,
            ror,
            options: Box::new(OrganizationOptions {
                address: (!address.is_empty())
                    .then(|| PostalAddressOrString::String(address.join(", "))),
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    for (part, name) in parts {
        context.locate(&part, &organization, Some(name));
    }

    organization
}

/// Add a typed identifier to a list of identifiers
///
/// An identifier without a type has no property to attach the type to, so is
/// added as a plain string.
pub(super) fn push_identifier(
    identifiers: &mut Option<Vec<PropertyValueOrString>>,
    property_id: Option<String>,
    value: &str,
) {
    let item = match property_id {
        Some(property_id) => PropertyValueOrString::PropertyValue(PropertyValue {
            property_id: Some(property_id),
            value: Primitive::String(value.to_string()),
            options: Box::new(PropertyValueOptions::default()),
            ..Default::default()
        }),
        None => PropertyValueOrString::String(value.to_string()),
    };

    let identifiers = identifiers.get_or_insert_default();
    if !identifiers.contains(&item) {
        identifiers.push(item);
    }
}

/// Convert text into an integer or a string
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
