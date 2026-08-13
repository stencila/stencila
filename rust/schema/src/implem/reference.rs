use stencila_codec_info::{lost_options, lost_options_of};

use crate::{
    Article, Author, AuthorRoleAuthor, CreativeWork, CreativeWorkType, CreativeWorkVariant, Date,
    Organization, Person, PersonOrOrganization, PostalAddressOrString, Reference, ReferenceOptions,
    prelude::*, replicate,
};

use super::article::{encode_text_element, identifiers};

pub(super) fn normalize_doi(doi: &str) -> &str {
    doi.trim()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://doi.org/")
        .trim_start_matches("doi:")
        .trim()
}

pub(super) fn normalize_orcid(orcid: &str) -> String {
    let identifier = orcid
        .trim()
        .trim_start_matches("https://orcid.org/")
        .trim_start_matches("http://orcid.org/")
        .trim_end_matches('/');
    format!("https://orcid.org/{identifier}")
}

pub(super) fn organization_name(organization: &Organization) -> Option<&str> {
    organization
        .name
        .as_deref()
        .or(organization.options.legal_name.as_deref())
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// Record a loss for every populated `Person` property that JATS contributor
/// encoding does not emit
///
/// `contrib` indicates a `<contrib>` context, where ORCID, emails, identifiers,
/// role, address and affiliations are emitted; in a `<person-group>` only the
/// name is.
pub(super) fn add_person_losses(person: &Person, contrib: bool, context: &mut JatsEncodeContext) {
    context.merge_losses(lost_options_of!(
        "Person",
        person.options,
        alternate_names,
        description,
        images,
        url,
        funders,
        honorific_prefix,
        honorific_suffix,
        member_of,
        telephone_numbers
    ));

    if !contrib {
        context.merge_losses(lost_options!(person, orcid, affiliations));
        context.merge_losses(lost_options_of!(
            "Person",
            person.options,
            emails,
            identifiers,
            job_title,
            address
        ));
    }
}

/// Record a loss for every populated `Organization` property that `<aff>` or
/// `<funding-source>` encoding does not emit
pub(super) fn add_organization_losses(
    organization: &Organization,
    context: &mut JatsEncodeContext,
) {
    context.merge_losses(lost_options_of!(
        "Organization",
        organization.options,
        alternate_names,
        description,
        images,
        url,
        brands,
        contact_points,
        departments,
        funders,
        logo,
        members,
        parent_organization
    ));
}

pub(super) fn encode_person_name(person: &Person, context: &mut JatsEncodeContext) -> bool {
    let family = person
        .family_names
        .as_ref()
        .map(|names| names.join(" "))
        .filter(|name| !name.trim().is_empty());
    let given = person
        .given_names
        .as_ref()
        .map(|names| names.join(" "))
        .filter(|name| !name.trim().is_empty());

    if family.is_some() || given.is_some() {
        context.enter_elem("name");
        if let Some(family) = family {
            context.enter_elem("surname").push_text(family).exit_elem();
        }
        if let Some(given) = given {
            context
                .enter_elem("given-names")
                .push_text(given)
                .exit_elem();
        }
        context.exit_elem();
        true
    } else if let Some(name) = person
        .options
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        context
            .enter_elem("string-name")
            .push_text(name)
            .exit_elem();
        true
    } else {
        false
    }
}

fn encode_reference_author(author: &Author, context: &mut JatsEncodeContext) -> bool {
    match author {
        Author::Person(person) => {
            add_person_losses(person, false, context);
            encode_person_name(person, context)
        }
        Author::Organization(organization) => {
            if let Some(name) = organization_name(organization) {
                context.enter_elem("collab").push_text(name).exit_elem();
                true
            } else {
                false
            }
        }
        Author::SoftwareApplication(software) => {
            let name = software.name.trim();
            if name.is_empty() {
                false
            } else {
                context.enter_elem("collab").push_text(name).exit_elem();
                true
            }
        }
        Author::AuthorRole(role) => match &role.author {
            AuthorRoleAuthor::Person(person) => {
                add_person_losses(person, false, context);
                encode_person_name(person, context)
            }
            AuthorRoleAuthor::Organization(organization) => {
                if let Some(name) = organization_name(organization) {
                    context.enter_elem("collab").push_text(name).exit_elem();
                    true
                } else {
                    false
                }
            }
            AuthorRoleAuthor::SoftwareApplication(software) => {
                let name = software.name.trim();
                if name.is_empty() {
                    false
                } else {
                    context.enter_elem("collab").push_text(name).exit_elem();
                    true
                }
            }
            AuthorRoleAuthor::Thing(..) => false,
        },
    }
}

fn publication_type(work_type: Option<CreativeWorkType>) -> Option<&'static str> {
    use CreativeWorkType::*;
    match work_type {
        Some(Article | Blog | Review) => Some("journal"),
        Some(Book | Chapter) => Some("book"),
        Some(Dataset) => Some("data"),
        Some(Report) => Some("report"),
        Some(Thesis) => Some("thesis"),
        Some(SoftwareApplication | SoftwareRepository | SoftwareSourceCode) => Some("software"),
        Some(WebPage) => Some("website"),
        _ => None,
    }
}

/// Whether the title of a reference with no container is the title of a whole
/// work, which JATS names with `<source>`, rather than of a part of one, which
/// it names with `<article-title>`
///
/// An untyped reference is treated as a whole work because a part is almost
/// always cited with the work that contains it.
fn is_whole_work(work_type: Option<CreativeWorkType>) -> bool {
    use CreativeWorkType::*;
    matches!(
        work_type,
        None | Some(
            Book | Collection
                | Dataset
                | Legislation
                | Periodical
                | Report
                | SoftwareApplication
                | SoftwareRepository
                | SoftwareSourceCode
                | Thesis
        )
    )
}

/// The year of a citation
///
/// A bibliographic year can carry a disambiguating suffix, as in "2017a", that
/// is not part of a date but which distinguishes the reference from others by
/// the same authors in the same year, so is kept.
fn citation_year(date: &Date) -> Option<String> {
    let value = date.value.trim();
    let year = value.split('-').next()?;
    let digits = year
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    if digits.len() != 4 {
        return None;
    }

    // Only a year on its own can carry a suffix
    Some(if value == year {
        year.to_string()
    } else {
        digits
    })
}

fn has_structured_content(reference: &Reference) -> bool {
    reference
        .authors
        .as_ref()
        .is_some_and(|items| !items.is_empty())
        || reference
            .options
            .editors
            .as_ref()
            .is_some_and(|items| !items.is_empty())
        || reference
            .title
            .as_ref()
            .is_some_and(|items| !items.is_empty())
        || reference.date.is_some()
        || reference.is_part_of.is_some()
        || reference.options.volume_number.is_some()
        || reference.options.issue_number.is_some()
        || reference.options.page_start.is_some()
        || reference.options.page_end.is_some()
        || reference.options.pagination.is_some()
        || reference.options.publisher.is_some()
        || reference.options.version.is_some()
        || reference
            .options
            .identifiers
            .as_ref()
            .is_some_and(|items| !items.is_empty())
        || reference
            .doi
            .as_deref()
            .is_some_and(|doi| !doi.trim().is_empty())
        || reference
            .url
            .as_deref()
            .is_some_and(|url| !url.trim().is_empty())
}

/// The name and location of a publisher, as JATS spells them
fn publisher_parts(publisher: &PersonOrOrganization) -> (Option<String>, Option<String>) {
    match publisher {
        PersonOrOrganization::Person(person) => {
            let name = person.name();
            ((!name.trim().is_empty()).then_some(name), None)
        }
        PersonOrOrganization::Organization(organization) => {
            let location = match organization.options.address.as_ref() {
                Some(PostalAddressOrString::String(address)) => {
                    let address = address.trim();
                    (!address.is_empty()).then(|| address.to_string())
                }
                Some(PostalAddressOrString::PostalAddress(address)) => address
                    .address_locality
                    .as_deref()
                    .map(str::trim)
                    .filter(|locality| !locality.is_empty())
                    .map(String::from),
                None => None,
            };
            (
                organization_name(organization).map(str::to_string),
                location,
            )
        }
    }
}

/// Emit the `<person-group>` for the authors or editors of a citation
///
/// Returns whether any name was emitted so that an empty group, which is
/// invalid JATS, can be dropped and reported instead.
fn encode_person_group<T>(
    group_type: &str,
    people: &[T],
    context: &mut JatsEncodeContext,
    mut encode: impl FnMut(&T, &mut JatsEncodeContext) -> bool,
) -> bool {
    context
        .enter_elem("person-group")
        .push_attr("person-group-type", group_type);
    let mut encoded = false;
    for person in people {
        encoded |= encode(person, context);
    }
    if encoded {
        context.exit_elem();
    } else {
        context.exit_elem_omit_empty();
    }
    encoded
}

impl Reference {
    pub(super) fn to_jats_with_id(&self, id: &str, context: &mut JatsEncodeContext) {
        context.enter_elem("ref").push_attr("id", id);

        if self.work_type.is_some() && publication_type(self.work_type).is_none() {
            context.add_loss("Reference.workType");
        }

        let text = self
            .options
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty());
        let content = self
            .options
            .content
            .as_ref()
            .filter(|content| !content.is_empty());

        // A citation is either a structured one, a rendering of the reference as
        // text, or, when the reference has both, the two as alternatives to each
        // other. The alternatives keep the raw text of a citation whose fields
        // could not all be decoded without duplicating those that could.
        let structured = has_structured_content(self);
        let raw = text.is_some() || content.is_some();
        if structured && raw {
            context.enter_elem("citation-alternatives");
        }

        if structured {
            context.enter_elem("element-citation");
            if let Some(publication_type) = publication_type(self.work_type) {
                context.push_attr("publication-type", publication_type);
            }

            if let Some(authors) = &self.authors
                && !encode_person_group("author", authors, context, encode_reference_author)
            {
                context.add_loss("Reference.authors");
            }

            let container = self.is_part_of.as_deref();
            let container_editors =
                container.and_then(|container| container.options.editors.as_deref());
            let editors_belong_to_container =
                container.is_some() && matches!(self.work_type, Some(CreativeWorkType::Chapter));
            let editors = if editors_belong_to_container {
                if self.options.editors.is_some() {
                    context.add_loss("Reference.editors");
                }
                container_editors.or(self.options.editors.as_deref())
            } else {
                if container_editors.is_some() {
                    context.add_loss("Reference.isPartOf.editors");
                }
                self.options.editors.as_deref().or(container_editors)
            };
            if let Some(editors) = editors
                && !encode_person_group("editor", editors, context, |editor, context| {
                    add_person_losses(editor, false, context);
                    encode_person_name(editor, context)
                })
            {
                context.add_loss(if editors_belong_to_container {
                    "Reference.isPartOf.editors"
                } else {
                    "Reference.editors"
                });
            }

            if let Some(title) = &self.title
                && !title.is_empty()
            {
                // JATS names the title of a work that is not part of another
                // one, such as a whole book, with <source>
                let element = if matches!(self.work_type, Some(CreativeWorkType::Chapter)) {
                    "chapter-title"
                } else if container.is_none() && is_whole_work(self.work_type) {
                    "source"
                } else {
                    "article-title"
                };
                context.enter_elem(element);
                title.to_jats(context);
                context.exit_elem();
            }

            if let Some(container) = container {
                let title = container.title.as_ref().filter(|title| !title.is_empty());
                if let Some(title) = title {
                    context.enter_elem("source");
                    title.to_jats(context);
                    context.exit_elem();
                }

                // Report the container properties that are not emitted here or
                // with the reference's own, against the container, so that they
                // are not confused with the same property on the reference
                for (name, populated) in [
                    ("title", title.is_none()),
                    ("doi", container.doi.is_some()),
                    ("url", container.url.is_some()),
                    ("authors", container.authors.is_some()),
                    ("date", container.date.is_some()),
                    ("pageStart", container.options.page_start.is_some()),
                    ("pageEnd", container.options.page_end.is_some()),
                    ("pagination", container.options.pagination.is_some()),
                    ("identifiers", container.options.identifiers.is_some()),
                ] {
                    if populated {
                        context.add_loss(format!("Reference.isPartOf.{name}"));
                    }
                }
            }

            if let Some(version) = &self.options.version {
                encode_text_element(context, "edition", &version.to_text());
            }

            // A flat JATS citation has one publisher, volume and issue. When
            // there is a container, decoding necessarily places them there, so
            // prefer that level and report any reference-level values that can
            // not be reconstructed.
            let container_publisher =
                container.and_then(|container| container.options.publisher.as_ref());
            if container.is_some() && self.options.publisher.is_some() {
                context.add_loss("Reference.publisher");
            }
            let publisher = if container.is_some() {
                container_publisher.or(self.options.publisher.as_ref())
            } else {
                self.options.publisher.as_ref()
            };
            if let Some(publisher) = publisher {
                let (name, location) = publisher_parts(publisher);
                if let Some(location) = &location {
                    encode_text_element(context, "publisher-loc", location);
                }
                if let Some(name) = &name {
                    encode_text_element(context, "publisher-name", name);
                } else if location.is_none() {
                    context.add_loss(if container.is_some() {
                        "Reference.isPartOf.publisher"
                    } else {
                        "Reference.publisher"
                    });
                }
            }

            if let Some(year) = self.date.as_ref().and_then(citation_year) {
                context.enter_elem("year").push_text(year).exit_elem();
            }

            // The volume and issue of a serial are of the container, when there
            // is one, but a citation of a whole serial has them on the
            // reference itself; see `assemble_reference` in the JATS decoder
            let mut citation_option = |name: &str, get: fn(&ReferenceOptions) -> Option<String>| {
                if container.is_some() && get(&self.options).is_some() {
                    context.add_loss(format!("Reference.{name}"));
                }
                container
                    .and_then(|container| get(&container.options))
                    .or_else(|| get(&self.options))
            };
            for (name, value) in [
                (
                    "volume",
                    citation_option("volumeNumber", |options| {
                        options.volume_number.as_ref().map(TextCodec::to_text)
                    }),
                ),
                (
                    "issue",
                    citation_option("issueNumber", |options| {
                        options.issue_number.as_ref().map(TextCodec::to_text)
                    }),
                ),
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
                if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
                    context.enter_elem(name).push_text(value).exit_elem();
                }
            }

            // An electronic location identifier stands in for a page range so
            // is emitted with the pagination rather than the identifiers
            let identifiers = identifiers(
                self.options.identifiers.as_deref(),
                "Reference.identifiers",
                context,
            );
            let (elocation_ids, pub_ids): (Vec<_>, Vec<_>) = identifiers
                .iter()
                .partition(|identifier| identifier.property_id == Some("elocation-id"));
            for identifier in &elocation_ids {
                encode_text_element(context, "elocation-id", &identifier.value);
            }

            if let Some(doi) = self
                .doi
                .as_deref()
                .map(normalize_doi)
                .filter(|doi| !doi.is_empty())
            {
                context
                    .enter_elem("pub-id")
                    .push_attr("pub-id-type", "doi")
                    .push_text(doi)
                    .exit_elem();
            }
            for identifier in &pub_ids {
                context.enter_elem("pub-id");
                if let Some(property_id) = identifier.property_id {
                    context.push_attr("pub-id-type", property_id);
                }
                if let Some(name) = identifier.name {
                    context.push_attr("specific-use", name);
                }
                context.push_text(&identifier.value).exit_elem();
            }

            if let Some(url) = self
                .url
                .as_deref()
                .map(str::trim)
                .filter(|url| !url.is_empty())
            {
                context
                    .enter_elem("ext-link")
                    .push_attr("ext-link-type", "uri")
                    .push_attr("xlink:href", url)
                    .push_text(url)
                    .exit_elem();
            }

            context.exit_elem_omit_empty();
        }

        if raw {
            context.enter_elem("mixed-citation");
            if let Some(publication_type) = publication_type(self.work_type) {
                context.push_attr("publication-type", publication_type);
            }
            if let Some(text) = text {
                context.push_text(text);
            } else if let Some(content) = content {
                content.to_jats(context);
            }
            context.exit_elem_omit_empty();
        } else if !structured {
            context.add_loss("Reference");
        }

        if structured && raw {
            context.exit_elem_omit_empty();
        }

        context.exit_elem_omit_empty();
    }
}

impl JatsCodec for Reference {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        let id = context.register_reference_id(self.id.as_deref(), "ref1");
        self.to_jats_with_id(&id, context);
    }
}

fn date_from_date_time(date_time: &crate::DateTime) -> Option<Date> {
    date_time
        .value
        .split('T')
        .next()
        .map(|date| Date::new(date.into()))
}

impl From<&Node> for Reference {
    fn from(node: &Node) -> Self {
        match node {
            Node::Article(article) => Reference::from(article),
            _ => Reference::default(),
        }
    }
}

impl From<&CreativeWorkVariant> for Reference {
    fn from(work: &CreativeWorkVariant) -> Self {
        match work {
            CreativeWorkVariant::Article(article) => Reference::from(article),
            _ => Reference {
                work_type: Some(work.work_type()),
                doi: work.doi(),
                title: work.title(),
                ..Default::default()
            },
        }
    }
}

impl From<&CreativeWork> for Reference {
    fn from(work: &CreativeWork) -> Self {
        Self {
            work_type: work.work_type,
            doi: work.doi.clone(),
            authors: work
                .options
                .authors
                .as_ref()
                .and_then(|authors| replicate(authors).ok()),
            date: work
                .options
                .date_published
                .as_ref()
                .or(work.options.date_modified.as_ref())
                .and_then(date_from_date_time),
            title: work
                .options
                .title
                .as_ref()
                .and_then(|title| replicate(title).ok()),
            is_part_of: work
                .options
                .is_part_of
                .as_ref()
                .map(|is_part_of| Box::new(Reference::from(is_part_of))),
            ..Default::default()
        }
    }
}

impl From<&Article> for Reference {
    fn from(article: &Article) -> Self {
        Self {
            work_type: Some(CreativeWorkType::Article),
            doi: article.doi.clone(),
            authors: article
                .authors
                .as_ref()
                .and_then(|authors| replicate(authors).ok()),
            date: article
                .date_published
                .as_ref()
                .or(article.options.date_modified.as_ref())
                .or(article.options.date_accepted.as_ref())
                .or(article.options.date_received.as_ref())
                .or(article.options.date_created.as_ref())
                .and_then(date_from_date_time),
            title: article.title(),
            is_part_of: article.is_part_of().map(Box::new),
            options: Box::new(ReferenceOptions {
                page_start: article.options.page_start.clone(),
                page_end: article.options.page_end.clone(),
                pagination: article.options.pagination.clone(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl MarkdownCodec for Reference {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(content) = &self.options.content {
            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    content.to_markdown(context)
                })
                .newline()
                .exit_node()
                .newline();

            return;
        }

        let mut content = false;

        if let Some(authors) = &self.authors {
            context.push_prop_fn(NodeProperty::Authors, |context| {
                for (index, author) in authors.iter().enumerate() {
                    if index > 0 {
                        context.push_str(", ");
                    }
                    context.push_str(&author.name());
                }
            });
            content = true;
        };

        if let Some(year) = self.date.as_ref().and_then(|date| date.year()) {
            if content {
                context.push_str(" ");
            }
            context
                .push_str("(")
                .push_prop_str(NodeProperty::Date, &year.to_string())
                .push_str(")");
            content = true;
        }

        if let Some(title) = &self.title {
            if content {
                context.push_str(" ");
            }
            context.push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context));
            if !context.content.ends_with('.') {
                context.push_str(".");
            }
            content = true;
        }

        if let Some(doi) = &self.doi {
            if content {
                context.push_str(" ");
            }
            context
                .push_str("https://doi.org/")
                .push_prop_str(NodeProperty::Doi, doi)
                // Trailing space prevents syntect highlighting when output on the CLI from bleeding to next
                // reference, but should ideally not need to be here
                .push_str(" ");
        }

        context.newline().exit_node().newline();
    }
}
