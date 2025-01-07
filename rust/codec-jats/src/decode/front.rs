use roxmltree::Node;

use codec::{
    common::itertools::Itertools,
    schema::{
        Article, Author, Block, CreativeWorkType, Date, IntegerOrString, Organization,
        OrganizationOptions, Person, PersonOptions, PersonOrOrganization, Primitive, PropertyValue,
        PropertyValueOrString, PublicationVolume, Section, SectionType, StringOrNumber, ThingType,
    },
    Losses,
};

use super::{
    body::{decode_blocks, decode_inlines},
    utilities::{extend_path, record_attrs_lost, record_node_lost},
};

/// Decode the `<front>` of an `<article>`
///
/// Recursively descends into the frontmatter, setting or adding to, properties of the
/// Stencila [`Article`]. An easier approach would be to use XPath as we did in Encoda
/// (https://github.com/stencila/encoda/blob/7dd7b143d0edcafa67cab96bf21dc3c077613fcc/src/codecs/jats/index.ts#L377)
/// However, the approach used here has the advantage of allowing us to enumerate tags
/// and attributes that are not handled (via `losses`).
pub(super) fn decode_front(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "article-meta" => decode_article_meta(&child_path, &child, article, losses),
            "journal-meta" => decode_journal_meta(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
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
            "funding-group" => decode_funding_group(&child_path, &child, article, losses),
            "contrib-group" => decode_contrib_group(&child_path, &child, article, losses),
            "title-group" => decode_title_group(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode an `<abstract>` element by adding it as a [`Section`] to the start of the [`Article`]
fn decode_abstract(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let content = decode_blocks(path, node.children(), losses, 0);

    let section = Block::Section(Section {
        section_type: Some(SectionType::Abstract),
        content,
        ..Default::default()
    });

    article.content.insert(0, section);
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

    let item = ThingType::PropertyValue(PropertyValue {
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

/// Decode an `<title-group>` element
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

    let date = year.iter().chain(month.iter()).chain(day.iter()).join("-");

    article.date_published = (!date.is_empty()).then(|| Date::new(date))
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

    let date = year.iter().chain(month.iter()).chain(day.iter()).join("-");

    if date_type == Some("accepted") {
        article.date_accepted = (!date.is_empty()).then(|| Date::new(date.clone()));
    } else if date_type == Some("received") {
        article.date_received = (!date.is_empty()).then(|| Date::new(date.clone()));
    }
}

/// Decode an `<volume>` element
fn decode_volume(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    if let Ok(volume_number) = node.text().unwrap_or_default().parse() {
        if article.options.parts.is_none() {
            article.options.parts = Some(vec![CreativeWorkType::PublicationVolume(
                PublicationVolume {
                    volume_number: Some(IntegerOrString::Integer(volume_number)),
                    ..Default::default()
                },
            )]);
        }
    }
}

/// Decode an `<funding-group>` element
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
                .map(|child| decode_funding_source(path, &child, losses))
                .collect::<Vec<PersonOrOrganization>>()
        })
        .collect();

    article.options.funders = Some(funders);
}

/// Decode an `<funding-source>` element
fn decode_funding_source(path: &str, node: &Node, losses: &mut Losses) -> PersonOrOrganization {
    record_attrs_lost(path, node, [], losses);

    let mut funder = None;
    let mut url = None;

    for funding_source in node.children() {
        for child in funding_source.children() {
            let tag = child.tag_name().name();
            if tag == "institution" {
                funder = child.text().map(str::to_string);
            } else if tag == "institution-id" {
                url = child.text().map(str::to_string);
            }
        }
    }

    PersonOrOrganization::Organization(Organization {
        name: funder,
        options: Box::new(OrganizationOptions {
            url,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Decode an `<contrib-group>` element
fn decode_contrib_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let mut authors = node
        .children()
        .filter(|child| child.tag_name().name() == "contrib")
        .map(|child| decode_contrib(path, &child, losses))
        .collect();

    if let Some(ref mut vector) = article.authors {
        vector.append(&mut authors);
    } else {
        article.authors = Some(authors);
    }
}

/// Decode an `<contrib-id>, <email> and <name>` elements
fn decode_contrib(path: &str, node: &Node, losses: &mut Losses) -> Author {
    record_attrs_lost(path, node, [], losses);

    let mut family_names = Vec::new();
    let mut given_names = Vec::new();
    let mut identifiers = Vec::new();
    let mut emails = Vec::new();

    for child in node.children() {
        let tag = child.tag_name().name();
        if tag == "name" {
            for grandchild in child.children() {
                let grandchild_tag = grandchild.tag_name().name();
                if grandchild_tag == "surname" {
                    if let Some(value) = grandchild.text() {
                        family_names.push(value.to_string());
                    }
                } else if grandchild_tag == "given-names" {
                    if let Some(value) = grandchild.text() {
                        given_names.push(value.to_string());
                    }
                }
            }
        } else if tag == "contrib-id" {
            if let Some(value) = child.text() {
                identifiers.push(PropertyValueOrString::PropertyValue(PropertyValue {
                    property_id: Some("https://registry.identifiers.org/registry/orcid".into()),
                    value: Primitive::String(value.into()),
                    ..Default::default()
                }));
            }
        } else if tag == "email" {
            if let Some(value) = child.text() {
                emails.push(value.into());
            }
        } else {
            record_node_lost(path, &child, losses);
        }
    }

    let family_names = (!family_names.is_empty()).then_some(family_names);
    let given_names = (!given_names.is_empty()).then_some(given_names);
    let identifiers = (!identifiers.is_empty()).then_some(identifiers);
    let emails = (!emails.is_empty()).then_some(emails);

    Author::Person(Person {
        family_names,
        given_names,
        options: Box::new(PersonOptions {
            identifiers,
            emails,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Decode an `<journal-meta>` tag to properties on an [`Article`]
fn decode_journal_meta(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "publisher" => decode_publisher(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode an `<publisher>` element
fn decode_publisher(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let name = node
        .children()
        .find(|child| child.tag_name().name() == "publisher-name")
        .and_then(|child| child.text().map(str::to_string));

    article.options.publisher = Some(PersonOrOrganization::Organization(Organization {
        name,
        ..Default::default()
    }));
}
