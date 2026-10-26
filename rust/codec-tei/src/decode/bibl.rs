use roxmltree::Node;

use stencila_codec::stencila_schema::{
    Author, CreativeWorkType, Date, Inline, IntegerOrString, Organization, OrganizationOptions,
    Person, PersonOrOrganization, PostalAddressOrString, Primitive, PropertyValue,
    PropertyValueOptions, PropertyValueOrString, Reference, ReferenceOptions, shortcuts::t,
};

use stencila_codec_utils::{split_given_names, strip_doi_prefix, strip_ror_prefix};

use super::{
    Context,
    metadata::{append, coordinates},
    non_empty_text, path,
    text::normalize,
    xml_id,
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
    doi: Option<String>,
    url: Option<String>,
    authors: Vec<Author>,
    editors: Vec<Person>,
    date: Option<Date>,
    title: Option<Vec<Inline>>,
    source: Option<Vec<Inline>>,
    source_level: SourceLevel,
    is_meeting: bool,
    scope: Scope,
    publisher_name: Option<String>,
    publisher_location: Option<String>,
    identifiers: Option<Vec<PropertyValueOrString>>,
    text: Option<String>,
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
/// | `author`                               | `authors`                              |
/// | `editor`                               | `editors`, of the container for a book |
/// | `biblScope[@unit='volume' or 'issue']` | container's, else the reference's      |
/// | `biblScope[@unit='page']`              | `pageStart`, `pageEnd`                 |
/// | `imprint/date`                         | `date`                                 |
/// | `imprint/publisher`, `pubPlace`        | container's `publisher`, else the reference's |
/// | `idno[@type='DOI']`                    | `doi`                                  |
/// | other `idno`s                          | `identifiers`, keyed by their type     |
/// | `ptr/@target`                          | `url`                                  |
/// | `note[@type='raw_reference']`          | `text`, only if there is no title      |
pub(super) fn decode_bibl_struct(node: &Node, context: &mut Context) -> Reference {
    context.lost_attrs(node, &["id", "coords", "status", "type"]);

    let mut fields = BiblFields {
        identifiers: coordinates(node, None),
        ..Default::default()
    };
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "analytic" => decode_analytic(&child, &mut fields, context),
            "monogr" => decode_monogr(&child, &mut fields, context),
            "idno" => decode_idno(&child, &mut fields.doi, &mut fields.identifiers, context),
            "ptr" => decode_ptr(&child, &mut fields.url, context),
            "note" if child.attribute("type") == Some("raw_reference") => {
                fields.text = non_empty_text(&child);
            }
            "note" => context.lost_label(note_label(&child)),
            _ => context.lost(&child),
        }
    }

    assemble_reference(xml_id(node).map(String::from), fields)
}

/// Decode the `<analytic>` of a `<biblStruct>`: the cited work when it is a
/// part of another work, such as an article in a journal
fn decode_analytic(node: &Node, fields: &mut BiblFields, context: &mut Context) {
    context.lost_attrs(node, &[]);

    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "title" => decode_title(&child, &mut fields.title, context),
            "author" => fields.authors.extend(decode_author(&child, context)),
            "editor" => fields.editors.extend(decode_editor(&child, context)),
            "idno" => decode_idno(&child, &mut fields.doi, &mut fields.identifiers, context),
            "ptr" => decode_ptr(&child, &mut fields.url, context),
            _ => context.lost(&child),
        }
    }
}

/// Decode the `<monogr>` of a `<biblStruct>`: the work that the cited work is
/// part of, or the cited work itself when there is no `<analytic>`
fn decode_monogr(node: &Node, fields: &mut BiblFields, context: &mut Context) {
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
            "author" => fields.authors.extend(decode_author(&child, context)),
            "editor" => fields.editors.extend(decode_editor(&child, context)),
            "idno" => decode_idno(&child, &mut fields.doi, &mut fields.identifiers, context),
            "ptr" => decode_ptr(&child, &mut fields.url, context),
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
fn decode_imprint(node: &Node, fields: &mut BiblFields, context: &mut Context) {
    context.lost_attrs(node, &[]);

    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "biblScope" => decode_bibl_scope(&child, &mut fields.scope, context),
            "date" => {
                if let Some(date) = decode_date(&child, context) {
                    fields.date = Some(Date::new(date));
                }
            }
            "publisher" => {
                context.lost_attrs(&child, &[]);
                fields.publisher_name = non_empty_text(&child);
            }
            "pubPlace" => {
                context.lost_attrs(&child, &[]);
                fields.publisher_location = non_empty_text(&child);
            }
            _ => context.lost(&child),
        }
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

    match unit {
        Some("volume") => set(&mut scope.volume_number, text.as_deref()),
        Some("issue") => set(&mut scope.issue_number, text.as_deref()),
        Some("page") => {
            set(&mut scope.page_start, start);
            set(&mut scope.page_end, end);
        }
        _ => {
            context.lost_label(format!(
                "{}[@unit='{}']",
                path(node),
                unit.unwrap_or_default()
            ));
            return;
        }
    }

    context.lost_attrs(node, &["unit", "from", "to"]);
}

/// Decode a `<title>` to inlines, if it has any text
///
/// A title that is not the main title (e.g. `type="sub"`) when there already is
/// one is recorded as lost.
fn decode_title(node: &Node, title: &mut Option<Vec<Inline>>, context: &mut Context) {
    context.lost_attrs(node, &["level", "type"]);

    let Some(text) = non_empty_text(node) else {
        return;
    };

    if title.is_some() {
        context.lost(node);
        return;
    }

    *title = Some(normalize(vec![t(text)]));
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

/// Decode a `<ptr>` as a URL, keeping the first
///
/// A pointer to a part of the TEI itself (e.g. GROBID's `#string-range(...)`
/// pointers from a reference to where a patent cites it) is not a URL of the
/// work, so is recorded as lost.
fn decode_ptr(node: &Node, url: &mut Option<String>, context: &mut Context) {
    let target = node
        .attribute("target")
        .map(str::trim)
        .filter(|target| !target.is_empty());

    if target.is_some_and(|target| target.starts_with('#')) {
        context.lost(node);
        return;
    }

    context.lost_attrs(node, &["target"]);

    if url.is_none() {
        *url = target.map(String::from);
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
/// with only a monograph title has no `workType`.
fn assemble_reference(id: Option<String>, fields: BiblFields) -> Reference {
    let BiblFields {
        doi,
        url,
        authors,
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
        identifiers,
        text,
    } = fields;

    let authors = (!authors.is_empty()).then_some(authors);
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
        doi,
        authors,
        date,
        url,
        options: Box::new(ReferenceOptions {
            page_start,
            page_end,
            identifiers,
            ..Default::default()
        }),
        ..Default::default()
    };

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
            reference.is_part_of = Some(Box::new(Reference {
                work_type: Some(match source_level {
                    SourceLevel::Journal => CreativeWorkType::Periodical,
                    SourceLevel::Monograph => CreativeWorkType::Book,
                }),
                title: Some(source),
                options: Box::new(ReferenceOptions {
                    // Editors of a book edited the container, not the chapter
                    editors: if is_book { editors.clone() } else { None },
                    publisher,
                    volume_number,
                    issue_number,
                    ..Default::default()
                }),
                ..Default::default()
            }));
            if !is_book {
                reference.options.editors = editors;
            }
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

/// Decode an `<author>` to an [`Author`]
///
/// A person's names are in `<persName>`, along with their `<email>`, ORCID
/// (`<idno type="ORCID">`) and `<affiliation>`s. An author with no personal
/// name but an `<orgName>` is an organization, such as a consortium.
pub(super) fn decode_author(node: &Node, context: &mut Context) -> Option<Author> {
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
    person.options.identifiers = coordinates(node, None);
    let mut org_name = None;
    for child in node.children() {
        if !child.is_element() {
            context.lost(&child);
            continue;
        }
        match child.tag_name().name() {
            "persName" => decode_pers_name(&child, &mut person, context),
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
            "idno" => {
                let mut doi = None;
                decode_idno(&child, &mut doi, &mut person.options.identifiers, context);
                if let Some(doi) = doi {
                    push_identifier(&mut person.options.identifiers, Some("DOI".into()), &doi);
                }
            }
            "affiliation" => person
                .affiliations
                .get_or_insert_default()
                .push(decode_affiliation(&child, context)),
            "orgName" => {
                context.lost_attrs(&child, &["id", "coords", "type"]);
                org_name = Some(child);
            }
            _ => context.lost(&child),
        }
    }

    let has_name = person.family_names.is_some()
        || person.given_names.is_some()
        || person.options.name.is_some();

    // The `orgName` is the name of the author only when it has no personal
    // name, so its id and coordinates are only kept then
    if !has_name {
        let Some((org_name, name)) =
            org_name.and_then(|org_name| Some((org_name, non_empty_text(&org_name)?)))
        else {
            // An author with no name of either kind is not decoded
            context.lost(node);
            return None;
        };

        if person.id.is_some() {
            context.lost_attrs_named(&org_name, &["id"]);
        }
        let mut identifiers = person.options.identifiers;
        append(&mut identifiers, coordinates(&org_name, Some("orgName")));

        return Some(Author::Organization(Organization {
            id: person.id.or_else(|| xml_id(&org_name).map(String::from)),
            name: Some(name),
            options: Box::new(OrganizationOptions {
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }));
    }

    if org_name.is_some() {
        context.lost_label([&path(node), "/orgName"].concat());
    }

    Some(Author::Person(person))
}

/// Decode an `<editor>` to a [`Person`]
fn decode_editor(node: &Node, context: &mut Context) -> Option<Person> {
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
    context.lost_attrs(node, &["id", "coords"]);
    // The author's own id, or that of an earlier `persName`, takes precedence
    if person.id.is_none() {
        person.id = xml_id(node).map(String::from);
    } else {
        context.lost_attrs_named(node, &["id"]);
    }
    append(
        &mut person.options.identifiers,
        coordinates(node, Some("persName")),
    );

    let mut given_names = Vec::new();
    let mut family_names = Vec::new();
    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        let Some(text) = non_empty_text(&child) else {
            continue;
        };
        match child.tag_name().name() {
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
    }

    person.given_names = (!given_names.is_empty()).then_some(given_names);
    person.family_names = (!family_names.is_empty()).then_some(family_names);
}

/// Decode an `<affiliation>` to an [`Organization`]
///
/// As when decoding a JATS `<aff>`, the names of the organization and its
/// divisions (`<orgName>`s, in the order given) are joined to form its name,
/// and the parts of its `<address>` are joined to form its address.
pub(super) fn decode_affiliation(node: &Node, context: &mut Context) -> Organization {
    // The `key` (e.g. "aff0") only identifies the affiliation within the TEI
    context.lost_attrs(node, &["id", "coords", "key"]);

    const TRIM_CHARS: &[char] = &[',', '.', ';', ' '];

    let mut name = Vec::new();
    let mut address = Vec::new();
    let mut ror = None;
    let mut identifiers = coordinates(node, None);
    let mut raw = None;
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "orgName" => {
                context.lost_attrs(&child, &["coords", "type", "key"]);
                append(&mut identifiers, coordinates(&child, Some("orgName")));
                if let Some(text) = non_empty_text(&child) {
                    name.push(text.trim_matches(TRIM_CHARS).to_string());
                }
            }
            "address" => {
                context.lost_attrs(&child, &["coords"]);
                append(&mut identifiers, coordinates(&child, Some("address")));
                for part in child.children().filter(Node::is_element) {
                    // The `key` of a <country> is its ISO 3166 code, which
                    // is not kept because the address is a string
                    context.lost_attrs(&part, &["key"]);
                    if let Some(text) = non_empty_text(&part) {
                        address.push(text.trim_matches(TRIM_CHARS).to_string());
                    }
                }
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

    let name = if name.is_empty() {
        raw.or_else(|| (!address.is_empty()).then(|| std::mem::take(&mut address).join(", ")))
    } else {
        Some(name.join(", "))
    };

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
    }
}

/// The loss label for a note, including its type, if any
fn note_label(node: &Node) -> String {
    match node.attribute("type") {
        Some(note_type) => format!("{}[@type='{note_type}']", path(node)),
        None => path(node),
    }
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
