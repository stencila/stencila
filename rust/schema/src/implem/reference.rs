use crate::{
    Article, Author, AuthorRoleAuthor, CreativeWork, CreativeWorkType, CreativeWorkVariant, Date,
    Organization, Person, PersonOrOrganization, Reference, ReferenceOptions, prelude::*, replicate,
};

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
        Author::Person(person) => encode_person_name(person, context),
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
            AuthorRoleAuthor::Person(person) => encode_person_name(person, context),
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
        || reference
            .doi
            .as_deref()
            .is_some_and(|doi| !doi.trim().is_empty())
        || reference
            .url
            .as_deref()
            .is_some_and(|url| !url.trim().is_empty())
}

impl Reference {
    pub(super) fn to_jats_with_id(&self, id: &str, context: &mut JatsEncodeContext) {
        context.enter_elem("ref").push_attr("id", id);

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

        if has_structured_content(self) {
            context.enter_elem(if text.is_some() || content.is_some() {
                "mixed-citation"
            } else {
                "element-citation"
            });
            if let Some(publication_type) = publication_type(self.work_type) {
                context.push_attr("publication-type", publication_type);
            }
            if let Some(text) = text {
                context.push_text(text);
            } else if let Some(content) = content {
                content.to_jats(context);
            }

            if let Some(authors) = &self.authors {
                context
                    .enter_elem("person-group")
                    .push_attr("person-group-type", "author");
                let mut encoded = false;
                for author in authors {
                    encoded |= encode_reference_author(author, context);
                }
                if encoded {
                    context.exit_elem();
                } else {
                    context.exit_elem_omit_empty();
                    context.add_loss("Reference.authors");
                }
            }

            if let Some(editors) = &self.options.editors {
                context
                    .enter_elem("person-group")
                    .push_attr("person-group-type", "editor");
                let mut encoded = false;
                for editor in editors {
                    encoded |= encode_person_name(editor, context);
                }
                if encoded {
                    context.exit_elem();
                } else {
                    context.exit_elem_omit_empty();
                    context.add_loss("Reference.editors");
                }
            }

            if let Some(title) = &self.title
                && !title.is_empty()
            {
                let element = if matches!(self.work_type, Some(CreativeWorkType::Chapter)) {
                    "chapter-title"
                } else {
                    "article-title"
                };
                context.enter_elem(element);
                title.to_jats(context);
                context.exit_elem();
            }

            if let Some(container) = &self.is_part_of
                && let Some(title) = &container.title
                && !title.is_empty()
            {
                context.enter_elem("source");
                title.to_jats(context);
                context.exit_elem();
            }

            if let Some(year) = self.date.as_ref().and_then(Date::year) {
                context
                    .enter_elem("year")
                    .push_text(year.to_string())
                    .exit_elem();
            }

            for (name, value) in [
                (
                    "volume",
                    self.options.volume_number.as_ref().map(TextCodec::to_text),
                ),
                (
                    "issue",
                    self.options.issue_number.as_ref().map(TextCodec::to_text),
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

            if let Some(publisher) = &self.options.publisher {
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
                    context
                        .enter_elem("publisher-name")
                        .push_text(name)
                        .exit_elem();
                } else {
                    context.add_loss("Reference.publisher");
                }
            }

            context.exit_elem_omit_empty();
        } else {
            if text.is_some() || content.is_some() {
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
            } else {
                context.add_loss("Reference");
            }
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
