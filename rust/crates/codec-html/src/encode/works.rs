//! Encode `CreativeWork` nodes to HTML

use super::{
    attr, attr_id, attr_itemprop, attr_itemtype, attr_prop, concat, elem, elem_empty, json,
    Context, ToHtml,
};
use codec_txt::ToTxt;
use html_escape::encode_safe;
use itertools::Itertools;
use std::collections::BTreeMap;
use stencila_schema::*;

impl ToHtml for CreativeWorkTypes {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        match self {
            CreativeWorkTypes::Article(node) => node.to_html(slot, context),
            CreativeWorkTypes::AudioObject(node) => node.to_html(slot, context),
            CreativeWorkTypes::Claim(node) => node.to_html(slot, context),
            CreativeWorkTypes::Collection(node) => node.to_html(slot, context),
            CreativeWorkTypes::Figure(node) => node.to_html(slot, context),
            CreativeWorkTypes::ImageObject(node) => node.to_html(slot, context),
            CreativeWorkTypes::Table(node) => node.to_html(slot, context),
            CreativeWorkTypes::VideoObject(node) => node.to_html(slot, context),
            _ => elem("div", &[attr("class", "unsupported")], &json(self)),
        }
    }
}

impl ToHtml for CreativeWorkContent {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        match self {
            CreativeWorkContent::String(node) => node.to_html(slot, context),
            CreativeWorkContent::VecNode(nodes) => nodes.to_html(slot, context),
        }
    }
}

impl ToHtml for Article {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let title = match &self.title {
            Some(title) => {
                let title = match &**title {
                    CreativeWorkTitle::String(title) => title.to_html("", context),
                    CreativeWorkTitle::VecInlineContent(title) => title.to_html("", context),
                };
                elem("h1", &[attr_prop(slot), attr_itemprop("headline")], &title)
            }
            None => "".to_string(),
        };

        // Create a map of organization name to Organization, in the order
        // they appear in affiliations.
        let orgs: BTreeMap<String, &Organization> = match &self.authors {
            Some(authors) => authors
                .iter()
                .filter_map(|author| match author {
                    CreativeWorkAuthors::Person(person) => {
                        person.affiliations.as_ref().map(|orgs| {
                            orgs.iter().filter_map(|org| {
                                org.name.as_ref().map(|name| (*name.clone(), org))
                            })
                        })
                    }
                    _ => None,
                })
                .flatten()
                .collect(),
            None => BTreeMap::new(),
        };
        let orgs = orgs.values().cloned().collect();

        let authors = match &self.authors {
            Some(authors) => {
                let authors = concat(authors, |author| match author {
                    CreativeWorkAuthors::Person(person) => {
                        author_person_to_html(person, Some(&orgs))
                    }
                    CreativeWorkAuthors::Organization(org) => author_org_to_html(org),
                });
                elem("ol", &[attr_prop("authors")], &authors)
            }
            None => "".to_string(),
        };

        let affiliations = if !orgs.is_empty() {
            elem(
                "ol",
                &[attr_prop("affiliations")],
                &concat(&orgs, |org| affiliation_org_to_html(org)),
            )
        } else {
            "".to_string()
        };

        let abstract_ = match &self.description {
            Some(desc) => {
                let meta = (**desc).to_txt();
                let content = match &**desc {
                    ThingDescription::String(string) => Paragraph {
                        content: vec![InlineContent::String(string.clone())],
                        ..Default::default()
                    }
                    .to_html("", context),
                    ThingDescription::VecInlineContent(inlines) => Paragraph {
                        content: inlines.clone(),
                        ..Default::default()
                    }
                    .to_html("", context),
                    ThingDescription::VecBlockContent(blocks) => blocks.to_html("", context),
                };
                elem(
                    "section",
                    &[attr_prop("description")],
                    &[
                        elem_empty(
                            "meta",
                            &[attr_itemprop("description"), attr("content", &meta)],
                        ),
                        content,
                    ]
                    .concat(),
                )
            }
            None => "".to_string(),
        };

        let content = elem(
            "div",
            &[attr_prop("content")],
            &self.content.to_html("", context),
        );

        elem(
            "article",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &[title, authors, affiliations, abstract_, content].concat(),
        )
    }
}

fn author_person_to_html(person: &Person, orgs: Option<&Vec<&Organization>>) -> String {
    let name_string = if person.given_names.is_some() && person.family_names.is_some() {
        [
            person
                .given_names
                .as_ref()
                .map_or("".to_string(), |vec| vec.join(" ")),
            person
                .family_names
                .as_ref()
                .map_or("".to_string(), |vec| vec.join(" ")),
        ]
        .join(" ")
    } else {
        person
            .name
            .as_ref()
            .map_or("".to_string(), |name| *name.clone())
    };
    let name_string = match name_string.is_empty() {
        true => "Anonymous".to_string(),
        false => name_string,
    };

    // If there are given and/or family names then encode name as invisible `<meta>` tag,
    // otherwise, as a visible `<span>`.
    let name = if person.given_names.is_some() && person.family_names.is_some() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            "<meta itemprop=\"name\"", &attr("content", &name_string), ">",
        ]
        .concat()
    } else {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            "<span itemprop=\"name\">", &encode_safe(&name_string), "</span>",
        ]
        .concat()
    };

    let given_names = match &person.given_names {
        Some(names) => [
            "<span data-itemprop=\"givenNames\">",
            &concat(names, |name| {
                ["<span itemprop=\"givenName\">", name, "</span>"].concat()
            }),
            "</span>",
        ]
        .concat(),
        None => "".to_string(),
    };

    let family_names = match &person.family_names {
        Some(names) => [
            "<span data-itemprop=\"familyNames\">",
            &concat(names, |name| {
                ["<span itemprop=\"familyName\">", name, "</span>"].concat()
            }),
            "</span>",
        ]
        .concat(),
        None => "".to_string(),
    };

    let emails = match &person.emails {
        Some(emails) =>
        {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            [
                "<span data-itemprop=\"emails\">",
                &concat(emails, |email| {
                    [
                        "<a itemprop=\"email\"", &attr("href", &["mailto:", email].concat()), ">",
                            email,
                        "</a>",
                    ].concat()
                }),
                "</span>",
            ]
            .concat()
        }
        None => "".to_string(),
    };

    let affiliations = if let (Some(affiliations), Some(orgs)) = (&person.affiliations, orgs) {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            "<span data-itemprop=\"affiliations\">",
            &concat(affiliations, |affiliation| {
                if let Some((index,..)) = orgs.iter().find_position(|org| {
                    org.name == affiliation.name
                }) {
                    let position = (index+1).to_string();
                    [
                        "<a itemprop=\"affiliation\"", &attr("href", &position), ">",
                            &position,
                        "</a>"
                    ].concat()
                } else {
                    "".to_string()
                }
            }),
            "</span>"
        ].concat()
    } else {
        "".to_string()
    };

    #[cfg_attr(rustfmt, rustfmt_skip)]
    [
        "<li itemprop=\"author\" itemtype=\"http://schema.org/Person\" itemscope>",
            &name,
            &given_names,
            &family_names,
            &emails,
            &affiliations,
        "</li>",
    ]
    .concat()
}

fn author_org_to_html(_org: &Organization) -> String {
    [
        "<li itemprop=\"author\" itemtype=\"http://schema.org/Organization\" itemscope>",
        // TODO
        "</li>",
    ]
    .concat()
}

fn affiliation_org_to_html(org: &Organization) -> String {
    // TODO Organization address etc
    let name = org
        .name
        .as_ref()
        .map_or("".to_string(), |boxed| *boxed.clone());
    ["<li>", &name, "</li>"].concat()
}

impl ToHtml for AudioObject {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        AudioObjectSimple {
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(slot, context)
    }
}

impl ToHtml for ImageObject {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        ImageObjectSimple {
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(slot, context)
    }
}

impl ToHtml for VideoObject {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        VideoObjectSimple {
            media_type: self.media_type.clone(),
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(slot, context)
    }
}

impl ToHtml for Collection {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let Collection { parts, .. } = self;
        CollectionSimple {
            parts: parts.clone(),
            ..Default::default()
        }
        .to_html(slot, context)
    }
}

impl ToHtml for Claim {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let Claim { content, .. } = self;
        ClaimSimple {
            content: content.clone(),
            ..Default::default()
        }
        .to_html(slot, context)
    }
}

impl ToHtml for Figure {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let Figure {
            caption, content, ..
        } = self;
        FigureSimple {
            caption: caption.clone(),
            content: content.clone(),
            ..Default::default()
        }
        .to_html(slot, context)
    }
}

impl ToHtml for Table {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let Table { caption, rows, .. } = self;
        TableSimple {
            caption: caption.clone(),
            rows: rows.clone(),
            ..Default::default()
        }
        .to_html(slot, context)
    }
}
