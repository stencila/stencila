use roxmltree::Node;
use stencila_codec_text_trait::to_text;

use stencila_codec::{
    Losses,
    stencila_schema::{
        Article, Author, Block, CreativeWorkVariant, Date, Heading, IntegerOrString, Organization,
        OrganizationOptions, Periodical, Person, PersonOptions, PersonOrOrganization,
        PostalAddressOrString, Primitive, PropertyValue, PropertyValueOrString, PublicationIssue,
        PublicationVolume, Section, SectionType, StringOrNumber, ThingVariant,
    },
};

use super::{
    body::{decode_blocks, decode_inlines},
    utilities::{extend_path, record_attrs_lost, record_node_lost, split_given_names},
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

/// Decode a `<journal-meta>` tag to properties on an [`Article`]
fn decode_journal_meta(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "journal-title-group" => {
                decode_journal_title_group(&child_path, &child, article, losses)
            }
            "journal-title" => decode_journal_title(&child_path, &child, article, losses),
            "publisher" => decode_publisher(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode a `<journal-title-group>` element
fn decode_journal_title_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "journal-title" => decode_journal_title(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode a `<journal-title>` element
fn decode_journal_title(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let name = node.text().map(String::from);

    article.options.is_part_of = Some(CreativeWorkVariant::Periodical(Periodical {
        name,
        ..Default::default()
    }));
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

/// Decode an `<article-meta>` tag to properties on an [`Article`]
fn decode_article_meta(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "abstract" => decode_abstract(&child_path, &child, article, losses),
            "article-categories" => decode_article_categories(&child_path, &child, article, losses),
            "article-id" => decode_article_id(&child_path, &child, article, losses),
            "article-version" => decode_article_version(&child_path, &child, article, losses),
            "pub-date" => decode_pub_date(&child_path, &child, article, losses),
            "history" => decode_history(&child_path, &child, article, losses),
            "volume" => decode_volume(&child_path, &child, article, losses),
            "issue" => decode_issue(&child_path, &child, article, losses),
            "fpage" => decode_fpage(&child_path, &child, article, losses),
            "lpage" => decode_lpage(&child_path, &child, article, losses),
            "funding-group" => decode_funding_group(&child_path, &child, article, losses),
            "contrib-group" => decode_contrib_group(&child_path, &child, article, losses),
            "title-group" => decode_title_group(&child_path, &child, article, losses),
            "kwd-group" => decode_kwd_group(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode an `<abstract>` element
fn decode_abstract(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    // Some articles have multiple <abstract> elements (e.g. an additional one with abstract-type="graphical")
    // We just take the first one.
    if article.r#abstract.is_some() {
        record_node_lost(path, node, losses);
        return;
    }

    record_attrs_lost(path, node, [], losses);

    // Use depth = 1 so that headings within abstract are at least level 2
    let content = decode_blocks(path, node.children(), losses, 1)
        .into_iter()
        .filter(|block| match block {
            Block::Heading(Heading { content, .. }) => {
                to_text(content).to_lowercase() != "abstract"
            }
            _ => true,
        })
        .collect();

    article.r#abstract = Some(content);
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

/// Decode a `<subj-group>` element by adding the text content of the inner `<subject>`
/// to the article's `about` property
fn decode_subj_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let subject_type = node.attribute("subj-group-type").map(String::from);

    record_attrs_lost(path, node, ["subj-group-type"], losses);

    let mut subject = None;
    for child in node.children() {
        if child.tag_name().name() == "subject" {
            if let Some(text) = child.text() {
                subject = Some(text.to_string());
            }
        } else {
            record_node_lost(path, &child, losses);
        }
    }
    let Some(subject) = subject else { return };

    let item = ThingVariant::PropertyValue(PropertyValue {
        property_id: subject_type,
        value: Primitive::String(subject),
        ..Default::default()
    });

    if let Some(about) = &mut article.options.about {
        about.push(item);
    } else {
        article.options.about = Some(vec![item])
    }
}

/// Decode an `<article-id>` element
fn decode_article_id(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let property_id = node.attribute("pub-id-type").map(String::from);

    record_attrs_lost(path, node, ["pub-id-type"], losses);

    let Some(id) = node.text() else { return };

    if property_id
        .as_ref()
        .map(|pid| pid.to_lowercase())
        .as_deref()
        == Some("doi")
    {
        article.doi = Some(
            id.trim_start_matches("https://doi.org/")
                .trim_start_matches("https://dx.doi.org/")
                .to_string(),
        );
        return;
    }

    let item = if property_id.is_some() {
        PropertyValueOrString::PropertyValue(PropertyValue {
            property_id,
            value: Primitive::String(id.into()),
            ..Default::default()
        })
    } else {
        PropertyValueOrString::String(id.into())
    };

    if let Some(about) = &mut article.options.identifiers {
        about.push(item);
    } else {
        article.options.identifiers = Some(vec![item])
    }
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
fn decode_pub_date(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    article.date_published = date_element_to_date(node)
}

/// Decode a `<history>` element
fn decode_history(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        let tag = child.tag_name().name();

        if tag == "date" {
            decode_date(path, &child, article, losses);
        }
    }
}

/// Decode a `<date>` element
fn decode_date(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let date_type = node.attribute("date-type");
    record_attrs_lost(path, node, ["date-type"], losses);

    if date_type == Some("accepted") {
        article.options.date_accepted = date_element_to_date(node);
    } else if date_type == Some("received") {
        article.options.date_received = date_element_to_date(node);
    }
}

/// Decode a `<pub-date>` or `<date>` element to a `Date`
fn date_element_to_date(node: &Node) -> Option<Date> {
    let mut day = None;
    let mut month = None;
    let mut year = None;

    for child in node.children() {
        if let Some(value) = child.text() {
            match child.tag_name().name() {
                "day" => day = Some(value),
                "month" => month = Some(value),
                "year" => year = Some(value),
                _ => {}
            }
        }
    }

    let mut date = year.map(String::from)?;

    if let Some(month) = month {
        date.push('-');
        date.push_str(month);
    } else {
        return Some(Date::new(date));
    }

    if let Some(day) = day {
        date.push('-');
        date.push_str(day);
    }

    Some(Date::new(date))
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

    let issue = PublicationIssue {
        issue_number: Some(IntegerOrString::from(issue_number)),
        ..Default::default()
    };

    let work = match &article.options.is_part_of {
        Some(CreativeWorkVariant::Periodical(periodical)) => {
            // Make this issue part of the existing periodical
            CreativeWorkVariant::PublicationIssue(PublicationIssue {
                is_part_of: Some(Box::new(CreativeWorkVariant::Periodical(
                    periodical.clone(),
                ))),
                ..issue
            })
        }
        Some(CreativeWorkVariant::PublicationVolume(volume)) => {
            // Make this issue part of the existing volume
            CreativeWorkVariant::PublicationIssue(PublicationIssue {
                is_part_of: Some(Box::new(CreativeWorkVariant::PublicationVolume(
                    volume.clone(),
                ))),
                ..issue.clone()
            })
        }
        _ => {
            // Use this issue
            CreativeWorkVariant::PublicationIssue(issue)
        }
    };

    article.options.is_part_of = Some(work);
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
fn decode_contrib_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let mut authors = Vec::new();
    let mut editors = Vec::new();
    for child in node
        .children()
        .filter(|child| child.tag_name().name() == "contrib")
    {
        let (contrib_type, contributor) = decode_contrib(path, &child, losses);
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
fn decode_contrib(path: &str, node: &Node, losses: &mut Losses) -> (String, PersonOrOrganization) {
    let contrib_type = node
        .attribute("contrib-type")
        .map_or_else(|| "author".to_string(), |ct| ct.to_lowercase().to_string());

    record_attrs_lost(path, node, ["contrib-type"], losses);

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

/// Decode a `<notes>` element to a [`Block::Section`] that will be appended to
/// th content of the article
///
/// Some JATS has `<notes>` elements that are merely wrappers around other
/// <notes> and have no `notes-type` or `<title>` child. For these, we
/// recursively check for nested notes.
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
                _ => record_node_lost(path, &child, losses),
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
