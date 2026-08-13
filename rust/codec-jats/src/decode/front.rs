use std::collections::BTreeMap;

use roxmltree::Node;
use stencila_codec_text_trait::to_text;

use stencila_codec::{
    Losses,
    stencila_schema::{
        Array, Article, ArticleOptions, Author, Block, CreativeWorkVariant,
        CreativeWorkVariantOrString, DateTime, Heading, IntegerOrString, Object, Organization,
        OrganizationOptions, Periodical, Person, PersonOptions, PersonOrOrganization,
        PostalAddressOrString, Primitive, PropertyValue, PropertyValueOptions,
        PropertyValueOrString, PublicationIssue, PublicationVolume, Section, SectionType,
        StringOrNumber, ThingVariant,
    },
};

use super::{
    body::{decode_blocks, decode_inlines},
    utilities::{XLINK, extend_path, record_attrs_lost, record_node_lost, split_given_names},
};

/// Decode the `<front>` of an `<article>`
///
/// Recursively descends into the frontmatter, setting or adding to, properties of the
/// Stencila [`Article`]. An easier approach could be to use XPath as we did in Encoda
/// (https://github.com/stencila/encoda/blob/7dd7b143d0edcafa67cab96bf21dc3c077613fcc/src/codecs/jats/index.ts#L377)
/// However, the approach used here has the advantage of allowing us to enumerate tags
/// and attributes that are not handled (via `losses`).
pub(super) fn decode_front(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "journal-meta" => decode_journal_meta(&child_path, &child, article, losses),
            "article-meta" => decode_article_meta(&child_path, &child, article, losses),
            "notes" => decode_notes(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Get the trimmed text of an element, if it has any
fn non_empty_text(node: &Node) -> Option<String> {
    let text = node
        .children()
        .filter(Node::is_text)
        .filter_map(|child| child.text())
        .collect::<String>();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Remove any resolver prefix from a DOI
fn strip_doi_prefix(doi: &str) -> String {
    doi.trim()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://doi.org/")
        .trim_start_matches("https://dx.doi.org/")
        .trim_start_matches("http://dx.doi.org/")
        .to_string()
}

/// Get the normalized publication format of an element
///
/// JATS spells the same distinction as either `pub-type` (`ppub`, `epub`) or
/// `publication-format` (`print`, `electronic`). Both are normalized to the
/// `pub-type` form because that is what is emitted again.
fn publication_format(node: &Node) -> Option<String> {
    let format = node
        .attribute("pub-type")
        .or_else(|| node.attribute("publication-format"))?;

    Some(
        match format {
            "electronic" => "epub",
            "print" => "ppub",
            format => format,
        }
        .to_string(),
    )
}

/// Add a typed identifier to a list of identifiers
///
/// An identifier without a type has no property to attach the type to, so is
/// added as a plain string.
fn push_identifier(
    identifiers: &mut Option<Vec<PropertyValueOrString>>,
    property_id: Option<String>,
    name: Option<String>,
    value: &str,
) {
    let item = match property_id {
        Some(property_id) => PropertyValueOrString::PropertyValue(PropertyValue {
            property_id: Some(property_id),
            value: Primitive::String(value.to_string()),
            options: Box::new(PropertyValueOptions {
                name,
                ..Default::default()
            }),
            ..Default::default()
        }),
        None => PropertyValueOrString::String(value.to_string()),
    };

    identifiers.get_or_insert_default().push(item);
}

/// Decode a `<journal-meta>` tag to properties on an [`Article`]
///
/// All of the journal's own metadata is collected into a single [`Periodical`]
/// so that identifiers and ISSNs are not overwritten by whichever element
/// happens to come last.
fn decode_journal_meta(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let mut periodical = Periodical::default();
    let mut decoded = false;

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "journal-id" => {
                decoded |= decode_journal_id(&child_path, &child, &mut periodical, losses)
            }
            "issn" | "issn-l" => {
                decoded |= decode_issn(&child_path, &child, &mut periodical, losses)
            }
            "journal-title-group" => {
                decoded |= decode_journal_title_group(&child_path, &child, &mut periodical, losses)
            }
            "journal-title" => {
                decoded |= decode_journal_title(&child_path, &child, &mut periodical, losses)
            }
            "abbrev-journal-title" => {
                decoded |= decode_abbrev_journal_title(&child_path, &child, &mut periodical, losses)
            }
            "publisher" => decode_publisher(&child_path, &child, article, losses),
            "custom-meta-group" => decode_custom_meta_group(&child_path, &child, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }

    if decoded {
        article.options.is_part_of = Some(CreativeWorkVariant::Periodical(periodical));
    }
}

/// Decode a `<journal-title-group>` element
fn decode_journal_title_group(
    path: &str,
    node: &Node,
    periodical: &mut Periodical,
    losses: &mut Losses,
) -> bool {
    record_attrs_lost(path, node, [], losses);

    let mut decoded = false;
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "journal-title" => {
                decoded |= decode_journal_title(&child_path, &child, periodical, losses)
            }
            "abbrev-journal-title" => {
                decoded |= decode_abbrev_journal_title(&child_path, &child, periodical, losses)
            }
            _ => record_node_lost(path, &child, losses),
        };
    }

    decoded
}

/// Decode a `<journal-title>` element
fn decode_journal_title(
    path: &str,
    node: &Node,
    periodical: &mut Periodical,
    losses: &mut Losses,
) -> bool {
    record_attrs_lost(path, node, [], losses);

    let Some(name) = non_empty_text(node) else {
        return false;
    };

    periodical.name = Some(name);

    true
}

/// Decode an `<abbrev-journal-title>` element
///
/// An abbreviated title is an alternate name for the periodical. It is dropped
/// when it is the same as the primary title (as it is for preprint servers).
fn decode_abbrev_journal_title(
    path: &str,
    node: &Node,
    periodical: &mut Periodical,
    losses: &mut Losses,
) -> bool {
    record_attrs_lost(path, node, ["abbrev-type"], losses);

    let Some(name) = non_empty_text(node) else {
        return false;
    };

    if periodical.name.as_ref() == Some(&name) {
        return false;
    }

    let alternate_names = periodical.options.alternate_names.get_or_insert_default();
    if !alternate_names.contains(&name) {
        alternate_names.push(name);
    }

    true
}

/// Decode a `<journal-id>` element
fn decode_journal_id(
    path: &str,
    node: &Node,
    periodical: &mut Periodical,
    losses: &mut Losses,
) -> bool {
    let id_type = node.attribute("journal-id-type").map(String::from);

    record_attrs_lost(path, node, ["journal-id-type"], losses);

    let Some(value) = non_empty_text(node) else {
        return false;
    };

    if id_type.as_deref().map(str::to_lowercase).as_deref() == Some("doi") {
        periodical.doi = Some(strip_doi_prefix(&value));
    } else {
        push_identifier(
            &mut periodical.options.identifiers,
            id_type,
            None,
            value.as_str(),
        );
    }

    true
}

/// Decode an `<issn>` or `<issn-l>` element
///
/// The values go in `Periodical.issns`, which does not distinguish print from
/// electronic. So that the distinction survives, an ISSN with a publication
/// format also gets a typed identifier, and a linking ISSN is only an
/// identifier.
fn decode_issn(path: &str, node: &Node, periodical: &mut Periodical, losses: &mut Losses) -> bool {
    let format = publication_format(node);

    record_attrs_lost(path, node, ["pub-type", "publication-format"], losses);

    let Some(value) = non_empty_text(node) else {
        return false;
    };

    if node.has_tag_name("issn-l") {
        push_identifier(
            &mut periodical.options.identifiers,
            Some("issn-l".to_string()),
            None,
            &value,
        );
        return true;
    }

    let issns = periodical.options.issns.get_or_insert_default();
    if !issns.contains(&value) {
        issns.push(value.clone());
    }

    if let Some(format) = format {
        push_identifier(
            &mut periodical.options.identifiers,
            Some(["issn-", &format].concat()),
            None,
            &value,
        );
    }

    true
}

/// Decode a `<publisher>` element
fn decode_publisher(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let name = node
        .children()
        .find(|child| child.tag_name().name() == "publisher-name")
        .and_then(|child| child.text().map(String::from));

    let address = node
        .children()
        .find(|child| child.tag_name().name() == "publisher-loc")
        .and_then(|child| {
            child
                .text()
                .map(|loc| PostalAddressOrString::String(loc.into()))
        });

    article.options.publisher = Some(PersonOrOrganization::Organization(Organization {
        name,
        options: Box::new(OrganizationOptions {
            address,
            ..Default::default()
        }),
        ..Default::default()
    }));
}

/// Publication metadata from `<article-meta>` that has no first-class schema
/// property
///
/// Retained as structured values in `Article.extra` so that it survives a round
/// trip. See `Article::to_jats` for how each is emitted again.
#[derive(Default)]
struct ArticleMetaExtra {
    /// Every `<pub-date>` as a `(pub-type, ISO 8601 date)` pair
    pub_dates: Vec<(Option<String>, String)>,

    /// Every `<history>` `<date>` as a `(date-type, ISO 8601 date)` pair
    history_dates: Vec<(Option<String>, String)>,

    /// The events of a `<pub-history>`
    pub_history: Vec<Object>,

    /// Portable `<self-uri>` links
    resources: Vec<Object>,

    /// The `<copyright-statement>`, `<copyright-year>` and `<copyright-holder>`
    copyright: Vec<(&'static str, String)>,

    /// The prose of a `<license>`, which can state restrictions that the
    /// license URL alone does not convey
    license_text: Option<String>,
}

impl ArticleMetaExtra {
    /// Set the article's dates and add the retained metadata to `Article.extra`
    fn apply(self, article: &mut Article) {
        if let Some((.., date)) = self
            .pub_dates
            .iter()
            .enumerate()
            .max_by_key(|(index, (pub_type, date))| {
                (
                    pub_date_rank(pub_type.as_deref()),
                    date.matches('-').count(),
                    // `max_by_key` returns the last maximum, so invert the
                    // index to keep the first of equally good dates
                    usize::MAX - index,
                )
            })
            .map(|(.., date)| date)
        {
            article.date_published = Some(DateTime::new(date.clone()));
        }

        let mut extra = Object::new();

        // A single untyped publication date is fully represented by
        // `datePublished`, so is not repeated here
        if self.pub_dates.len() > 1
            || self
                .pub_dates
                .iter()
                .any(|(pub_type, ..)| pub_type.is_some())
        {
            extra.insert(
                "publicationDates".to_string(),
                Primitive::Array(typed_dates(&self.pub_dates, "type")),
            );
        }

        // `dateReceived` and `dateAccepted` represent the common cases; the
        // full history is only needed when it has other kinds of event
        if self.history_dates.iter().any(|(date_type, ..)| {
            !matches!(date_type.as_deref(), Some("received") | Some("accepted"))
        }) {
            extra.insert(
                "historyDates".to_string(),
                Primitive::Array(typed_dates(&self.history_dates, "type")),
            );
        }

        if !self.pub_history.is_empty() {
            extra.insert(
                "publicationHistory".to_string(),
                Primitive::Array(Array(
                    self.pub_history
                        .into_iter()
                        .map(Primitive::Object)
                        .collect(),
                )),
            );
        }

        if !self.resources.is_empty() {
            extra.insert(
                "resources".to_string(),
                Primitive::Array(Array(
                    self.resources.into_iter().map(Primitive::Object).collect(),
                )),
            );
        }

        for (name, value) in self.copyright {
            extra.insert(name.to_string(), Primitive::String(value));
        }

        if let Some(text) = self.license_text {
            extra.insert("licenseText".to_string(), Primitive::String(text));
        }

        if !extra.is_empty() {
            match &mut article.options.extra {
                Some(existing) => existing.extend(extra.0),
                None => article.options.extra = Some(extra),
            }
        }
    }
}

/// Represent typed dates as objects that can be stored in `Article.extra`
fn typed_dates(dates: &[(Option<String>, String)], type_key: &str) -> Array {
    let dates = dates
        .iter()
        .map(|(date_type, date)| {
            let mut object = Object::new();
            if let Some(date_type) = date_type {
                object.insert(
                    type_key.to_string(),
                    Primitive::String(date_type.to_string()),
                );
            }
            object.insert("date".to_string(), Primitive::String(date.to_string()));
            Primitive::Object(object)
        })
        .collect::<Vec<Primitive>>();

    Array(dates)
}

/// Rank a `<pub-date>` by how well it represents the date the article was published
///
/// Electronic and unqualified publication dates are preferred over print and
/// collection dates, which are often only a year or a month, and over
/// source-system dates such as when a preprint server ingested the article.
fn pub_date_rank(pub_type: Option<&str>) -> u8 {
    match pub_type.unwrap_or("pub") {
        "epub" | "pub" | "publication" | "electronic" => 5,
        "ppub" | "print" => 4,
        "epub-original" | "epreprint" => 3,
        "collection" | "collected" => 2,
        _ => 1,
    }
}

/// Decode an `<article-meta>` (or a sub-article's `<front-stub>`) tag to
/// properties on an [`Article`]
pub(super) fn decode_article_meta(
    path: &str,
    node: &Node,
    article: &mut Article,
    losses: &mut Losses,
) {
    let correspondence_emails = correspondence_emails(node);
    let mut extra = ArticleMetaExtra::default();

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "abstract" => decode_abstract(&child_path, &child, article, losses),
            "article-categories" => decode_article_categories(&child_path, &child, article, losses),
            "article-id" => decode_article_id(&child_path, &child, article, losses),
            "article-version" => decode_article_version(&child_path, &child, article, losses),
            "pub-date" => decode_pub_date(&child_path, &child, &mut extra, losses),
            "history" => decode_history(&child_path, &child, article, &mut extra, losses),
            "pub-history" => decode_pub_history(&child_path, &child, &mut extra, losses),
            "volume" => decode_volume(&child_path, &child, article, losses),
            "issue" => decode_issue(&child_path, &child, article, losses),
            "issue-id" => decode_issue_id(&child_path, &child, article, losses),
            "issue-title" => decode_issue_title(&child_path, &child, article, losses),
            "fpage" => decode_fpage(&child_path, &child, article, losses),
            "lpage" => decode_lpage(&child_path, &child, article, losses),
            "page-range" => decode_page_range(&child_path, &child, article, losses),
            "elocation-id" => decode_elocation_id(&child_path, &child, article, losses),
            "permissions" => decode_permissions(&child_path, &child, article, &mut extra, losses),
            "self-uri" => decode_self_uri(&child_path, &child, &mut extra, losses),
            "counts" => decode_counts(&child_path, &child, losses),
            "custom-meta-group" | "custom-meta-wrap" => {
                decode_custom_meta_group(&child_path, &child, losses)
            }
            "funding-group" => decode_funding_group(&child_path, &child, article, losses),
            "contrib-group" => {
                decode_contrib_group(&child_path, &child, &correspondence_emails, article, losses)
            }
            // Correspondence email addresses are associated with contributors
            // in the pre-pass above. Retain the loss for the surrounding notes
            // because labels, prose, and non-correspondence notes are not decoded.
            "author-notes" => record_node_lost(path, &child, losses),
            "title-group" => decode_title_group(&child_path, &child, article, losses),
            "kwd-group" => decode_kwd_group(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }

    extra.apply(article);
}

/// Collect correspondence email addresses by the id referenced from contributors.
fn correspondence_emails(node: &Node) -> BTreeMap<String, Vec<String>> {
    node.children()
        .find(|child| child.has_tag_name("author-notes"))
        .into_iter()
        .flat_map(|notes| {
            notes
                .children()
                .filter(|child| child.has_tag_name("corresp"))
        })
        .filter_map(|correspondence| {
            let id = correspondence.attribute("id")?.to_string();
            let emails = correspondence
                .descendants()
                .filter(|child| child.has_tag_name("email"))
                .filter_map(|child| child.text().map(str::to_string))
                .collect::<Vec<_>>();
            (!emails.is_empty()).then_some((id, emails))
        })
        .collect()
}

/// Decode an `<abstract>` element
///
/// Articles may have more than one abstract, typically an untyped one plus a
/// graphical, plain language or translated abstract. The untyped abstract is the
/// article's `abstract`; the others are kept in `parts` as nested works that
/// carry only an `abstract`, along with their `abstract-type` as a genre.
fn decode_abstract(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let abstract_type = node.attribute("abstract-type").map(String::from);

    record_attrs_lost(path, node, ["abstract-type", "id"], losses);

    // Use depth = 1 so that headings within abstract are at least level 2
    let content: Vec<Block> = decode_blocks(path, node.children(), losses, 1)
        .into_iter()
        .filter(|block| match block {
            Block::Heading(Heading { content, .. }) => {
                to_text(content).to_lowercase() != "abstract"
            }
            _ => true,
        })
        .collect();

    if abstract_type.is_none() && article.r#abstract.is_none() {
        article.r#abstract = Some(content);
        return;
    }

    let part = CreativeWorkVariant::Article(Article {
        id: node.attribute("id").map(String::from),
        r#abstract: Some(content),
        options: Box::new(ArticleOptions {
            genre: abstract_type.map(|abstract_type| vec![abstract_type]),
            ..Default::default()
        }),
        ..Default::default()
    });

    match &mut article.options.parts {
        Some(parts) => parts.push(part),
        None => article.options.parts = Some(vec![part]),
    }
}

/// Decode an `<article-categories>` element
fn decode_article_categories(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "subj-group" => decode_subj_group(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode a `<subj-group>` element by adding its subjects to the article's
/// `about` property
///
/// Subject taxonomies are frequently hierarchical, with each level narrowing
/// the one above it. Each path through the hierarchy becomes one
/// [`PropertyValue`], whose value is the whole path rather than only the
/// broadest or narrowest subject in it.
fn decode_subj_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let subject_type = node.attribute("subj-group-type").map(String::from);

    record_attrs_lost(path, node, ["subj-group-type"], losses);

    let mut subjects = Vec::new();
    collect_subjects(path, node, &[], &mut subjects, losses);

    let about = article.options.about.get_or_insert_default();
    for mut subject in subjects {
        let value = if subject.len() == 1 {
            Primitive::String(subject.swap_remove(0))
        } else {
            Primitive::Array(Array(subject.into_iter().map(Primitive::String).collect()))
        };

        about.push(ThingVariant::PropertyValue(PropertyValue {
            property_id: subject_type.clone(),
            value,
            ..Default::default()
        }));
    }
}

/// Collect each path from the outermost to an innermost `<subject>` of a
/// `<subj-group>`
fn collect_subjects(
    path: &str,
    node: &Node,
    prefix: &[String],
    subjects: &mut Vec<Vec<String>>,
    losses: &mut Losses,
) {
    let mut here = Vec::new();
    let mut groups = Vec::new();

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "subject" => {
                record_attrs_lost(&child_path, &child, [], losses);
                if let Some(subject) = non_empty_text_deep(&child) {
                    here.push(subject);
                }
            }
            "subj-group" => {
                record_attrs_lost(&child_path, &child, [], losses);
                groups.push((child_path, child));
            }
            _ => record_node_lost(path, &child, losses),
        };
    }

    if groups.is_empty() {
        for subject in here {
            let mut subject_path = prefix.to_vec();
            subject_path.push(subject);
            subjects.push(subject_path);
        }
        return;
    }

    let mut prefix = prefix.to_vec();
    prefix.extend(here);
    for (child_path, group) in groups {
        collect_subjects(&child_path, &group, &prefix, subjects, losses);
    }
}

/// Decode an `<article-id>` element
///
/// A publisher may emit several identifiers of the same type, distinguished
/// only by a qualifier such as `specific-use` or a source-system subtype. The
/// qualifier is kept as the identifier's name so that they remain distinct.
fn decode_article_id(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let property_id = node.attribute("pub-id-type").map(String::from);
    let name = node
        .attribute("specific-use")
        .or_else(|| node.attribute("sub-type"))
        .map(String::from);

    record_attrs_lost(
        path,
        node,
        ["pub-id-type", "specific-use", "sub-type"],
        losses,
    );

    let Some(id) = non_empty_text(node) else {
        return;
    };

    if property_id
        .as_ref()
        .map(|pid| pid.to_lowercase())
        .as_deref()
        == Some("doi")
        && article.doi.is_none()
    {
        article.doi = Some(strip_doi_prefix(&id));
        return;
    }

    push_identifier(&mut article.options.identifiers, property_id, name, &id);
}

/// Decode an `<elocation-id>` element
///
/// An electronic location identifier stands in for a page range but is an
/// identifier, not a page, so is kept as one rather than as `pageStart`.
fn decode_elocation_id(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let Some(value) = non_empty_text(node) else {
        return;
    };

    push_identifier(
        &mut article.options.identifiers,
        Some("elocation-id".to_string()),
        None,
        &value,
    );
}

/// Decode a `<title-group>` element
fn decode_title_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        if child.tag_name().name() == "article-title" {
            article.title = Some(decode_inlines(path, child.children(), losses));
        }
    }
}

/// Decode an `<article-version>` element
fn decode_article_version(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    if let Some(version) = node.text() {
        article.options.version = Some(StringOrNumber::String(version.into()))
    };
}

/// Decode a `<pub-date>` element
///
/// An article commonly has several publication dates. They are all retained so
/// that the most specific one can be chosen once they have all been seen,
/// rather than the last one overwriting the others.
fn decode_pub_date(path: &str, node: &Node, extra: &mut ArticleMetaExtra, losses: &mut Losses) {
    let pub_type = node
        .attribute("pub-type")
        .map(String::from)
        .or_else(|| node.attribute("date-type").map(String::from))
        .or_else(|| publication_format(node));

    record_attrs_lost(
        path,
        node,
        [
            "pub-type",
            "date-type",
            "publication-format",
            "iso-8601-date",
        ],
        losses,
    );

    if let Some(date) = date_element_to_date(node) {
        extra.pub_dates.push((pub_type, date));
    }
}

/// Decode a `<history>` element
fn decode_history(
    path: &str,
    node: &Node,
    article: &mut Article,
    extra: &mut ArticleMetaExtra,
    losses: &mut Losses,
) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "date" => decode_date(&child_path, &child, article, extra, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode a `<date>` element
fn decode_date(
    path: &str,
    node: &Node,
    article: &mut Article,
    extra: &mut ArticleMetaExtra,
    losses: &mut Losses,
) {
    let date_type = node.attribute("date-type").map(String::from);

    record_attrs_lost(path, node, ["date-type", "iso-8601-date"], losses);

    let Some(date) = date_element_to_date(node) else {
        return;
    };

    match date_type.as_deref() {
        Some("accepted") => article.options.date_accepted = Some(DateTime::new(date.clone())),
        Some("received") => article.options.date_received = Some(DateTime::new(date.clone())),
        _ => {}
    }

    extra.history_dates.push((date_type, date));
}

/// Decode a `<pub-history>` element
///
/// Publication history events, such as the posting of a preprint or of a
/// reviewed preprint, are a different kind of milestone from the received and
/// accepted dates of a `<history>`, so are kept apart from them.
fn decode_pub_history(path: &str, node: &Node, extra: &mut ArticleMetaExtra, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        if tag != "event" {
            record_node_lost(path, &child, losses);
            continue;
        }

        let event_type = child.attribute("event-type").map(String::from);
        record_attrs_lost(&child_path, &child, ["event-type"], losses);

        let mut date_type = None;
        let mut date = None;
        let mut description = None;
        let mut url = None;
        for grandchild in child.children() {
            let tag = grandchild.tag_name().name();
            let grandchild_path = extend_path(&child_path, tag);
            match tag {
                "date" => {
                    date_type = grandchild.attribute("date-type").map(String::from);
                    record_attrs_lost(
                        &grandchild_path,
                        &grandchild,
                        ["date-type", "iso-8601-date"],
                        losses,
                    );
                    date = date_element_to_date(&grandchild);
                }
                "event-desc" => {
                    record_attrs_lost(&grandchild_path, &grandchild, [], losses);
                    description = non_empty_text_deep(&grandchild);
                }
                "self-uri" => {
                    record_attrs_lost(&grandchild_path, &grandchild, ["href"], losses);
                    url = grandchild.attribute((XLINK, "href")).map(String::from);
                }
                _ => record_node_lost(&child_path, &grandchild, losses),
            };
        }

        let Some(date) = date else {
            record_node_lost(path, &child, losses);
            continue;
        };

        let mut event = Object::new();
        for (key, value) in [
            ("eventType", event_type),
            ("dateType", date_type),
            ("description", description),
        ] {
            if let Some(value) = value {
                event.insert(key.to_string(), Primitive::String(value));
            }
        }
        event.insert("date".to_string(), Primitive::String(date));
        if let Some(url) = url {
            event.insert("url".to_string(), Primitive::String(url));
        }

        extra.pub_history.push(event);
    }
}

/// Decode a `<pub-date>` or `<date>` element to an ISO 8601 date
///
/// Prefers an `iso-8601-date` attribute, if valid, over the date parts because
/// the parts are frequently not zero padded, and sometimes absent.
fn date_element_to_date(node: &Node) -> Option<String> {
    if let Some(date) = node.attribute("iso-8601-date")
        && is_iso_date(date)
    {
        return Some(date.to_string());
    }

    let mut day = None;
    let mut month = None;
    let mut year = None;

    for child in node.children() {
        if let Some(value) = child.text().map(str::trim) {
            match child.tag_name().name() {
                "day" => day = Some(value),
                "month" => month = Some(value),
                "year" => year = Some(value),
                _ => {}
            }
        }
    }

    let year = year?;
    if year.len() != 4 || !year.chars().all(|char| char.is_ascii_digit()) {
        return None;
    }

    let mut date = year.to_string();

    let Some(month) = month.and_then(pad_date_part) else {
        return Some(date);
    };
    date.push('-');
    date.push_str(&month);

    if let Some(day) = day.and_then(pad_date_part) {
        date.push('-');
        date.push_str(&day);
    }

    Some(date)
}

/// Zero pad a month or day so that the emitted date is a valid ISO 8601 date
fn pad_date_part(part: &str) -> Option<String> {
    let part = part.trim_start_matches('0');
    let number: u32 = part.parse().ok()?;
    (number > 0).then(|| format!("{number:02}"))
}

/// Whether a string is an ISO 8601 date, to at least year precision
fn is_iso_date(date: &str) -> bool {
    let mut parts = date.split('-');
    let Some(year) = parts.next() else {
        return false;
    };
    if year.len() != 4 || !year.chars().all(|char| char.is_ascii_digit()) {
        return false;
    }
    parts.all(|part| part.len() == 2 && part.chars().all(|char| char.is_ascii_digit()))
}

/// Decode a `<volume>` element
fn decode_volume(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let Some(volume_number) = node.text() else {
        return;
    };

    let volume = PublicationVolume {
        volume_number: Some(IntegerOrString::from(volume_number)),
        ..Default::default()
    };

    let work = match &article.options.is_part_of {
        Some(CreativeWorkVariant::Periodical(periodical)) => {
            // Make this volume part of the existing periodical
            CreativeWorkVariant::PublicationVolume(PublicationVolume {
                is_part_of: Some(Box::new(CreativeWorkVariant::Periodical(
                    periodical.clone(),
                ))),
                ..volume
            })
        }
        Some(CreativeWorkVariant::PublicationIssue(issue)) => {
            // Make the existing issue part of this volume
            CreativeWorkVariant::PublicationIssue(PublicationIssue {
                is_part_of: Some(Box::new(CreativeWorkVariant::PublicationVolume(volume))),
                ..issue.clone()
            })
        }
        _ => {
            // Use this volume
            CreativeWorkVariant::PublicationVolume(volume)
        }
    };

    article.options.is_part_of = Some(work);
}

/// Decode an `<issue>` element
fn decode_issue(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let Some(issue_number) = node.text() else {
        return;
    };

    // Use the article's existing issue, if any, so that an issue identifier or
    // title decoded before the issue number is not discarded
    article_issue(article).issue_number = Some(IntegerOrString::from(issue_number));
}

/// Decode an `<fpage>` element
fn decode_fpage(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    article.options.page_start = node.text().map(IntegerOrString::from)
}

/// Decode an `<lpage>` element
fn decode_lpage(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    article.options.page_end = node.text().map(IntegerOrString::from)
}

/// Decode a `<page-range>` element
fn decode_page_range(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    article.options.pagination = non_empty_text(node)
}

/// Get the `PublicationIssue` that the article is part of, adding one if necessary
///
/// The issue that `<issue-id>` and `<issue-title>` describe is the one that the
/// article is in, which may already be nested within a volume and periodical.
fn article_issue(article: &mut Article) -> &mut PublicationIssue {
    let is_part_of = &mut article.options.is_part_of;

    if !matches!(is_part_of, Some(CreativeWorkVariant::PublicationIssue(..))) {
        let issue = PublicationIssue {
            is_part_of: is_part_of.take().map(Box::new),
            ..Default::default()
        };
        *is_part_of = Some(CreativeWorkVariant::PublicationIssue(issue));
    }

    let Some(CreativeWorkVariant::PublicationIssue(issue)) = is_part_of else {
        unreachable!("issue was just created")
    };

    issue
}

/// Decode an `<issue-id>` element
fn decode_issue_id(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let property_id = node.attribute("pub-id-type").map(String::from);

    record_attrs_lost(path, node, ["pub-id-type"], losses);

    let Some(value) = non_empty_text(node) else {
        return;
    };

    let issue = article_issue(article);
    if property_id.as_deref().map(str::to_lowercase).as_deref() == Some("doi") {
        issue.doi = Some(strip_doi_prefix(&value));
    } else {
        push_identifier(&mut issue.options.identifiers, property_id, None, &value);
    }
}

/// Decode an `<issue-title>` element
fn decode_issue_title(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let title = decode_inlines(path, node.children(), losses);
    if title.is_empty() {
        return;
    }

    article_issue(article).options.title = Some(title);
}

/// Decode a `<permissions>` element
///
/// The license URL is the machine readable part and goes in `Article.licenses`.
/// Copyright details and any license prose are retained as structured metadata
/// because the schema has no properties for them.
fn decode_permissions(
    path: &str,
    node: &Node,
    article: &mut Article,
    extra: &mut ArticleMetaExtra,
    losses: &mut Losses,
) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "copyright-statement" | "copyright-year" | "copyright-holder" => {
                record_attrs_lost(&child_path, &child, ["content-type"], losses);
                if let Some(value) = non_empty_text_deep(&child) {
                    let name = match tag {
                        "copyright-statement" => "copyrightStatement",
                        "copyright-year" => "copyrightYear",
                        _ => "copyrightHolder",
                    };
                    // Some publishers repeat these for the issue as well as the
                    // article; the first, which is the article's, is kept
                    if !extra.copyright.iter().any(|(key, ..)| key == &name) {
                        extra.copyright.push((name, value));
                    }
                }
            }
            "license" => decode_license(&child_path, &child, article, extra, losses),
            // Marks the article as free to read, which the license URL conveys
            "free_to_read" => record_attrs_lost(&child_path, &child, [], losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode a `<license>` element
fn decode_license(
    path: &str,
    node: &Node,
    article: &mut Article,
    extra: &mut ArticleMetaExtra,
    losses: &mut Losses,
) {
    record_attrs_lost(path, node, ["href", "license-type"], losses);

    let mut url = node.attribute((XLINK, "href")).map(String::from);
    let mut text = String::new();

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "license_ref" => {
                record_attrs_lost(
                    &child_path,
                    &child,
                    ["specific-use", "content-type"],
                    losses,
                );
                if url.is_none() {
                    url = non_empty_text(&child);
                }
            }
            "license-p" | "p" => {
                record_attrs_lost(&child_path, &child, [], losses);
                if url.is_none() {
                    url = child
                        .descendants()
                        .find(|descendant| descendant.has_tag_name("ext-link"))
                        .and_then(|link| link.attribute((XLINK, "href")))
                        .map(String::from);
                }
                // Only the prose is kept, because the license statement is
                // metadata rather than article content
                if let Some(paragraph) = non_empty_text_deep(&child) {
                    if !text.is_empty() {
                        text.push_str("\n\n");
                    }
                    text.push_str(&paragraph);
                }
            }
            _ => record_node_lost(path, &child, losses),
        };
    }

    if let Some(url) = url {
        let licenses = article.options.licenses.get_or_insert_default();
        let license = CreativeWorkVariantOrString::String(url);
        if !licenses.contains(&license) {
            licenses.push(license);
        }
    }

    if !text.is_empty() && extra.license_text.is_none() {
        extra.license_text = Some(text);
    }
}

/// Decode a `<self-uri>` element
///
/// Links to the article's own PDF, source and other representations are useful,
/// but publishing systems also emit absolute paths within their own file
/// systems. Those are not portable so are filtered out, with a loss.
fn decode_self_uri(path: &str, node: &Node, extra: &mut ArticleMetaExtra, losses: &mut Losses) {
    let content_type = node.attribute("content-type").map(String::from);
    let role = node.attribute((XLINK, "role")).map(String::from);
    let href = node.attribute((XLINK, "href")).map(str::trim);

    record_attrs_lost(path, node, ["content-type", "href", "role"], losses);

    let Some(href) = href.filter(|href| !href.is_empty()) else {
        losses.add(path);
        return;
    };

    if href.starts_with("file:/") || href.starts_with('/') {
        losses.add(extend_path(path, "@href"));
        return;
    }

    let mut resource = Object::new();
    if let Some(content_type) = content_type {
        resource.insert("type".to_string(), Primitive::String(content_type));
    }
    if let Some(role) = role {
        resource.insert("role".to_string(), Primitive::String(role));
    }
    resource.insert("url".to_string(), Primitive::String(href.to_string()));

    if !extra.resources.contains(&resource) {
        extra.resources.push(resource);
    }
}

/// Decode a `<counts>` element
///
/// Figure, table, page and word counts are all derivable from, or specific to,
/// the publisher's own rendering of the article. Each is reported as a distinct
/// loss rather than dropping the group as a whole.
fn decode_counts(path: &str, node: &Node, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        record_node_lost(path, &child, losses);
    }
}

/// Decode a `<custom-meta-group>` element
///
/// Custom metadata is by definition outside the JATS vocabulary, and in the
/// examples seen is source-system state. Each entry is reported by name so that
/// the loss says which metadata was dropped.
fn decode_custom_meta_group(path: &str, node: &Node, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        if !child.has_tag_name("custom-meta") {
            record_node_lost(path, &child, losses);
            continue;
        }

        let name = child
            .children()
            .find(|grandchild| grandchild.has_tag_name("meta-name"))
            .and_then(|grandchild| non_empty_text(&grandchild));

        match name {
            Some(name) => losses.add(format!("{path}/custom-meta[meta-name='{name}']")),
            None => losses.add(extend_path(path, "custom-meta")),
        }
    }
}

/// Get the trimmed text of an element and all of its descendants
fn non_empty_text_deep(node: &Node) -> Option<String> {
    let text = node
        .descendants()
        .filter(Node::is_text)
        .filter_map(|descendant| descendant.text())
        .collect::<String>();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

/// Decode a `<funding-group>` element
fn decode_funding_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let funders = node
        .children()
        .filter(|child| child.tag_name().name() == "award-group")
        .flat_map(|award_group| {
            let path = &extend_path(path, "award-group");
            award_group
                .children()
                .filter(|child| child.tag_name().name() == "funding-source")
                .filter_map(|child| decode_funding_source(path, &child, losses))
                .collect::<Vec<PersonOrOrganization>>()
        })
        .collect();

    article.options.funders = Some(funders);
}

/// Decode a `<funding-source>` element
fn decode_funding_source(
    path: &str,
    node: &Node,
    losses: &mut Losses,
) -> Option<PersonOrOrganization> {
    record_attrs_lost(path, node, [], losses);

    let mut name = None;
    let mut url = None;

    for child in node.descendants() {
        let tag = child.tag_name().name();
        if tag == "institution" {
            name = child.text().map(String::from);
        } else if tag == "institution-id" {
            url = child.text().map(String::from);
        }
    }

    if name.is_none()
        && let Some(text) = node.text()
    {
        let text = text.trim();
        if !text.is_empty() {
            name = Some(text.to_string());
        }
    }

    if name.is_none() && url.is_none() {
        return None;
    }

    Some(PersonOrOrganization::Organization(Organization {
        name,
        options: Box::new(OrganizationOptions {
            url,
            ..Default::default()
        }),
        ..Default::default()
    }))
}

/// Decode a `<contrib-group>` element
fn decode_contrib_group(
    path: &str,
    node: &Node,
    correspondence_emails: &BTreeMap<String, Vec<String>>,
    article: &mut Article,
    losses: &mut Losses,
) {
    record_attrs_lost(path, node, [], losses);

    let mut authors = Vec::new();
    let mut editors = Vec::new();
    for child in node
        .children()
        .filter(|child| child.tag_name().name() == "contrib")
    {
        let (contrib_type, contributor) =
            decode_contrib(path, &child, correspondence_emails, losses);
        if contrib_type.contains("author") {
            let author = match contributor {
                PersonOrOrganization::Person(person) => Author::Person(person),
                PersonOrOrganization::Organization(org) => Author::Organization(org),
            };
            authors.push(author);
        } else if contrib_type.contains("editor") {
            // Allows for variants such as "senior_editor"
            if let PersonOrOrganization::Person(person) = contributor {
                editors.push(person);
            }
        }
    }

    if !authors.is_empty() {
        match &mut article.authors {
            Some(existing) => existing.extend(authors),
            None => article.authors = Some(authors),
        }
    }

    if !editors.is_empty() {
        match &mut article.options.editors {
            Some(existing) => existing.extend(editors),
            None => article.options.editors = Some(editors),
        }
    }
}

/// Decode a `<contrib>` element
fn decode_contrib(
    path: &str,
    node: &Node,
    correspondence_emails: &BTreeMap<String, Vec<String>>,
    losses: &mut Losses,
) -> (String, PersonOrOrganization) {
    let contrib_type = node
        .attribute("contrib-type")
        .map_or_else(|| "author".to_string(), |ct| ct.to_lowercase().to_string());

    record_attrs_lost(path, node, ["contrib-type", "corresp"], losses);

    let mut family_names = Vec::new();
    let mut given_names = Vec::new();
    let mut orcid = None;
    let mut emails = Vec::new();
    let mut affiliations = Vec::new();

    for child in node.children() {
        let tag = child.tag_name().name();
        if tag == "name" {
            for grandchild in child.children() {
                let grandchild_tag = grandchild.tag_name().name();
                if grandchild_tag == "surname" {
                    if let Some(value) = grandchild.text() {
                        family_names.push(value.to_string());
                    }
                } else if grandchild_tag == "given-names"
                    && let Some(value) = grandchild.text()
                {
                    given_names.append(&mut split_given_names(value));
                }
            }
        } else if tag == "contrib-id"
            && matches!(child.attribute("contrib-id-type"), Some("orcid"))
            && orcid.is_none()
        {
            orcid = child.text().map(|orcid| {
                orcid
                    .trim_start_matches("https://orcid.org/")
                    .trim_start_matches("http://orcid.org/")
                    .to_string()
            });
        } else if tag == "object-id" && orcid.is_none() {
            if let Some(url) = child.attribute("xlink:href")
                && let Some(id) = url
                    .strip_prefix("https://orcid.org/")
                    .or_else(|| url.strip_prefix("http://orcid.org/"))
            {
                orcid = Some(id.into())
            };
        } else if tag == "email" {
            if let Some(value) = child.text() {
                emails.push(value.into());
            }
        } else if tag == "aff" {
            affiliations.push(decode_aff(&child))
        } else if tag == "xref"
            && matches!(child.attribute("ref-type"), Some("aff"))
            && let Some(id) = child.attribute("rid")
        {
            // Search up the tree for the <aff> with the id, starting at this node
            let mut ancestor = Some(*node);
            while let Some(ancestor_node) = ancestor {
                if let Some(aff) = ancestor_node
                    .children()
                    .find(|n| n.has_tag_name("aff") && n.attribute("id").unwrap_or_default() == id)
                {
                    affiliations.push(decode_aff(&aff));
                    break;
                }

                ancestor = ancestor_node.parent();
            }
        } else if tag == "xref"
            && matches!(child.attribute("ref-type"), Some("corresp"))
            && let Some(id) = child.attribute("rid")
            && let Some(correspondence) = correspondence_emails.get(id)
        {
            emails.extend(correspondence.iter().cloned());
        } else {
            record_node_lost(path, &child, losses);
        }
    }

    let family_names = (!family_names.is_empty()).then_some(family_names);
    let given_names = (!given_names.is_empty()).then_some(given_names);
    let emails = (!emails.is_empty()).then_some(emails);
    let affiliations = (!affiliations.is_empty()).then_some(affiliations);

    let contributor = PersonOrOrganization::Person(Person {
        orcid,
        family_names,
        given_names,
        affiliations,
        options: Box::new(PersonOptions {
            emails,
            ..Default::default()
        }),
        ..Default::default()
    });

    (contrib_type, contributor)
}

/// Decode an `<aff>` element
fn decode_aff(node: &Node) -> Organization {
    const TRIM_CHARS: &[char] = &[',', '.', ' ', '\n'];

    let mut ror = None;
    let mut name = Vec::new();
    let mut address = Vec::new();
    for child in node.children() {
        let tag = child.tag_name().name();
        match tag {
            "institution-id" if matches!(child.attribute("institution-id-type"), Some("ror")) => {
                ror = child.text().map(String::from);
            }

            "named-content"
                if matches!(
                    child.attribute("content-type"),
                    Some("organisation-division")
                ) =>
            {
                if let Some(text) = child.text() {
                    name.push(text.trim_matches(TRIM_CHARS))
                }
            }
            "institution" => {
                if let Some(text) = child.text() {
                    name.push(text.trim_matches(TRIM_CHARS))
                }
            }
            "institution-wrap" => {
                for grandchild in child.children() {
                    let tag_name = grandchild.tag_name().name();
                    if tag_name == "institution-id"
                        && matches!(grandchild.attribute("institution-id-type"), Some("ror"))
                    {
                        ror = grandchild.text().map(String::from);
                    } else if tag_name == "institution"
                        && let Some(text) = grandchild.text()
                    {
                        name.push(text.trim_matches(TRIM_CHARS))
                    }
                }
            }

            "addr-line" | "city" | "state" | "country" => {
                if let Some(text) = child.text() {
                    address.push(text.trim_matches(TRIM_CHARS))
                }
            }
            "named-content"
                if matches!(
                    child.attribute("content-type"),
                    Some("street" | "city" | "county-part" | "country")
                ) =>
            {
                if let Some(text) = child.text() {
                    address.push(text.trim_matches(TRIM_CHARS))
                }
            }

            _ => {
                if child.is_text()
                    && let Some(text) = child.text()
                {
                    address.push(text.trim_matches(TRIM_CHARS))
                }
            }
        };
    }
    name.retain(|name| !name.is_empty());
    address.retain(|name| !name.is_empty());

    let ror = ror.map(|ror| ror.trim_start_matches("https://ror.org/").to_string());

    let name = if name.is_empty() {
        if address.is_empty() {
            None
        } else {
            // Use address as name
            let name = address.join(", ");
            address.clear();
            Some(name)
        }
    } else {
        Some(name.join(", "))
    };

    let address = (!address.is_empty()).then(|| PostalAddressOrString::String(address.join(", ")));

    Organization {
        name,
        ror,
        options: Box::new(OrganizationOptions {
            address,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Decode a `<kwd-group>` element
fn decode_kwd_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let mut keywords = node
        .children()
        .filter(|child| child.tag_name().name() == "kwd")
        .map(|child| decode_kwd(path, &child, losses))
        .collect();

    if let Some(ref mut vector) = article.options.keywords {
        vector.append(&mut keywords);
    } else {
        article.options.keywords = Some(keywords);
    }
}

/// Decode a `<kwd>` element
fn decode_kwd(path: &str, node: &Node, losses: &mut Losses) -> String {
    record_attrs_lost(path, node, [], losses);

    let mut keyword = String::new();

    for child in node.children() {
        if node.text().is_none() {
            keyword.push_str(&decode_kwd(path, &child, losses))
        } else if let Some(text) = child.text()
            && !text.trim().is_empty()
        {
            keyword.push_str(text)
        }
    }

    keyword
}

/// Decode a `<notes>` element to blocks that will be appended to the content of
/// the article
///
/// Some JATS has `<notes>` elements that are merely wrappers around other
/// `<notes>`, or around footnote groups, and have no `notes-type` or `<title>`
/// child. Such a wrapper is transparent: its supported descendants are decoded
/// in place rather than the whole wrapper being discarded.
pub fn decode_notes(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let section_type = node
        .attribute("notes-type")
        .and_then(|value| SectionType::from_text(value).ok())
        .or_else(|| {
            node.children()
                .find(|child| child.tag_name().name() == "title")
                .and_then(|node| node.text())
                .and_then(|value| SectionType::from_text(value).ok())
        });

    record_attrs_lost(path, node, ["notes-type"], losses);

    if section_type.is_none() {
        for child in node.children() {
            let tag = child.tag_name().name();
            let child_path = extend_path(path, tag);
            match tag {
                "notes" => decode_notes(&child_path, &child, article, losses),
                _ => {
                    let mut blocks = decode_blocks(path, std::iter::once(child), losses, 1);
                    article.content.append(&mut blocks);
                }
            };
        }
    } else {
        let content = decode_blocks(path, node.children(), losses, 1);
        article.content.push(Block::Section(Section {
            section_type,
            content,
            ..Default::default()
        }));
    }
}
