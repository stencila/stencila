//! Encode `InlineContent` nodes to HTML

use super::{
    attr, attr_id, attr_itemprop, attr_itemtype, attr_itemtype_str, attr_prop, concat, elem,
    elem_empty, json, Context, ToHtml,
};
use codec_txt::ToTxt;
use html_escape::encode_safe;
use std::{fs, path::PathBuf};
use stencila_schema::*;

impl ToHtml for InlineContent {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        match self {
            InlineContent::AudioObject(node) => node.to_html(slot, context),
            InlineContent::Boolean(node) => node.to_html(slot, context),
            InlineContent::Cite(node) => node.to_html(slot, context),
            InlineContent::CiteGroup(node) => node.to_html(slot, context),
            InlineContent::CodeExpression(node) => node.to_html(slot, context),
            InlineContent::CodeFragment(node) => node.to_html(slot, context),
            InlineContent::Delete(node) => node.to_html(slot, context),
            InlineContent::Emphasis(node) => node.to_html(slot, context),
            InlineContent::ImageObject(node) => node.to_html(slot, context),
            InlineContent::Integer(node) => node.to_html(slot, context),
            InlineContent::Link(node) => node.to_html(slot, context),
            InlineContent::MathFragment(node) => node.to_html(slot, context),
            InlineContent::NontextualAnnotation(node) => node.to_html(slot, context),
            InlineContent::Note(node) => node.to_html(slot, context),
            InlineContent::Null(node) => node.to_html(slot, context),
            InlineContent::Number(node) => node.to_html(slot, context),
            InlineContent::Parameter(node) => node.to_html(slot, context),
            InlineContent::Quote(node) => node.to_html(slot, context),
            InlineContent::String(node) => node.to_html(slot, context),
            InlineContent::Strong(node) => node.to_html(slot, context),
            InlineContent::Subscript(node) => node.to_html(slot, context),
            InlineContent::Superscript(node) => node.to_html(slot, context),
            InlineContent::VideoObject(node) => node.to_html(slot, context),
        }
    }
}

macro_rules! mark_to_html {
    ($type:ident, $tag:literal) => {
        impl ToHtml for $type {
            fn to_html(&self, slot: &str, context: &Context) -> String {
                elem(
                    $tag,
                    &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
                    &self.content.to_html("", context),
                )
            }
        }
    };
}

mark_to_html!(Delete, "del");
mark_to_html!(Emphasis, "em");
mark_to_html!(NontextualAnnotation, "u");
mark_to_html!(Strong, "strong");
mark_to_html!(Subscript, "sub");
mark_to_html!(Superscript, "sup");

/// Convert a file:// URL to a data:// URI
///
/// Note that this function assumes that paths are absolute.
/// File URLs are usually resolves elsewhere e.g. in the `compile` method
/// before encoding to HTML.
fn file_uri_to_data_uri(url: &str) -> String {
    let path = if let Some(path) = url.strip_prefix("file://") {
        PathBuf::from(path)
    } else {
        return url.into();
    };

    // Read the file, convert it to a dataURI, and record it as a dependency
    match fs::read(&path) {
        Ok(bytes) => {
            let mime = match mime_guess::from_path(&path).first() {
                Some(mime) => mime.to_string(),
                None => "image/png".to_string(),
            };
            let data = base64::encode(bytes);
            ["data:", &mime, ";base64,", &data].concat()
        }
        Err(error) => {
            tracing::warn!("Unable to read media file {}: {}", path.display(), error);
            url.into()
        }
    }
}

fn content_url_to_src_attr(content_url: &str, context: &Context) -> String {
    let url = match context.bundle {
        true => file_uri_to_data_uri(content_url),
        false => content_url.to_string(),
    };
    attr("src", &url)
}

impl ToHtml for AudioObjectSimple {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem(
            "audio",
            &[
                attr_prop(slot),
                attr_itemtype_str("AudioObject"),
                attr_id(&self.id),
                "controls".to_string(),
                content_url_to_src_attr(&self.content_url, context),
            ],
            "",
        )
    }
}

impl ToHtml for ImageObjectSimple {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem_empty(
            "img",
            &[
                attr_prop(slot),
                attr_itemtype_str("ImageObject"),
                attr_id(&self.id),
                content_url_to_src_attr(&self.content_url, context),
            ],
        )
    }
}

impl ToHtml for VideoObjectSimple {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let src_attr = content_url_to_src_attr(&self.content_url, context);
        let type_attr = match &self.media_type {
            Some(media_type) => attr("type", media_type),
            None => "".to_string(),
        };
        elem(
            "video",
            &[
                attr_prop(slot),
                attr_itemtype_str("VideoObject"),
                attr_id(&self.id),
                "controls".to_string(),
            ],
            &elem("source", &[src_attr, type_attr], ""),
        )
    }
}

impl ToHtml for Cite {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let content = match &self.content {
            Some(nodes) => nodes.to_html("", context),
            None => {
                // Get the list of references from the root nodes
                let references = match context.root {
                    Node::Article(article) => article.references.clone(),
                    Node::CreativeWork(work) => work.references.clone(),
                    _ => {
                        tracing::warn!("Unhandled root document type");
                        None
                    }
                };
                if let Some(references) = references {
                    // Find the reference that matches the `target`
                    let reference = references
                        .iter()
                        .enumerate()
                        .find_map(|(index, reference)| match reference {
                            // A string reference so match against its index in the references list
                            CreativeWorkReferences::String(_) => {
                                if self.target == format!("ref{}", index + 1)
                                    || self.target == format!("ref{}", index + 1)
                                {
                                    Some((index, reference))
                                } else {
                                    None
                                }
                            }
                            // A `CreativeWork` reference so match against its `id`/
                            CreativeWorkReferences::CreativeWorkTypes(work) => match work {
                                CreativeWorkTypes::Article(Article { id, .. }) => {
                                    if let Some(id) = id.as_deref() {
                                        if self.target == *id {
                                            Some((index, reference))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            },
                        });
                    // Create the content for the citation
                    match reference {
                        None => {
                            tracing::warn!("When encoding citation was unable to find reference '{}' in root document", self.target);
                            format!(r#"<span>{}</span>"#, self.target.clone())
                        }
                        Some((index, reference)) => {
                            // Always have a numeric citation
                            let mut content = format!(r#"<span>{}</span>"#, index + 1);
                            // If a `CreativeWorkType` then add authors and year
                            if let CreativeWorkReferences::CreativeWorkTypes(work) = reference {
                                let (authors, date) = match &work {
                                    CreativeWorkTypes::Article(article) => {
                                        (article.authors.clone(), article.date_published.clone())
                                    }
                                    CreativeWorkTypes::CreativeWork(work) => {
                                        (work.authors.clone(), work.date_published.clone())
                                    }
                                    _ => {
                                        tracing::warn!("Unhandled root document type");
                                        (None, None)
                                    }
                                };
                                if let Some(authors) = authors {
                                    let names: Vec<String> = authors
                                        .iter()
                                        .map(|author| match author {
                                            CreativeWorkAuthors::Person(person) => {
                                                match &person.family_names {
                                                    Some(family_names) => family_names.join(" "),
                                                    None => "Anonymous".to_string(),
                                                }
                                            }
                                            CreativeWorkAuthors::Organization(org) => {
                                                match &org.name {
                                                    Some(name) => *name.clone(),
                                                    None => "Anonymous".to_string(),
                                                }
                                            }
                                        })
                                        .collect();
                                    let names = if authors.len() == 1 {
                                        names.join("")
                                    } else if authors.len() == 2 {
                                        format!(r#"{} and {}"#, names[0], names[1])
                                    } else {
                                        format!(r#"{} et al"#, names[0])
                                    };
                                    content += &format!(r#"<span>{}</span>"#, names)
                                }
                                if let Some(date) = date {
                                    if date.value.len() >= 4 {
                                        let year = date.value[..4].to_string();
                                        content += &format!(r#"<span>{}</span>"#, year)
                                    }
                                }
                            }
                            content
                        }
                    }
                } else {
                    tracing::warn!("When encoding citation was unable to find references list in root document");
                    format!(r#"<span>{}</span>"#, self.target.clone())
                }
            }
        };
        elem(
            "cite",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &elem("a", &[attr("href", &self.target)], &content),
        )
    }
}

impl ToHtml for CiteGroup {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem(
            "span",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &concat(&self.items, |cite| cite.to_html("", context)),
        )
    }
}

impl ToHtml for CodeExpression {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let output = match &self.output {
            Some(output) => output.to_html("", context),
            None => "".to_string(),
        };
        elem(
            "stencila-code-expression",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("programming-language", &self.programming_language),
            ],
            &[
                elem("code", &[attr("slot", "text")], &encode_safe(&self.text)),
                elem("output", &[attr("slot", "output")], &output),
            ]
            .concat(),
        )
    }
}

impl ToHtml for CodeFragment {
    /// Encode a [`CodeFragment`] as HTML
    ///
    /// See `CodeBlock::to_html` for why `programming_language` is encoded
    /// as both a `class` attribute and a `<meta>` element.
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        let (class, meta) = match &self.programming_language {
            Some(programming_language) => (
                attr("class", &["language-", programming_language].concat()),
                elem_empty(
                    "meta",
                    &[
                        attr_itemprop("programming_language"),
                        attr("content", programming_language),
                    ],
                ),
            ),
            None => ("".to_string(), "".to_string()),
        };

        let text = encode_safe(&self.text);

        elem(
            "code",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                class,
            ],
            &[meta, text.to_string()].concat(),
        )
    }
}

impl ToHtml for Link {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem(
            "a",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("href", &self.target),
            ],
            &self.content.to_html("", context),
        )
    }
}

impl ToHtml for MathFragment {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem(
            "code",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &encode_safe(&self.text),
        )
    }
}

impl ToHtml for Note {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem(
            "code",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("class", "todo"),
            ],
            &json(self),
        )
    }
}

impl ToHtml for Parameter {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        let input_type = match self.validator.as_deref() {
            Some(ValidatorTypes::NumberValidator(..)) => "number",
            _ => "text",
        };
        let value_attr = match self.value.as_deref() {
            Some(node) => attr("value", &node.to_txt()),
            _ => "".to_string(),
        };
        elem_empty(
            "input",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("type", input_type),
                attr("name", &self.name),
                value_attr,
            ],
        )
    }
}

impl ToHtml for Quote {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem(
            "q",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html("", context),
        )
    }
}
