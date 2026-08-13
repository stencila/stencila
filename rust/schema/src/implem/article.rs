use std::collections::BTreeMap;

use crate::{
    Article, Author, AuthorRoleAuthor, Block, CreativeWorkType, CreativeWorkVariant,
    CreativeWorkVariantOrString, DateTime, Heading, Inline, Organization, Periodical, Person,
    PersonOrOrganization, PostalAddressOrString, Primitive, PropertyValueOrString,
    PublicationIssue, RawBlock, Reference, SectionType, Text, ThingVariant,
    prelude::*,
    replicate,
    shortcuts::{h1, t},
};
use stencila_codec_info::{lost_options, lost_options_of};
use stencila_codec_markdown_trait::{MarkdownEncodeMode, to_markdown_with};
use stencila_codec_text_trait::to_text;

use super::reference::{
    add_organization_losses, add_person_losses, encode_person_name, normalize_doi, normalize_orcid,
    organization_name,
};

/// Get the person represented by an author, including authors wrapped in a role.
fn author_person(author: &Author) -> Option<&Person> {
    match author {
        Author::Person(person) => Some(person),
        Author::AuthorRole(role) => match &role.author {
            AuthorRoleAuthor::Person(person) => Some(person),
            _ => None,
        },
        _ => None,
    }
}

/// Get the displayable parts of a postal address.
fn address_parts(address: &PostalAddressOrString) -> Vec<String> {
    match address {
        PostalAddressOrString::String(address) => {
            let address = address.trim();
            (!address.is_empty())
                .then(|| address.to_string())
                .into_iter()
                .collect()
        }
        PostalAddressOrString::PostalAddress(address) => [
            address.street_address.clone(),
            address
                .options
                .post_office_box_number
                .as_ref()
                .map(|number| format!("PO Box {number}")),
            address.address_locality.clone(),
            address.address_region.clone(),
            address.postal_code.clone(),
            address.address_country.clone(),
        ]
        .into_iter()
        .flatten()
        .filter(|part| !part.trim().is_empty())
        .collect(),
    }
}

/// Generate an identity key for an affiliation.
///
/// Prefer canonical identifiers. When those are unavailable, use all serialized
/// organization metadata rather than the rendered label, so organizations that
/// happen to have the same display name are not incorrectly merged.
fn affiliation_key(organization: &Organization) -> String {
    if let Some(ror) = organization.ror.as_deref().filter(|ror| !ror.is_empty()) {
        return format!(
            "ror:{}",
            ror.trim_start_matches("https://ror.org/")
                .trim_end_matches('/')
        );
    }
    if let Some(id) = organization.id.as_deref().filter(|id| !id.is_empty()) {
        return format!("id:{id}");
    }

    match serde_json::to_string(organization) {
        Ok(json) => format!("organization:{json}"),
        Err(error) => format!("unserialized:{error}:{organization:p}"),
    }
}

/// Whether an affiliation has enough information to display.
fn affiliation_is_displayable(organization: &Organization) -> bool {
    organization
        .name
        .as_deref()
        .or(organization.options.legal_name.as_deref())
        .is_some_and(|name| !name.trim().is_empty())
        || organization
            .options
            .address
            .as_ref()
            .is_some_and(|address| !address_parts(address).is_empty())
        || organization
            .ror
            .as_deref()
            .is_some_and(|ror| !ror.is_empty())
}

/// Collect unique affiliations in author order.
fn article_affiliations(authors: &[Author]) -> Vec<&Organization> {
    let mut affiliations = Vec::new();
    let mut keys = Vec::new();

    for author in authors {
        if let Some(person) = author_person(author)
            && let Some(organizations) = &person.affiliations
        {
            for organization in organizations {
                let key = affiliation_key(organization);
                if affiliation_is_displayable(organization) && !keys.contains(&key) {
                    keys.push(key);
                    affiliations.push(organization);
                }
            }
        }
    }

    affiliations
}

/// Get explicit correspondence emails for a person.
///
/// Unlike `Person.emails`, emails on a structured postal address are defined as
/// correspondence addresses in the schema.
fn correspondence_emails(person: &Person) -> Option<&[String]> {
    match person.options.address.as_ref() {
        Some(PostalAddressOrString::PostalAddress(address)) => address.emails.as_deref(),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum ArticleContactKind {
    Contact,
    Correspondence,
}

struct ArticleContact<'a> {
    author_index: usize,
    author_name: String,
    emails: Vec<&'a str>,
}

/// Collect author contact details while preserving which author they belong to.
fn article_contacts(authors: &[Author], kind: ArticleContactKind) -> Vec<ArticleContact<'_>> {
    authors
        .iter()
        .enumerate()
        .filter_map(|(index, author)| {
            let person = author_person(author)?;
            let correspondence = correspondence_emails(person).unwrap_or_default();
            let emails = match kind {
                ArticleContactKind::Contact => person
                    .options
                    .emails
                    .iter()
                    .flatten()
                    .filter(|email| !correspondence.contains(email))
                    .collect_vec(),
                ArticleContactKind::Correspondence => correspondence.iter().collect_vec(),
            };
            let emails = emails
                .into_iter()
                .map(String::as_str)
                .filter(|email| !email.is_empty())
                .unique()
                .collect_vec();

            (!emails.is_empty()).then(|| ArticleContact {
                author_index: index + 1,
                author_name: person.name(),
                emails,
            })
        })
        .collect()
}

/// Render a list of author contacts with individually addressable details.
fn contacts_to_dom(contacts: &[ArticleContact<'_>], label: &str, context: &mut DomEncodeContext) {
    context
        .enter_elem_attrs("span", [("class", "author-contact-label")])
        .push_text(label)
        .exit_elem();
    context.push_text(" ");

    for (contact_index, contact) in contacts.iter().enumerate() {
        if contact_index > 0 {
            context
                .enter_elem_attrs("span", [("class", "author-contact-separator")])
                .push_text(", ")
                .exit_elem();
        }

        let author_index = contact.author_index.to_string();
        context
            .enter_elem_attrs(
                "span",
                [
                    ("class", "author-contact"),
                    ("data-author-index", &author_index),
                ],
            )
            .enter_elem_attrs("span", [("class", "author-contact-name")])
            .push_text(&contact.author_name)
            .push_text(": ")
            .exit_elem();

        for (email_index, email) in contact.emails.iter().enumerate() {
            if email_index > 0 {
                context
                    .enter_elem_attrs("span", [("class", "author-email-separator")])
                    .push_text(", ")
                    .exit_elem();
            }
            context
                .enter_elem_attrs("a", [("class", "author-email")])
                .push_attr("href", &format!("mailto:{email}"))
                .push_text(email)
                .exit_elem();
        }
        context.exit_elem();
    }
}

/// Convert a DOI, including common URL and label forms, to its canonical URL.
fn doi_url(doi: &str) -> String {
    let doi = doi.trim();
    let lowercase = doi.to_ascii_lowercase();
    let prefixes = [
        "https://doi.org/",
        "http://doi.org/",
        "https://dx.doi.org/",
        "http://dx.doi.org/",
        "https://www.doi.org/",
        "http://www.doi.org/",
        "doi:",
    ];
    let identifier = prefixes
        .iter()
        .find_map(|prefix| {
            lowercase
                .strip_prefix(prefix)
                .map(|_| doi[prefix.len()..].trim())
        })
        .unwrap_or(doi);

    format!("https://doi.org/{identifier}")
}

impl Article {
    /// Does the article appear to be have been decoded from the format using the `--coarse` option
    ///
    /// Checks whether the first block in the content of the article is a `RawBlock` of the given formats
    pub fn is_coarse(&self, format: &Format) -> bool {
        if let Some(Block::RawBlock(raw)) = self.content.first() {
            &Format::from_name(&raw.format) == format
        } else {
            false
        }
    }

    /// Get the `title` property of an article, or generate it from its
    /// `path` property, if any
    pub fn title(&self) -> Option<Vec<Inline>> {
        if let Some(title) = &self.title {
            return replicate(title).ok();
        };

        if let Some(path) = &self.options.path {
            return Some(vec![t(path.to_string())]);
        }

        None
    }

    /// Create a [`Reference`] from the `is_part_of` of an article, or from its
    /// `repository` property, if any
    pub fn is_part_of(&self) -> Option<Reference> {
        if let Some(is_part_of) = &self.options.is_part_of {
            Some(Reference::from(is_part_of))
        } else if let Some(repo) = self.options.repository.clone() {
            if let Some(name) = repo
                .strip_prefix("https://github.com/")
                .or_else(|| repo.strip_prefix("https://gitlab.com/"))
            {
                Some(Reference {
                    work_type: Some(CreativeWorkType::SoftwareRepository),
                    title: Some(vec![t(name)]),
                    url: Some(repo),
                    ..Default::default()
                })
            } else {
                Some(Reference {
                    url: Some(repo),
                    ..Default::default()
                })
            }
        } else {
            None
        }
    }

    /// Generate document-level CSS variables from article metadata
    ///
    /// Extracts metadata like title, authors, dates, and DOI into CSS variable
    /// name/value pairs (without the `--` prefix). These can be used for print
    /// headers/footers or injected into computed theme variables.
    pub fn document_variables(&self) -> BTreeMap<String, String> {
        let mut vars = BTreeMap::new();

        if let Some(title) = &self.title {
            let mut title = to_text(title).replace("\"", "'");
            const MAX_LEN: usize = 120;
            if title.len() > MAX_LEN {
                title.truncate(MAX_LEN);
                title.push('…');
            }
            vars.insert("document-title".to_string(), title);
        }

        if let Some(authors) = &self.authors {
            let authors = match authors.len() {
                0 => String::new(),
                1 => authors[0].short_name(),
                2 => [&authors[0].short_name(), " & ", &authors[1].short_name()].concat(),
                _ => [&authors[0].short_name(), " et al."].concat(),
            };
            vars.insert("document-authors".to_string(), authors.replace("\"", "'"));
        }

        if let Some(date) = self
            .date_published
            .as_ref()
            .or(self.options.date_modified.as_ref())
            .or(self.options.date_accepted.as_ref())
            .or(self.options.date_received.as_ref())
            .or(self.options.date_created.as_ref())
        {
            let date = date
                .value
                .split('T')
                .next()
                .unwrap_or(&date.value)
                .replace("\"", "'");
            vars.insert("document-date".to_string(), date);
        }

        if let Some(doi) = &self.doi {
            vars.insert(
                "document-doi".to_string(),
                format!("DOI: {}", doi.replace("\"", "'")),
            );
            vars.insert(
                "document-doi-url".to_string(),
                doi_url(doi).replace("\"", "'"),
            );
        }

        vars
    }
}

fn encode_text_element(context: &mut JatsEncodeContext, name: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        context.enter_elem(name).push_text(value).exit_elem();
    }
}

fn date_value(date_time: &DateTime) -> Option<&str> {
    let value = date_time.value.split('T').next()?.trim();
    value
        .split('-')
        .next()
        .is_some_and(|year| year.len() == 4 && year.chars().all(|char| char.is_ascii_digit()))
        .then_some(value)
}

fn encode_date_parts(context: &mut JatsEncodeContext, value: &str) {
    let mut parts = value.split('-');
    if let Some(year) = parts.next() {
        encode_text_element(context, "year", year);
    }
    if let Some(month) = parts.next() {
        encode_text_element(context, "month", month);
    }
    if let Some(day) = parts.next() {
        encode_text_element(context, "day", day);
    }
}

fn encode_article_date(
    context: &mut JatsEncodeContext,
    element: &str,
    date_type_attr: Option<&str>,
    date_type: Option<&str>,
    date_time: &DateTime,
    loss: &str,
) {
    let Some(value) = date_value(date_time) else {
        context.add_loss(loss);
        return;
    };

    context.enter_elem(element);
    if let (Some(attr), Some(date_type)) = (date_type_attr, date_type) {
        context.push_attr(attr, date_type);
    }
    context.push_attr("iso-8601-date", value);
    encode_date_parts(context, value);
    context.exit_elem();
}

/// The publication that an article is part of, flattened from the chain of
/// works in `Article.isPartOf`
#[derive(Default)]
struct JournalMetadata<'a> {
    periodical: Option<&'a Periodical>,
    volume: Option<String>,
    issue: Option<String>,
    issue_work: Option<&'a PublicationIssue>,
}

impl JournalMetadata<'_> {
    /// Whether anything at all will be emitted from `Article.isPartOf`
    fn encoded(&self) -> bool {
        self.periodical.is_some()
            || self.volume.is_some()
            || self.issue.is_some()
            || self
                .issue_work
                .is_some_and(|issue| issue.doi.is_some() || issue.options.title.is_some())
    }
}

fn collect_journal_metadata<'a>(work: &'a CreativeWorkVariant, metadata: &mut JournalMetadata<'a>) {
    match work {
        CreativeWorkVariant::Periodical(periodical) => {
            metadata.periodical = Some(periodical);
        }
        CreativeWorkVariant::PublicationVolume(volume) => {
            metadata.volume = volume.volume_number.as_ref().map(TextCodec::to_text);
            if let Some(parent) = &volume.is_part_of {
                collect_journal_metadata(parent, metadata);
            }
        }
        CreativeWorkVariant::PublicationIssue(issue) => {
            metadata.issue = issue.issue_number.as_ref().map(TextCodec::to_text);
            metadata.issue_work = Some(issue);
            if let Some(parent) = &issue.is_part_of {
                collect_journal_metadata(parent, metadata);
            }
        }
        _ => {}
    }
}

/// The text of a primitive value that can be the value of an identifier
fn primitive_text(value: &Primitive) -> Option<String> {
    match value {
        Primitive::String(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        }
        Primitive::Integer(value) => Some(value.to_string()),
        Primitive::UnsignedInteger(value) => Some(value.to_string()),
        Primitive::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

/// An identifier flattened into the parts that JATS represents
struct Identifier<'a> {
    /// The kind of identifier, e.g. `pmid`, from `PropertyValue.propertyId`
    property_id: Option<&'a str>,
    /// A qualifier distinguishing identifiers of the same kind, from `PropertyValue.name`
    name: Option<&'a str>,
    /// The identifier itself
    value: String,
}

/// Flatten identifiers, reporting any whose value can not be represented as text
fn identifiers<'a>(
    identifiers: Option<&'a Vec<PropertyValueOrString>>,
    loss: &str,
    context: &mut JatsEncodeContext,
) -> Vec<Identifier<'a>> {
    let mut flattened = Vec::new();
    for identifier in identifiers.into_iter().flatten() {
        match identifier {
            PropertyValueOrString::String(value) => {
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }
                flattened.push(Identifier {
                    property_id: None,
                    name: None,
                    value: value.to_string(),
                });
            }
            PropertyValueOrString::PropertyValue(property_value) => {
                match primitive_text(&property_value.value) {
                    Some(value) => flattened.push(Identifier {
                        property_id: property_value.property_id.as_deref(),
                        name: property_value.options.name.as_deref(),
                        value,
                    }),
                    None => {
                        context.add_loss(loss);
                    }
                }
            }
        }
    }
    flattened
}

/// Emit a `<journal-meta>` element
fn encode_journal_meta(
    journal: &JournalMetadata,
    publisher: Option<&PersonOrOrganization>,
    context: &mut JatsEncodeContext,
) {
    let periodical = journal.periodical;
    let title = periodical.and_then(|periodical| periodical.name.as_deref());
    let identifiers = identifiers(
        periodical.and_then(|periodical| periodical.options.identifiers.as_ref()),
        "Periodical.identifiers",
        context,
    );

    // An ISSN's publication format is kept as a typed identifier alongside the
    // ISSN itself, and a linking ISSN only as an identifier; see `decode_issn`
    fn issn_format<'a>(identifier: &&Identifier<'a>) -> Option<&'a str> {
        identifier
            .property_id
            .filter(|id| *id != "issn-l")
            .and_then(|id| id.strip_prefix("issn-"))
    }
    let issn_formats = identifiers
        .iter()
        .filter(|identifier| issn_format(identifier).is_some())
        .collect_vec();
    let issn_l = identifiers
        .iter()
        .filter(|identifier| identifier.property_id == Some("issn-l"))
        .collect_vec();
    let journal_ids = identifiers
        .iter()
        .filter(|identifier| {
            issn_format(identifier).is_none() && identifier.property_id != Some("issn-l")
        })
        .collect_vec();

    if periodical.is_none() && publisher.is_none() {
        return;
    }

    context.enter_elem("journal-meta");

    if let Some(doi) = periodical
        .and_then(|periodical| periodical.doi.as_deref())
        .map(normalize_doi)
        .filter(|doi| !doi.is_empty())
    {
        context
            .enter_elem("journal-id")
            .push_attr("journal-id-type", "doi")
            .push_text(doi)
            .exit_elem();
    }
    for identifier in &journal_ids {
        context.enter_elem("journal-id");
        if let Some(property_id) = identifier.property_id {
            context.push_attr("journal-id-type", property_id);
        }
        context.push_text(&identifier.value).exit_elem();
    }

    let alternate_names = periodical
        .and_then(|periodical| periodical.options.alternate_names.as_ref())
        .map(Vec::as_slice)
        .unwrap_or_default();
    if title.is_some() || !alternate_names.is_empty() {
        context.enter_elem("journal-title-group");
        if let Some(title) = title {
            encode_text_element(context, "journal-title", title);
        }
        for name in alternate_names {
            encode_text_element(context, "abbrev-journal-title", name);
        }
        context.exit_elem_omit_empty();
    }

    for issn in periodical
        .and_then(|periodical| periodical.options.issns.as_ref())
        .into_iter()
        .flatten()
        .map(|issn| issn.trim())
        .filter(|issn| !issn.is_empty())
    {
        context.enter_elem("issn");
        if let Some(format) = issn_formats
            .iter()
            .find(|identifier| identifier.value == issn)
            .and_then(issn_format)
        {
            context.push_attr("pub-type", format);
        }
        context.push_text(issn).exit_elem();
    }
    for identifier in &issn_l {
        encode_text_element(context, "issn-l", &identifier.value);
    }

    if let Some(publisher) = publisher {
        encode_publisher(publisher, context);
    }

    // Any other periodical metadata is not part of a JATS `<journal-meta>`
    if let Some(periodical) = periodical {
        context.merge_losses(lost_options_of!(
            "Periodical",
            periodical.options,
            description,
            url,
            about,
            r#abstract,
            authors,
            contributors,
            editors,
            date_start,
            date_end,
            date_published,
            keywords,
            licenses,
            parts,
            publisher,
            references,
            title
        ));
    }

    context.exit_elem_omit_empty();
}

/// A path through a hierarchical subject taxonomy
struct SubjectTree {
    subject: String,
    children: Vec<SubjectTree>,
}

impl SubjectTree {
    /// Add the remainder of a subject path below this subject
    fn insert(&mut self, path: &[String]) {
        let Some((subject, rest)) = path.split_first() else {
            return;
        };

        let index = match self
            .children
            .iter()
            .position(|child| &child.subject == subject)
        {
            Some(index) => index,
            None => {
                self.children.push(SubjectTree {
                    subject: subject.clone(),
                    children: Vec::new(),
                });
                self.children.len() - 1
            }
        };

        self.children[index].insert(rest);
    }
}

/// Rebuild the subject hierarchies that `Article.about` was decoded from
///
/// Each entry is one path through a taxonomy. Paths that share a group type and
/// a broadest subject belong to the same `<subj-group>`; paths that do not are
/// kept separate so that their broadest subjects are not conflated.
fn subject_trees(
    article: &Article,
    context: &mut JatsEncodeContext,
) -> Vec<(Option<String>, SubjectTree)> {
    let mut groups: Vec<(Option<String>, SubjectTree)> = Vec::new();

    for thing in article.options.about.iter().flatten() {
        let ThingVariant::PropertyValue(property_value) = thing else {
            context.add_loss("Article.about");
            continue;
        };

        let path = match &property_value.value {
            Primitive::Array(values) => values.iter().filter_map(primitive_text).collect_vec(),
            value => primitive_text(value).into_iter().collect_vec(),
        };
        let Some((subject, rest)) = path.split_first() else {
            context.add_loss("Article.about");
            continue;
        };

        let property_id = property_value.property_id.clone();
        match groups
            .iter_mut()
            .find(|(id, tree)| id == &property_id && &tree.subject == subject)
        {
            Some((.., tree)) => tree.insert(rest),
            None => {
                let mut tree = SubjectTree {
                    subject: subject.clone(),
                    children: Vec::new(),
                };
                tree.insert(rest);
                groups.push((property_id, tree));
            }
        }
    }

    groups
}

/// Emit a `<subj-group>` element and the nested groups of its narrower subjects
fn encode_subj_group(
    tree: &SubjectTree,
    group_type: Option<&str>,
    context: &mut JatsEncodeContext,
) {
    context.enter_elem("subj-group");
    if let Some(group_type) = group_type {
        context.push_attr("subj-group-type", group_type);
    }
    encode_text_element(context, "subject", &tree.subject);
    for child in &tree.children {
        encode_subj_group(child, None, context);
    }
    context.exit_elem();
}

/// Publication metadata retained in `Article.extra` by the JATS decoder
///
/// See `ArticleMetaExtra` in the `stencila-codec-jats` crate for how each of
/// these is decoded.
#[derive(Default)]
struct ArticleExtra<'a> {
    publication_dates: Vec<(Option<&'a str>, &'a str)>,
    history_dates: Vec<(Option<&'a str>, &'a str)>,
    publication_history: Vec<PubHistoryEvent<'a>>,
    resources: Vec<(Option<&'a str>, Option<&'a str>, &'a str)>,
    copyright_statement: Option<&'a str>,
    copyright_year: Option<&'a str>,
    copyright_holder: Option<&'a str>,
    license_text: Option<&'a str>,
}

/// An event in the publication history of an article
#[derive(Default)]
struct PubHistoryEvent<'a> {
    event_type: Option<&'a str>,
    date_type: Option<&'a str>,
    description: Option<&'a str>,
    date: &'a str,
    url: Option<&'a str>,
}

/// Read the retained publication metadata, reporting anything else as lost
fn article_extra<'a>(article: &'a Article, context: &mut JatsEncodeContext) -> ArticleExtra<'a> {
    let mut extra = ArticleExtra::default();

    let string = |value: &'a Primitive| match value {
        Primitive::String(value) => Some(value.as_str()),
        _ => None,
    };

    let typed_dates = |value: &'a Primitive| -> Vec<(Option<&'a str>, &'a str)> {
        let Primitive::Array(values) = value else {
            return Vec::new();
        };
        values
            .iter()
            .filter_map(|value| {
                let Primitive::Object(object) = value else {
                    return None;
                };
                let date = object.get("date").and_then(string)?;
                Some((object.get("type").and_then(string), date))
            })
            .collect()
    };

    for (name, value) in article.options.extra.iter().flat_map(|extra| extra.iter()) {
        match name.as_str() {
            "publicationDates" => extra.publication_dates = typed_dates(value),
            "historyDates" => extra.history_dates = typed_dates(value),
            "publicationHistory" => {
                if let Primitive::Array(values) = value {
                    for value in values.iter() {
                        let Primitive::Object(object) = value else {
                            continue;
                        };
                        if let Some(date) = object.get("date").and_then(string) {
                            extra.publication_history.push(PubHistoryEvent {
                                event_type: object.get("eventType").and_then(string),
                                date_type: object.get("dateType").and_then(string),
                                description: object.get("description").and_then(string),
                                date,
                                url: object.get("url").and_then(string),
                            });
                        }
                    }
                }
            }
            "resources" => {
                if let Primitive::Array(values) = value {
                    for value in values.iter() {
                        let Primitive::Object(object) = value else {
                            continue;
                        };
                        if let Some(url) = object.get("url").and_then(string) {
                            extra.resources.push((
                                object.get("type").and_then(string),
                                object.get("role").and_then(string),
                                url,
                            ));
                        }
                    }
                }
            }
            "copyrightStatement" => extra.copyright_statement = string(value),
            "copyrightYear" => extra.copyright_year = string(value),
            "copyrightHolder" => extra.copyright_holder = string(value),
            "licenseText" => extra.license_text = string(value),
            _ => {
                context.add_loss("Article.extra");
            }
        }
    }

    extra
}

/// Emit the `<pub-date>` elements of an article
///
/// A JATS article usually has several publication dates, of which
/// `datePublished` is the most specific. The others, and the kind of each date,
/// are only available when they were retained by the decoder.
fn encode_pub_dates(article: &Article, extra: &ArticleExtra, context: &mut JatsEncodeContext) {
    if extra.publication_dates.is_empty() {
        if let Some(date) = &article.date_published {
            encode_article_date(
                context,
                "pub-date",
                None,
                None,
                date,
                "Article.datePublished",
            );
        }
        return;
    }

    for (pub_type, date) in &extra.publication_dates {
        encode_article_date(
            context,
            "pub-date",
            Some("pub-type"),
            *pub_type,
            &DateTime::new(date.to_string()),
            "Article.datePublished",
        );
    }
}

/// Emit the `<history>` of an article
fn encode_history(article: &Article, extra: &ArticleExtra, context: &mut JatsEncodeContext) {
    if !extra.history_dates.is_empty() {
        context.enter_elem("history");
        for (date_type, date) in &extra.history_dates {
            encode_article_date(
                context,
                "date",
                Some("date-type"),
                *date_type,
                &DateTime::new(date.to_string()),
                "Article.dateReceived",
            );
        }
        context.exit_elem_omit_empty();
        return;
    }

    if article.options.date_received.is_none() && article.options.date_accepted.is_none() {
        return;
    }

    context.enter_elem("history");
    if let Some(date) = &article.options.date_received {
        encode_article_date(
            context,
            "date",
            Some("date-type"),
            Some("received"),
            date,
            "Article.dateReceived",
        );
    }
    if let Some(date) = &article.options.date_accepted {
        encode_article_date(
            context,
            "date",
            Some("date-type"),
            Some("accepted"),
            date,
            "Article.dateAccepted",
        );
    }
    context.exit_elem_omit_empty();
}

/// Emit the `<pub-history>` of an article
fn encode_pub_history(extra: &ArticleExtra, context: &mut JatsEncodeContext) {
    if extra.publication_history.is_empty() {
        return;
    }

    context.enter_elem("pub-history");
    for event in &extra.publication_history {
        context.enter_elem("event");
        if let Some(event_type) = event.event_type {
            context.push_attr("event-type", event_type);
        }
        if let Some(description) = event.description {
            encode_text_element(context, "event-desc", description);
        }
        encode_article_date(
            context,
            "date",
            Some("date-type"),
            event.date_type,
            &DateTime::new(event.date.to_string()),
            "Article.extra",
        );
        if let Some(url) = event.url {
            context
                .enter_elem("self-uri")
                .push_attr("xlink:href", url)
                .exit_elem();
        }
        context.exit_elem();
    }
    context.exit_elem_omit_empty();
}

/// Emit the `<permissions>` of an article
fn encode_permissions(article: &Article, extra: &ArticleExtra, context: &mut JatsEncodeContext) {
    let licenses = article.options.licenses.iter().flatten().collect_vec();
    if licenses.is_empty()
        && extra.copyright_statement.is_none()
        && extra.copyright_year.is_none()
        && extra.copyright_holder.is_none()
        && extra.license_text.is_none()
    {
        return;
    }

    context.enter_elem("permissions");
    for (element, value) in [
        ("copyright-statement", extra.copyright_statement),
        ("copyright-year", extra.copyright_year),
        ("copyright-holder", extra.copyright_holder),
    ] {
        if let Some(value) = value {
            encode_text_element(context, element, value);
        }
    }

    // Any license prose belongs to the first license, which is where the
    // decoder takes it from
    let mut license_text = extra.license_text;

    for license in licenses {
        match license {
            CreativeWorkVariantOrString::String(url) => {
                context.enter_elem("license").push_attr("xlink:href", url);
                encode_license_text(&mut license_text, context);
                context.exit_elem();
            }
            _ => {
                context.add_loss("Article.licenses");
            }
        }
    }
    if license_text.is_some() {
        context.enter_elem("license");
        encode_license_text(&mut license_text, context);
        context.exit_elem_omit_empty();
    }

    context.exit_elem_omit_empty();
}

/// Emit the prose of a license, once, as `<license-p>` paragraphs
fn encode_license_text(text: &mut Option<&str>, context: &mut JatsEncodeContext) {
    let Some(text) = text.take() else {
        return;
    };

    for paragraph in text.split("\n\n").filter(|para| !para.trim().is_empty()) {
        encode_text_element(context, "license-p", paragraph);
    }
}

fn encode_publisher(publisher: &PersonOrOrganization, context: &mut JatsEncodeContext) {
    let name = match publisher {
        PersonOrOrganization::Person(person) => {
            let name = person.name();
            (!name.trim().is_empty()).then_some(name)
        }
        PersonOrOrganization::Organization(organization) => {
            organization_name(organization).map(str::to_string)
        }
    };
    if let Some(name) = name {
        context.enter_elem("publisher");
        encode_text_element(context, "publisher-name", &name);
        context.exit_elem();
    } else {
        context.add_loss("Article.publisher");
    }
}

fn encode_article_author(
    author: &Author,
    affiliations: &BTreeMap<String, String>,
    context: &mut JatsEncodeContext,
) {
    context
        .enter_elem("contrib")
        .push_attr("contrib-type", "author");

    let (person, role, encoded) = match author {
        Author::Person(person) => {
            add_person_losses(person, true, context);
            (Some(person), None, encode_person_name(person, context))
        }
        Author::Organization(organization) => {
            let encoded = organization_name(organization).is_some_and(|name| {
                context.enter_elem("collab").push_text(name).exit_elem();
                true
            });
            (None, None, encoded)
        }
        Author::SoftwareApplication(software) => {
            let name = software.name.trim();
            let encoded = if name.is_empty() {
                false
            } else {
                context.enter_elem("collab").push_text(name).exit_elem();
                true
            };
            (None, None, encoded)
        }
        Author::AuthorRole(author_role) => match &author_role.author {
            AuthorRoleAuthor::Person(person) => {
                add_person_losses(person, true, context);
                (
                    Some(person),
                    Some(author_role.role_name.to_string()),
                    encode_person_name(person, context),
                )
            }
            AuthorRoleAuthor::Organization(organization) => {
                let encoded = organization_name(organization).is_some_and(|name| {
                    context.enter_elem("collab").push_text(name).exit_elem();
                    true
                });
                (None, Some(author_role.role_name.to_string()), encoded)
            }
            AuthorRoleAuthor::SoftwareApplication(software) => {
                let name = software.name.trim();
                let encoded = if name.is_empty() {
                    false
                } else {
                    context.enter_elem("collab").push_text(name).exit_elem();
                    true
                };
                (None, Some(author_role.role_name.to_string()), encoded)
            }
            AuthorRoleAuthor::Thing(..) => {
                context.add_loss("AuthorRole.author");
                (None, Some(author_role.role_name.to_string()), false)
            }
        },
    };

    if !encoded {
        context.add_loss("Article.authors");
        context.exit_elem_omit_empty();
        return;
    }
    if let Some(person) = person {
        if let Some(orcid) = person
            .orcid
            .as_deref()
            .map(str::trim)
            .filter(|orcid| !orcid.is_empty())
        {
            context
                .enter_elem("contrib-id")
                .push_attr("contrib-id-type", "orcid")
                .push_text(normalize_orcid(orcid))
                .exit_elem();
        }
        if let Some(emails) = &person.options.emails {
            for email in emails
                .iter()
                .map(|email| email.trim())
                .filter(|email| !email.is_empty())
            {
                encode_text_element(context, "email", email);
            }
        }
        if let Some(organizations) = &person.affiliations {
            for organization in organizations {
                if let Some(id) = affiliations.get(&affiliation_key(organization)) {
                    context
                        .enter_elem("xref")
                        .push_attr("ref-type", "aff")
                        .push_attr("rid", id)
                        .exit_elem();
                }
            }
        }
    }
    if let Some(role) = role {
        encode_text_element(context, "role", &role);
    }
    context.exit_elem();
}

fn encode_affiliation(organization: &Organization, id: &str, context: &mut JatsEncodeContext) {
    context.enter_elem("aff").push_attr("id", id);
    add_organization_losses(organization, context);
    let name = organization_name(organization);
    let ror = organization
        .ror
        .as_deref()
        .map(str::trim)
        .filter(|ror| !ror.is_empty());
    if name.is_some() || ror.is_some() {
        context.enter_elem("institution-wrap");
        if let Some(name) = name {
            encode_text_element(context, "institution", name);
        }
        if let Some(ror) = ror {
            let ror = format!(
                "https://ror.org/{}",
                ror.trim_start_matches("https://ror.org/")
                    .trim_start_matches("http://ror.org/")
                    .trim_end_matches('/')
            );
            context
                .enter_elem("institution-id")
                .push_attr("institution-id-type", "ror")
                .push_text(ror)
                .exit_elem();
        }
        context.exit_elem();
    }
    if let Some(address) = organization.options.address.as_ref() {
        let address = address_parts(address).join(", ");
        encode_text_element(context, "addr-line", &address);
    }
    context.exit_elem();
}

/// Emit a `<contrib-group>` for the authors of an article, followed by the
/// `<aff>` elements that its contributors reference
fn encode_contrib_group(authors: &[Author], context: &mut JatsEncodeContext) {
    let organizations = article_affiliations(authors);
    let affiliations = organizations
        .iter()
        .enumerate()
        .map(|(index, organization)| (affiliation_key(organization), format!("aff{}", index + 1)))
        .collect::<BTreeMap<_, _>>();
    context.enter_elem("contrib-group");
    for author in authors {
        encode_article_author(author, &affiliations, context);
    }
    for organization in organizations {
        if let Some(id) = affiliations.get(&affiliation_key(organization)) {
            encode_affiliation(organization, id, context);
        }
    }
    context.exit_elem_omit_empty();
}

/// How a nested work in `Article.parts` is represented in JATS
enum ArticlePart<'a> {
    /// An additional `<abstract>`, e.g. a graphical or plain language abstract
    Abstract(&'a Article),
    /// A `<sub-article>`, e.g. a review or an editor's assessment
    SubArticle(&'a Article),
}

/// Classify the nested works of an article
///
/// A part that carries only an `abstract` came from an additional `<abstract>`
/// element; any other nested article came from a `<sub-article>`. Parts that are
/// neither are reported as lost.
fn article_parts<'a>(
    article: &'a Article,
    context: &mut JatsEncodeContext,
) -> Vec<ArticlePart<'a>> {
    let mut parts = Vec::new();
    for part in article.options.parts.iter().flatten() {
        match part {
            CreativeWorkVariant::Article(article) => {
                if article.r#abstract.is_some()
                    && article.content.is_empty()
                    && article.title.is_none()
                {
                    parts.push(ArticlePart::Abstract(article));
                } else {
                    parts.push(ArticlePart::SubArticle(article));
                }
            }
            _ => {
                context.add_loss("Article.parts");
            }
        }
    }
    parts
}

/// Emit an additional `<abstract>` element for a nested work
fn encode_extra_abstract(article: &Article, context: &mut JatsEncodeContext) {
    let Some(content) = article
        .r#abstract
        .as_ref()
        .filter(|content| !content.is_empty())
    else {
        return;
    };

    context.enter_elem("abstract");
    if let Some(id) = &article.id {
        context.push_attr("id", id);
    }
    if let Some(abstract_type) = article.options.genre.iter().flatten().next() {
        context.push_attr("abstract-type", abstract_type);
    }
    content.to_jats(context);
    context.exit_elem_omit_empty();
}

/// Emit a `<sub-article>` element for a nested work
///
/// The nested work's own metadata goes in a `<front-stub>`, which is the
/// article metadata of a sub-article.
fn encode_sub_article(article: &Article, context: &mut JatsEncodeContext) {
    context.enter_elem("sub-article");
    if let Some(id) = &article.id {
        context.push_attr("id", id);
    }
    if let Some(article_type) = article.options.genre.iter().flatten().next() {
        context.push_attr("article-type", article_type);
    }

    context.enter_elem("front-stub");
    if let Some(doi) = article
        .doi
        .as_deref()
        .map(normalize_doi)
        .filter(|doi| !doi.is_empty())
    {
        context
            .enter_elem("article-id")
            .push_attr("pub-id-type", "doi")
            .push_text(doi)
            .exit_elem();
    }
    if let Some(title) = article.title.as_ref().filter(|title| !title.is_empty()) {
        context
            .enter_elem("title-group")
            .enter_elem("article-title");
        title.to_jats(context);
        context.exit_elem_omit_empty().exit_elem_omit_empty();
    }
    if let Some(authors) = article
        .authors
        .as_ref()
        .filter(|authors| !authors.is_empty())
    {
        encode_contrib_group(authors, context);
    }
    if let Some(content) = article
        .r#abstract
        .as_ref()
        .filter(|content| !content.is_empty())
    {
        context.enter_elem("abstract");
        content.to_jats(context);
        context.exit_elem_omit_empty();
    }
    context.exit_elem_omit_empty();

    context.enter_elem("body");
    article.content.to_jats(context);
    context.exit_elem_omit_empty();

    context.exit_elem();

    // Core properties that a top level article emits but a sub-article does not
    for (populated, label) in [
        (article.date_published.is_some(), "Article.datePublished"),
        (article.options.keywords.is_some(), "Article.keywords"),
        (article.references.is_some(), "Article.references"),
    ] {
        if populated {
            context.add_loss(label);
        }
    }
}

/// Record a loss for every populated `Article` property that JATS encoding does
/// not emit
///
/// `journal_encoded` indicates whether anything was emitted from `is_part_of`,
/// and `genre_encoded` whether the genre was emitted, which it is for a nested
/// work but not for a top level article. `full` indicates whether the article
/// is emitted with a complete `<article-meta>`; a nested work has only the
/// subset of metadata that a `<front-stub>` or an `<abstract>` can carry.
fn add_article_losses(
    article: &Article,
    journal_encoded: bool,
    genre_encoded: bool,
    full: bool,
    context: &mut JatsEncodeContext,
) {
    context
        .merge_losses(lost_options!(
            article,
            provenance,
            execution_mode,
            frontmatter
        ))
        .merge_losses(lost_options_of!(
            "Article",
            article.options,
            alternate_names,
            description,
            images,
            name,
            url,
            contributors,
            editors,
            maintainers,
            comments,
            date_created,
            date_modified,
            funders,
            funded_by,
            bibliography,
            text,
            repository,
            path,
            commit,
            worktree_status,
            headings,
            archive
        ));

    if !full {
        context.merge_losses(lost_options_of!(
            "Article",
            article.options,
            identifiers,
            about,
            licenses,
            version,
            extra
        ));
    }

    if article.options.is_part_of.is_some() && !journal_encoded {
        context.add_loss("Article.isPartOf");
    }

    // Only the first genre is emitted, as the JATS `article-type` or `abstract-type`
    let genres = article.options.genre.iter().flatten().count();
    if genres > usize::from(genre_encoded) {
        context.add_loss("Article.genre");
    }
}

impl JatsCodec for Article {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        let reference_ids = self
            .references
            .iter()
            .flatten()
            .enumerate()
            .map(|(index, reference)| {
                context.register_reference_id(reference.id.as_deref(), &format!("ref{}", index + 1))
            })
            .collect_vec();

        context
            .enter_elem("article")
            .push_attr("dtd-version", "1.4")
            .push_attr("xmlns:xlink", "http://www.w3.org/1999/xlink")
            .push_attr("xmlns:mml", "http://www.w3.org/1998/Math/MathML");
        if let Some(id) = &self.id {
            context.push_attr("id", id);
        }

        context.enter_elem("front");

        let mut journal = JournalMetadata::default();
        if let Some(work) = &self.options.is_part_of {
            collect_journal_metadata(work, &mut journal);
        }
        add_article_losses(self, journal.encoded(), false, true, context);

        let parts = article_parts(self, context);
        let extra = article_extra(self, context);

        encode_journal_meta(&journal, self.options.publisher.as_ref(), context);

        context.enter_elem("article-meta");
        if let Some(doi) = self
            .doi
            .as_deref()
            .map(normalize_doi)
            .filter(|doi| !doi.is_empty())
        {
            context
                .enter_elem("article-id")
                .push_attr("pub-id-type", "doi")
                .push_text(doi)
                .exit_elem();
        }

        // An electronic location identifier is emitted with the pagination it
        // stands in for, not with the other identifiers
        let article_identifiers = identifiers(
            self.options.identifiers.as_ref(),
            "Article.identifiers",
            context,
        );
        let (elocation_ids, article_ids): (Vec<_>, Vec<_>) = article_identifiers
            .iter()
            .partition(|identifier| identifier.property_id == Some("elocation-id"));
        for identifier in &article_ids {
            context.enter_elem("article-id");
            if let Some(property_id) = identifier.property_id {
                context.push_attr("pub-id-type", property_id);
            }
            if let Some(name) = identifier.name {
                context.push_attr("specific-use", name);
            }
            context.push_text(&identifier.value).exit_elem();
        }

        if let Some(version) = &self.options.version {
            encode_text_element(context, "article-version", &version.to_text());
        }

        let subjects = subject_trees(self, context);
        if !subjects.is_empty() {
            context.enter_elem("article-categories");
            for (group_type, tree) in &subjects {
                encode_subj_group(tree, group_type.as_deref(), context);
            }
            context.exit_elem_omit_empty();
        }

        if let Some(title) = self.title.as_ref().filter(|title| !title.is_empty()) {
            context
                .enter_elem("title-group")
                .enter_elem("article-title");
            title.to_jats(context);
            context.exit_elem_omit_empty().exit_elem_omit_empty();
        }

        if let Some(authors) = self.authors.as_ref().filter(|authors| !authors.is_empty()) {
            encode_contrib_group(authors, context);
        }

        encode_pub_dates(self, &extra, context);

        if let Some(volume) = journal.volume.as_deref() {
            encode_text_element(context, "volume", volume);
        }
        if let Some(issue) = journal.issue.as_deref() {
            encode_text_element(context, "issue", issue);
        }
        if let Some(issue_work) = journal.issue_work {
            if let Some(doi) = issue_work
                .doi
                .as_deref()
                .map(normalize_doi)
                .filter(|doi| !doi.is_empty())
            {
                context
                    .enter_elem("issue-id")
                    .push_attr("pub-id-type", "doi")
                    .push_text(doi)
                    .exit_elem();
            }
            for identifier in &identifiers(
                issue_work.options.identifiers.as_ref(),
                "PublicationIssue.identifiers",
                context,
            ) {
                context.enter_elem("issue-id");
                if let Some(property_id) = identifier.property_id {
                    context.push_attr("pub-id-type", property_id);
                }
                context.push_text(&identifier.value).exit_elem();
            }
            if let Some(title) = issue_work
                .options
                .title
                .as_ref()
                .filter(|title| !title.is_empty())
            {
                context.enter_elem("issue-title");
                title.to_jats(context);
                context.exit_elem_omit_empty();
            }
        }

        let mut paginated = false;
        for (name, value) in [
            (
                "fpage",
                self.options.page_start.as_ref().map(TextCodec::to_text),
            ),
            (
                "lpage",
                self.options.page_end.as_ref().map(TextCodec::to_text),
            ),
            ("page-range", self.options.pagination.clone()),
        ] {
            if let Some(value) = value {
                paginated = true;
                encode_text_element(context, name, &value);
            }
        }
        // JATS allows either pagination or an electronic location, not both
        if !paginated {
            for identifier in &elocation_ids {
                encode_text_element(context, "elocation-id", &identifier.value);
            }
        } else if !elocation_ids.is_empty() {
            context.add_loss("Article.identifiers");
        }

        encode_history(self, &extra, context);
        encode_pub_history(&extra, context);
        encode_permissions(self, &extra, context);

        for (content_type, role, url) in &extra.resources {
            context.enter_elem("self-uri");
            if let Some(content_type) = content_type {
                context.push_attr("content-type", content_type);
            }
            if let Some(role) = role {
                context.push_attr("xlink:role", role);
            }
            context.push_attr("xlink:href", url).exit_elem();
        }

        if let Some(abstract_content) = self
            .r#abstract
            .as_ref()
            .filter(|content| !content.is_empty())
        {
            context.enter_elem("abstract");
            abstract_content.to_jats(context);
            context.exit_elem_omit_empty();
        }
        for part in &parts {
            if let ArticlePart::Abstract(article) = part {
                add_article_losses(article, false, true, false, context);
                encode_extra_abstract(article, context);
            }
        }
        if let Some(keywords) = self.options.keywords.as_ref() {
            context.enter_elem("kwd-group");
            for keyword in keywords
                .iter()
                .map(|keyword| keyword.trim())
                .filter(|keyword| !keyword.is_empty())
            {
                encode_text_element(context, "kwd", keyword);
            }
            context.exit_elem_omit_empty();
        }
        context.exit_elem().exit_elem();

        context.enter_elem("body");
        for block in &self.content {
            match block {
                Block::Section(section)
                    if matches!(
                        section.section_type,
                        Some(SectionType::Acknowledgements | SectionType::Appendix)
                    ) => {}
                Block::AppendixBreak(..) => {}
                block => block.to_jats(context),
            }
        }
        context.exit_elem_omit_empty();

        context.enter_elem("back");
        for block in &self.content {
            if let Block::Section(section) = block
                && matches!(section.section_type, Some(SectionType::Acknowledgements))
            {
                context.enter_elem("ack");
                section.content.to_jats(context);
                context.exit_elem_omit_empty();
            }
        }

        let appendices = self.content.iter().filter_map(|block| match block {
            Block::Section(section)
                if matches!(section.section_type, Some(SectionType::Appendix)) =>
            {
                Some(section)
            }
            _ => None,
        });
        context.enter_elem("app-group");
        for section in appendices {
            context.enter_elem("app");
            if let Some(id) = &section.id {
                context.push_attr("id", id);
            }
            section.content.to_jats(context);
            context.exit_elem_omit_empty();
        }
        context.exit_elem_omit_empty();

        if let Some(references) = &self.references
            && !references.is_empty()
        {
            context.enter_elem("ref-list");
            for (reference, id) in references.iter().zip(reference_ids) {
                reference.to_jats_with_id(&id, context);
            }
            context.exit_elem_omit_empty();
        }
        context.exit_elem_omit_empty();

        for part in &parts {
            if let ArticlePart::SubArticle(article) = part {
                add_article_losses(article, false, true, false, context);
                encode_sub_article(article, context);
            }
        }

        context.exit_elem();
    }
}

impl DomCodec for Article {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        self.doi.to_dom_attr("doi", context);
        self.options.identifiers.to_dom_attr("identifiers", context);

        self.options
            .date_created
            .to_dom_attr("date-created", context);
        self.options
            .date_modified
            .to_dom_attr("date-modified", context);
        self.options
            .date_received
            .to_dom_attr("date-received", context);
        self.options
            .date_accepted
            .to_dom_attr("date-accepted", context);
        self.date_published.to_dom_attr("date-published", context);

        self.options.is_part_of.to_dom_attr("is-part-of", context);
        self.options.page_start.to_dom_attr("page-start", context);
        self.options.page_end.to_dom_attr("page-end", context);

        self.options.repository.to_dom_attr("repository", context);
        self.options.path.to_dom_attr("path", context);
        self.options.commit.to_dom_attr("commit", context);

        if context.is_root() {
            // Generate CSS variables for print media support from document metadata
            let doc_vars = self.document_variables();
            if !doc_vars.is_empty() {
                let mut css = String::new();
                for (name, value) in doc_vars {
                    css.push_str(&format!("--{name}: \"{value}\";"));
                }
                context.push_css(&[":root {", &css, "}"].concat());
            }

            if let Some(title) = &self.title {
                // We do not use <h1> or <header><p role="heading" aria-level="1"> for title
                // because  bot result in the title being treated as a level one header
                // when generating a PDF. Instead we use the ARIA "banner" role.
                context.push_slot_fn("header", "title", |context| {
                    context.push_attr("role", "banner").enter_elem("p");
                    title.to_dom(context);
                    context.exit_elem();
                });
            }

            if let Some(authors) = &self.authors {
                let affiliations = article_affiliations(authors);
                let affiliation_keys = affiliations
                    .iter()
                    .map(|organization| affiliation_key(organization))
                    .collect_vec();
                let contacts = article_contacts(authors, ArticleContactKind::Contact);
                let correspondence = article_contacts(authors, ArticleContactKind::Correspondence);

                context.push_slot_fn("section", "authors", |context| {
                    for (index, author) in authors.iter().enumerate() {
                        if index > 0 {
                            let (kind, separator) = if index + 1 == authors.len() {
                                ("last", " and ")
                            } else {
                                ("middle", ", ")
                            };
                            context
                                .enter_elem_attrs(
                                    "span",
                                    [
                                        ("class", "article-author-separator"),
                                        ("data-position", kind),
                                    ],
                                )
                                .push_text(separator)
                                .exit_elem();
                        }
                        context
                            .enter_node(author.node_type(), author.node_id())
                            .push_attr("data-author-index", &(index + 1).to_string())
                            .push_slot_fn("span", "name", |context| {
                                context
                                    .enter_elem_attrs("span", [("class", "article-author-name")])
                                    .push_text(&author.name())
                                    .exit_elem();

                                if let Some(person) = author_person(author) {
                                    let affiliation_numbers = person
                                        .affiliations
                                        .iter()
                                        .flatten()
                                        .filter_map(|organization| {
                                            let key = affiliation_key(organization);
                                            affiliation_keys
                                                .iter()
                                                .position(|item| item == &key)
                                                .map(|position| position + 1)
                                        })
                                        .unique()
                                        .collect_vec();
                                    let corresponding =
                                        correspondence_emails(person).is_some_and(|emails| {
                                            emails.iter().any(|email| !email.is_empty())
                                        });

                                    if !affiliation_numbers.is_empty() || corresponding {
                                        context.enter_elem_attrs(
                                            "sup",
                                            [("class", "author-affiliation-marks")],
                                        );
                                        for (mark_index, number) in
                                            affiliation_numbers.iter().enumerate()
                                        {
                                            if mark_index > 0 {
                                                context
                                                    .enter_elem_attrs(
                                                        "span",
                                                        [(
                                                            "class",
                                                            "author-affiliation-mark-separator",
                                                        )],
                                                    )
                                                    .push_text(",")
                                                    .exit_elem();
                                            }

                                            let number = number.to_string();
                                            context
                                                .enter_elem_attrs(
                                                    "a",
                                                    [
                                                        ("class", "author-affiliation-mark"),
                                                        ("data-affiliation-index", &number),
                                                    ],
                                                )
                                                .push_attr(
                                                    "href",
                                                    &format!("#article-affiliation-{number}"),
                                                )
                                                .push_text(&number)
                                                .exit_elem();
                                        }
                                        if corresponding {
                                            if !affiliation_numbers.is_empty() {
                                                context
                                                    .enter_elem_attrs(
                                                        "span",
                                                        [(
                                                            "class",
                                                            "author-affiliation-mark-separator",
                                                        )],
                                                    )
                                                    .push_text(",")
                                                    .exit_elem();
                                            }
                                            context
                                                .enter_elem_attrs(
                                                    "span",
                                                    [("class", "author-correspondence-mark")],
                                                )
                                                .push_text("*")
                                                .exit_elem();
                                        }
                                        context.exit_elem();
                                    }
                                }
                            })
                            .exit_node();
                    }
                });

                if !affiliations.is_empty() {
                    context.push_slot_fn("section", "affiliations", |context| {
                        context.push_attr("class", "author-affiliations");
                        for (index, organization) in affiliations.iter().enumerate() {
                            let number = (index + 1).to_string();
                            context.enter_elem_attrs(
                                "div",
                                [
                                    ("class", "author-affiliation"),
                                    ("id", &format!("article-affiliation-{number}")),
                                ],
                            );
                            context.push_attr("data-affiliation-index", &number);
                            if let Some(id) = &organization.id {
                                context.push_attr("data-organization-id", id);
                            }
                            if let Some(ror) = &organization.ror {
                                context.push_attr("data-ror", ror);
                            }

                            context
                                .enter_elem_attrs("sup", [("class", "author-affiliation-label")])
                                .push_text(&number)
                                .exit_elem();

                            let address = organization
                                .options
                                .address
                                .as_ref()
                                .map(address_parts)
                                .unwrap_or_default();
                            let name = organization
                                .name
                                .as_ref()
                                .or(organization.options.legal_name.as_ref());
                            if let Some(name) = name {
                                context
                                    .enter_elem_attrs(
                                        "span",
                                        [("class", "author-affiliation-name")],
                                    )
                                    .push_text(name)
                                    .exit_elem();
                            } else if address.is_empty()
                                && let Some(ror) = &organization.ror
                            {
                                context
                                    .enter_elem_attrs(
                                        "span",
                                        [("class", "author-affiliation-name")],
                                    )
                                    .push_text(ror.trim_start_matches("https://ror.org/"))
                                    .exit_elem();
                            }

                            if !address.is_empty() {
                                if name.is_some() {
                                    context
                                        .enter_elem_attrs(
                                            "span",
                                            [("class", "author-affiliation-separator")],
                                        )
                                        .push_text(", ")
                                        .exit_elem();
                                }
                                context.enter_elem_attrs(
                                    "span",
                                    [("class", "author-affiliation-address")],
                                );
                                for (part_index, part) in address.iter().enumerate() {
                                    if part_index > 0 {
                                        context
                                            .enter_elem_attrs(
                                                "span",
                                                [("class", "author-address-part-separator")],
                                            )
                                            .push_text(", ")
                                            .exit_elem();
                                    }
                                    context
                                        .enter_elem_attrs(
                                            "span",
                                            [("class", "author-address-part")],
                                        )
                                        .push_text(part)
                                        .exit_elem();
                                }
                                context.exit_elem();
                            }

                            if let Some(ror) = &organization.ror {
                                let url =
                                    if ror.starts_with("http://") || ror.starts_with("https://") {
                                        ror.clone()
                                    } else {
                                        format!("https://ror.org/{ror}")
                                    };
                                context
                                    .enter_elem_attrs("a", [("class", "author-affiliation-ror")])
                                    .push_attr("href", &url)
                                    .push_text("ROR")
                                    .exit_elem();
                            }

                            context.exit_elem();
                        }
                    });
                }

                if !contacts.is_empty() {
                    context.push_slot_fn("section", "contacts", |context| {
                        context.push_attr("class", "author-contacts");
                        contacts_to_dom(&contacts, "Contact:", context)
                    });
                }

                if !correspondence.is_empty() {
                    context.push_slot_fn("section", "correspondence", |context| {
                        context.push_attr("class", "author-correspondence");
                        contacts_to_dom(&correspondence, "*Correspondence:", context)
                    });
                }
            }
        } else {
            // If this article is not the root (e.g. an article output from a
            // search query) then represent as a reference
            let reference = Reference::from(self);
            context.push_slot_fn("div", "reference", |context| reference.to_dom(context));
        }

        if let Some(r#abstract) = &self.r#abstract {
            // For consistency with sections in the `content` render as a
            // <stencila-section> with a heading if necessary
            context.push_slot_fn("section", "abstract", |context| {
                context
                    .enter_node(NodeType::Section, NodeId::new(b"sec", b"abstract"))
                    .push_slot_fn("section", "content", |context| {
                        // Add an abstract heading if one does not yet exist
                        if !r#abstract.iter().any(|block| match block {
                            Block::Heading(Heading { content, .. }) => {
                                content.iter().any(|inline| match inline {
                                    Inline::Text(Text { value, .. }) => {
                                        value.to_lowercase() == "abstract"
                                    }
                                    _ => false,
                                })
                            }
                            _ => false,
                        }) {
                            h1([t("Abstract")]).to_dom(context);
                        }

                        r#abstract.to_dom(context)
                    })
                    .exit_node();
            });
        }

        if context.is_root()
            && let Some(keywords) = &self.options.keywords
            && !keywords.is_empty()
        {
            context.push_slot_fn("section", "keywords", |context| {
                context
                    .enter_elem_attrs("span", [("class", "article-keywords-label")])
                    .push_text("Keywords:")
                    .exit_elem();
                context.push_text(" ");
                context.enter_elem_attrs("span", [("class", "article-keywords")]);
                for (index, keyword) in keywords.iter().enumerate() {
                    if index > 0 {
                        context
                            .enter_elem_attrs("span", [("class", "article-keyword-separator")])
                            .push_text(", ")
                            .exit_elem();
                    }
                    context
                        .enter_elem_attrs("span", [("class", "article-keyword")])
                        .push_text(keyword)
                        .exit_elem();
                }
                context.exit_elem();
            });
        }

        if !self.content.is_empty() {
            context.push_slot_fn("section", "content", |context| self.content.to_dom(context));
        }

        if let Some(references) = &self.references
            && !references.is_empty()
        {
            // For consistency with sections in the `content` render as a
            // <stencila-section> with a heading
            context.push_slot_fn("section", "references", |context| {
                context
                    .enter_node(NodeType::Section, NodeId::new(b"sec", b"references"))
                    .push_slot_fn("section", "content", |context| {
                        h1([t("References")]).to_dom(context);
                        references.to_dom(context)
                    })
                    .exit_node();
            });
        }

        context.exit_node();
    }
}

impl LatexCodec for Article {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        // Scan any raw latex blocks to check if command is already present
        let has = |what: &str| -> bool {
            self.content.iter().any(|block| match block {
                Block::RawBlock(RawBlock {
                    format, content, ..
                }) => matches!(Format::from_name(format), Format::Latex) && content.contains(what),
                _ => false,
            })
        };

        const ENVIRON: &str = "document";
        if context.standalone {
            if !has("\\documentclass") {
                context.str("\\documentclass{article}\n\n");
            }

            if !has("\\title")
                && let Some(title) = &self.title
            {
                context.property_fn(NodeProperty::Title, |context| {
                    context.command_begin("title");
                    title.to_latex(context);
                    context.command_end().newline();
                });
                context.newline();
            }

            if !has("\\author")
                && let Some(authors) = &self.authors
            {
                context.property_fn(NodeProperty::Authors, |context| {
                    for author in authors {
                        context
                            .command_begin("author")
                            .escaped_str(&author.name())
                            .command_end()
                            .newline();
                    }
                });
                context.newline();
            }

            if !has("\\date")
                && let Some(date) = self
                    .date_published
                    .as_ref()
                    .or(self.options.date_modified.as_ref())
                    .or(self.options.date_accepted.as_ref())
                    .or(self.options.date_received.as_ref())
                    .or(self.options.date_created.as_ref())
            {
                context.property_fn(NodeProperty::Date, |context| {
                    context
                        .command_begin("date")
                        .escaped_str(&date.value)
                        .command_end()
                        .newline();
                });
                context.newline();
            }

            if !has("\\keywords")
                && let Some(keywords) = &self.options.keywords
            {
                context.property_fn(NodeProperty::Keywords, |context| {
                    context
                        .command_begin("keywords")
                        .escaped_str(&keywords.join(", "))
                        .command_end()
                        .newline();
                });
                context.newline();
            }

            if !has("\\begin{document}") {
                context.environ_begin(ENVIRON).str("\n\n");
            }

            if self.title.is_some() && !has("\\maketitle") {
                context.str("\\maketitle\n\n");
            }
        }

        context.property_fn(NodeProperty::Content, |context| {
            self.content.to_latex(context)
        });

        if context.standalone && !has("\\end{document}") {
            context.ensure_blankline().environ_end(ENVIRON).char('\n');
        }

        context.exit_node_final();
    }
}

impl MarkdownCodec for Article {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        let yaml = if self.title.is_some() || self.r#abstract.is_some() {
            // If there are frontmatter related properties on the article, create, or update existing, YAML frontmatter
            // See `rust/codec-markdown/src/decode/frontmatter.rs` for how frontmatter is decoded.
            // This should be compatible with that if possible
            let mut yaml: Option<serde_yaml::Mapping> = if let Some(yaml) = &self.frontmatter {
                // Parse existing frontmatter so it can be updated
                serde_yaml::from_str(yaml).ok()
            } else {
                // Start with empty frontmatter
                Some(serde_yaml::Mapping::new())
            };

            if let Some(yaml) = &mut yaml {
                // Update the title and abstract of the work, which may include executable code expressions,
                // which if render: true will be encoded differently from in any original template.

                // Track whether title or abstract changed to avoid unnecessary reformatting
                let mut title_changed = false;
                let mut abstract_changed = false;

                if let Some(title) = &self.title {
                    let new_markdown =
                        to_markdown_with(title, context.format.clone(), MarkdownEncodeMode::Render);

                    // Only update if the content has actually changed
                    let should_update =
                        if let Some(existing) = yaml.get("title").and_then(|v| v.as_str()) {
                            existing.trim() != new_markdown.trim()
                        } else {
                            true // No existing title, definitely update
                        };

                    if should_update {
                        yaml.insert("title".into(), new_markdown.into());
                        title_changed = true;
                    }
                }

                if let Some(date_published) = &self.date_published {
                    yaml.insert("date".into(), date_published.value.clone().into());
                }

                if let Some(r#abstract) = &self.r#abstract {
                    let new_markdown = to_markdown_with(
                        r#abstract,
                        context.format.clone(),
                        MarkdownEncodeMode::Render,
                    );

                    // Only update if the content has actually changed
                    let should_update =
                        if let Some(existing) = yaml.get("abstract").and_then(|v| v.as_str()) {
                            existing.trim() != new_markdown.trim()
                        } else {
                            true // No existing abstract, definitely update
                        };

                    if should_update {
                        yaml.insert("abstract".into(), new_markdown.into());
                        abstract_changed = true;
                    }
                }

                // If neither title nor abstract changed, return original frontmatter to avoid reformatting
                if !title_changed && !abstract_changed && self.frontmatter.is_some() {
                    self.frontmatter.clone().unwrap_or_default()
                } else {
                    serde_yaml::to_string(&yaml)
                        .unwrap_or_default()
                        .trim()
                        .to_string()
                }
            } else {
                // Should only end up here if there is already frontmatter but
                // that errored when parsed. So just return it  verbatim,
                // without trying to update it.
                self.frontmatter.clone().unwrap_or_default()
            }
        } else if let Some(yaml) = &self.frontmatter {
            // Front matter is already defined for the article so just use that
            yaml.clone()
        } else {
            String::new()
        };

        if !yaml.is_empty() {
            context.push_prop_fn(NodeProperty::Frontmatter, |context| {
                context.push_str("---\n");
                context.push_str(&yaml);
                context.push_str("\n---\n\n");
            });
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        if matches!(context.mode, MarkdownEncodeMode::Render)
            && let Some(references) = &self.references
            && !references.is_empty()
        {
            context.push_prop_fn(NodeProperty::References, |context| {
                context.push_str("# References\n\n");
                references.to_markdown(context);
            });
        }

        context.append_footnotes();

        if matches!(context.format, Format::Smd)
            && !matches!(context.mode, MarkdownEncodeMode::Clean)
        {
            if let Some(comments) = &self.options.comments {
                for comment in comments {
                    comment.to_markdown(context);
                }
            }
            context.append_comments();
        }

        context.exit_node_final();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Author, DateTime, OrganizationOptions, Person, PersonOptions, PostalAddress,
        PostalAddressOptions,
    };
    use stencila_codec_dom_trait::to_dom;

    #[test]
    fn test_document_variables_empty() {
        let article = Article::default();
        let vars = article.document_variables();
        assert!(vars.is_empty());
    }

    #[test]
    fn test_document_variables_title() {
        let article = Article {
            title: Some(vec![t("Test Article Title")]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-title"),
            Some(&"Test Article Title".to_string())
        );
    }

    #[test]
    fn test_document_variables_title_truncation() {
        let long_title = "a".repeat(150);
        let article = Article {
            title: Some(vec![t(&long_title)]),
            ..Default::default()
        };
        let vars = article.document_variables();
        let title = vars.get("document-title").expect("should have title");
        // Ellipsis is 3 bytes in UTF-8, so 120 chars + 3 byte ellipsis = 123
        assert_eq!(title.len(), 123);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn test_document_variables_single_author() {
        let article = Article {
            authors: Some(vec![Author::Person(Person {
                given_names: Some(vec!["Jane".to_string()]),
                family_names: Some(vec!["Doe".to_string()]),
                ..Default::default()
            })]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(vars.get("document-authors"), Some(&"Doe".to_string()));
    }

    #[test]
    fn test_document_variables_two_authors() {
        let article = Article {
            authors: Some(vec![
                Author::Person(Person {
                    given_names: Some(vec!["Jane".to_string()]),
                    family_names: Some(vec!["Doe".to_string()]),
                    ..Default::default()
                }),
                Author::Person(Person {
                    given_names: Some(vec!["John".to_string()]),
                    family_names: Some(vec!["Smith".to_string()]),
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-authors"),
            Some(&"Doe & Smith".to_string())
        );
    }

    #[test]
    fn test_document_variables_three_authors() {
        let article = Article {
            authors: Some(vec![
                Author::Person(Person {
                    given_names: Some(vec!["Jane".to_string()]),
                    family_names: Some(vec!["Doe".to_string()]),
                    ..Default::default()
                }),
                Author::Person(Person {
                    given_names: Some(vec!["John".to_string()]),
                    family_names: Some(vec!["Smith".to_string()]),
                    ..Default::default()
                }),
                Author::Person(Person {
                    given_names: Some(vec!["Bob".to_string()]),
                    family_names: Some(vec!["Jones".to_string()]),
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-authors"),
            Some(&"Doe et al.".to_string())
        );
    }

    #[test]
    fn test_document_variables_date() {
        let article = Article {
            date_published: Some(DateTime {
                value: "2025-01-15T00:00:00Z".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(vars.get("document-date"), Some(&"2025-01-15".to_string()));
    }

    #[test]
    fn test_document_variables_doi() {
        let article = Article {
            doi: Some("10.1234/test.2025".to_string()),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-doi"),
            Some(&"DOI: 10.1234/test.2025".to_string())
        );
        assert_eq!(
            vars.get("document-doi-url"),
            Some(&"https://doi.org/10.1234/test.2025".to_string())
        );
    }

    #[test]
    fn test_document_variables_doi_url_normalization() {
        for doi in [
            "https://doi.org/10.1234/test",
            "http://doi.org/10.1234/test",
            "https://dx.doi.org/10.1234/test",
            "DOI: 10.1234/test",
        ] {
            let article = Article {
                doi: Some(doi.to_string()),
                ..Default::default()
            };

            assert_eq!(
                article.document_variables().get("document-doi-url"),
                Some(&"https://doi.org/10.1234/test".to_string())
            );
        }
    }

    #[test]
    fn test_document_variables_all_fields() {
        let article = Article {
            title: Some(vec![t("Complete Test")]),
            authors: Some(vec![Author::Person(Person {
                given_names: Some(vec!["Jane".to_string()]),
                family_names: Some(vec!["Doe".to_string()]),
                ..Default::default()
            })]),
            date_published: Some(DateTime {
                value: "2025-01-15T00:00:00Z".to_string(),
                ..Default::default()
            }),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(vars.len(), 5);
        assert!(vars.contains_key("document-title"));
        assert!(vars.contains_key("document-authors"));
        assert!(vars.contains_key("document-date"));
        assert!(vars.contains_key("document-doi"));
        assert_eq!(
            vars.get("document-doi-url"),
            Some(&"https://doi.org/10.1234/test".to_string())
        );
    }

    #[test]
    fn article_dom_preserves_structured_affiliations() {
        let article = Article {
            authors: Some(vec![Author::Person(Person {
                given_names: Some(vec!["Jane".to_string()]),
                family_names: Some(vec!["Doe".to_string()]),
                affiliations: Some(vec![Organization {
                    name: Some("Example University".to_string()),
                    ror: Some("012345678".to_string()),
                    options: Box::new(OrganizationOptions {
                        address: Some(PostalAddressOrString::PostalAddress(PostalAddress {
                            street_address: Some("1 Research Way".to_string()),
                            address_locality: Some("Wellington".to_string()),
                            address_country: Some("New Zealand".to_string()),
                            options: Box::new(PostalAddressOptions::default()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            })]),
            ..Default::default()
        };

        let dom = to_dom(&article);
        assert!(dom.contains("slot=affiliations"));
        assert!(dom.contains("data-ror=012345678"));
        assert!(dom.contains("Example University"));
        assert!(dom.contains("1 Research Way"));
        assert!(dom.contains("Wellington"));
        assert!(dom.contains("New Zealand"));
        assert!(dom.contains("href=#article-affiliation-1"));
    }

    #[test]
    fn article_dom_does_not_merge_affiliations_by_display_text() {
        let affiliations = ["012345678", "876543210"]
            .into_iter()
            .map(|ror| Organization {
                name: Some("Shared University Name".to_string()),
                ror: Some(ror.to_string()),
                ..Default::default()
            })
            .collect();
        let article = Article {
            authors: Some(vec![Author::Person(Person {
                affiliations: Some(affiliations),
                ..Default::default()
            })]),
            ..Default::default()
        };

        let dom = to_dom(&article);
        assert_eq!(dom.matches("class=author-affiliation id=").count(), 2);
        assert!(dom.contains("data-affiliation-index=1"));
        assert!(dom.contains("data-affiliation-index=2"));
    }

    #[test]
    fn article_dom_distinguishes_contact_from_correspondence() {
        let article = Article {
            authors: Some(vec![
                Author::Person(Person {
                    given_names: Some(vec!["Contact".to_string()]),
                    family_names: Some(vec!["Author".to_string()]),
                    options: Box::new(PersonOptions {
                        emails: Some(vec!["contact@example.org".to_string()]),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                Author::Person(Person {
                    given_names: Some(vec!["Corresponding".to_string()]),
                    family_names: Some(vec!["Author".to_string()]),
                    options: Box::new(PersonOptions {
                        address: Some(PostalAddressOrString::PostalAddress(PostalAddress {
                            emails: Some(vec!["corresponding@example.org".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        };

        let dom = to_dom(&article);
        assert!(dom.contains("slot=contacts"));
        assert!(dom.contains("Contact:"));
        assert!(dom.contains("contact@example.org"));
        assert!(dom.contains("slot=correspondence"));
        assert!(dom.contains("*Correspondence:"));
        assert!(dom.contains("corresponding@example.org"));
        assert_eq!(dom.matches("class=author-correspondence-mark").count(), 1);
    }

    #[test]
    fn article_dom_keeps_keywords_individually_addressable() {
        let article = Article {
            options: Box::new(crate::ArticleOptions {
                keywords: Some(vec!["one".to_string(), "two".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let dom = to_dom(&article);
        assert!(dom.contains("slot=keywords"));
        assert_eq!(dom.matches("class=article-keyword>").count(), 2);
        assert!(dom.contains("class=article-keyword-separator"));
    }
}
