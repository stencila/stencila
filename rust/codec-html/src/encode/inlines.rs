//! Encode `InlineContent` nodes to HTML

use std::{fmt::Write, fs, path::PathBuf};

use codec::common::{base64, tracing};
use stencila_schema::*;

use super::{
    attr, attr_and_meta, attr_and_meta_opt, attr_id, attr_itemprop, attr_itemtype,
    attr_itemtype_str, attr_prop, attr_slot, concat, elem, elem_empty, elem_meta, elem_placeholder,
    json, nothing, EncodeContext, ToHtml,
};

impl ToHtml for InlineContent {
    fn to_html(&self, context: &EncodeContext) -> String {
        match self {
            InlineContent::AudioObject(node) => node.to_html(context),
            InlineContent::Boolean(node) => node.to_html(context),
            InlineContent::Cite(node) => node.to_html(context),
            InlineContent::CiteGroup(node) => node.to_html(context),
            InlineContent::CodeExpression(node) => node.to_html(context),
            InlineContent::CodeFragment(node) => node.to_html(context),
            InlineContent::Date(node) => node.to_html(context),
            InlineContent::DateTime(node) => node.to_html(context),
            InlineContent::Delete(node) => node.to_html(context),
            InlineContent::Duration(node) => node.to_html(context),
            InlineContent::Emphasis(node) => node.to_html(context),
            InlineContent::ImageObject(node) => node.to_html(context),
            InlineContent::Integer(node) => node.to_html(context),
            InlineContent::Link(node) => node.to_html(context),
            InlineContent::MathFragment(node) => node.to_html(context),
            InlineContent::NontextualAnnotation(node) => node.to_html(context),
            InlineContent::Note(node) => node.to_html(context),
            InlineContent::Null(node) => node.to_html(context),
            InlineContent::Number(node) => node.to_html(context),
            InlineContent::Parameter(node) => node.to_html(context),
            InlineContent::Quote(node) => node.to_html(context),
            // Unlike everything else, `String` instances are not wrapped in an element.
            // However, `InlineContent::String` (and `Node::String` instances) should be
            // because they are usually part of a vector (e.g. `Paragraph.content`).
            // Wrapping them indicates to the DOM patching algorithm that the container element
            // represents a vector and not a placeholder for an optional string property.
            // To reduce the size of the generated HTML a little, unlike for `Node::String`,
            // the `itemtype` attribute is not used.
            InlineContent::String(node) => elem("span", &[], &node.to_html(context)),
            InlineContent::Strikeout(node) => node.to_html(context),
            InlineContent::Strong(node) => node.to_html(context),
            InlineContent::Subscript(node) => node.to_html(context),
            InlineContent::Superscript(node) => node.to_html(context),
            InlineContent::Time(node) => node.to_html(context),
            InlineContent::Timestamp(node) => node.to_html(context),
            InlineContent::Underline(node) => node.to_html(context),
            InlineContent::VideoObject(node) => node.to_html(context),
        }
    }
}

macro_rules! mark_to_html {
    ($type:ident, $tag:literal) => {
        impl ToHtml for $type {
            fn to_html(&self, context: &EncodeContext) -> String {
                elem(
                    $tag,
                    &[attr_itemtype::<Self>(), attr_id(&self.id)],
                    &self.content.to_html(context),
                )
            }
        }
    };
}

mark_to_html!(Emphasis, "em");
mark_to_html!(Strikeout, "del");
mark_to_html!(Delete, "del");
mark_to_html!(Strong, "strong");
mark_to_html!(Subscript, "sub");
mark_to_html!(Superscript, "sup");
mark_to_html!(Underline, "u");
mark_to_html!(NontextualAnnotation, "u");

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

fn content_url_to_src_attr(content_url: &str, context: &EncodeContext) -> String {
    let url = match context.bundle {
        true => file_uri_to_data_uri(content_url),
        false => content_url.to_string(),
    };
    attr("src", &url)
}

impl ToHtml for AudioObjectSimple {
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "audio",
            &[
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
    fn to_html(&self, context: &EncodeContext) -> String {
        fn decode_base64(data: &str) -> String {
            match base64::decode(data) {
                Ok(data) => String::from_utf8_lossy(&data).to_string(),
                Err(error) => {
                    tracing::error!("While decoding base64 image data: {}", error);
                    format!("\"{}\"", error)
                }
            }
        }

        if let Some(data) = self
            .content_url
            .strip_prefix("data:application/vnd.plotly.")
        {
            let mut parts = data.split(";base64,");

            let version = parts.next().unwrap_or("1");
            let media_type = match &self.media_type {
                Some(value) => *value.clone(),
                None => ["application/vnd.plotly.", version].concat(),
            };

            let data = decode_base64(parts.next().unwrap_or_default());

            elem(
                "stencila-image-plotly",
                &[],
                &elem(
                    "picture",
                    &[],
                    &elem("script", &[attr("type", &media_type)], &data),
                ),
            )
        } else if let Some(data) = self.content_url.strip_prefix("data:application/vnd.vega") {
            let mut parts = data.split(";base64,");
            let data = decode_base64(parts.nth(1).unwrap_or_default());

            elem(
                "stencila-image-vega",
                &[],
                &elem(
                    "picture",
                    &[],
                    &elem(
                        "script",
                        &[attr("type", "application/vnd.vega+json")],
                        &data,
                    ),
                ),
            )
        } else {
            elem_empty(
                "img",
                &[
                    attr_itemtype_str("ImageObject"),
                    attr_id(&self.id),
                    content_url_to_src_attr(&self.content_url, context),
                ],
            )
        }
    }
}

impl ToHtml for VideoObjectSimple {
    fn to_html(&self, context: &EncodeContext) -> String {
        let src_attr = content_url_to_src_attr(&self.content_url, context);
        let type_attr = match &self.media_type {
            Some(media_type) => attr("type", media_type),
            None => "".to_string(),
        };
        elem(
            "video",
            &[
                attr_itemtype_str("VideoObject"),
                attr_id(&self.id),
                "controls".to_string(),
            ],
            &elem("source", &[src_attr, type_attr], ""),
        )
    }
}

impl ToHtml for Cite {
    fn to_html(&self, context: &EncodeContext) -> String {
        let content = match &self.content {
            Some(nodes) => nodes.to_html(context),
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
                                    write!(content, r#"<span>{}</span>"#, names)
                                        .expect("Unable to write to string")
                                }
                                if let Some(date) = date {
                                    if date.value.len() >= 4 {
                                        let year = date.value[..4].to_string();
                                        write!(content, r#"<span>{}</span>"#, year)
                                            .expect("Unable to write to string")
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
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &elem("a", &[attr("href", &self.target)], &content),
        )
    }
}

impl ToHtml for CiteGroup {
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "span",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &concat(&self.items, |cite| cite.to_html(context)),
        )
    }
}

impl ToHtml for CodeExpression {
    fn to_html(&self, context: &EncodeContext) -> String {
        let lang = attr_and_meta("programming_language", &self.programming_language);

        let compile_digest = attr_and_meta_opt(
            "compile_digest",
            self.compile_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_digest = attr_and_meta_opt(
            "execute_digest",
            self.execute_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_required = attr_and_meta_opt(
            "execute_required",
            self.execute_required
                .as_ref()
                .map(|required| (*required).as_ref().to_string()),
        );

        let execute_kernel = attr_and_meta_opt(
            "execute_kernel",
            self.execute_kernel
                .as_deref()
                .map(|kernel| kernel.to_string()),
        );

        let execute_status = attr_and_meta_opt(
            "execute_status",
            self.execute_status
                .as_ref()
                .map(|status| (*status).as_ref().to_string()),
        );

        let execute_ended = attr_and_meta_opt(
            "execute_ended",
            self.execute_ended
                .as_ref()
                .map(|date| (**date).value.to_string()),
        );

        let execute_duration = attr_and_meta_opt(
            "execute_duration",
            self.execute_duration
                .as_ref()
                .map(|seconds| seconds.to_string()),
        );

        let execute_count = attr_and_meta_opt(
            "execute_count",
            self.execute_count.map(|count| count.to_string()),
        );

        let text = elem(
            "code",
            &[attr_prop("text"), attr_slot("text")],
            &self.text.to_html(context),
        );

        // For code_dependencies it is necessary to place the items in a <span> under
        // the custom element to avoid elements added by the Web Component interfering
        // with patch indexes.
        let dependencies = elem(
            "stencila-code-dependencies",
            &[attr_slot("code-dependencies")],
            &elem_placeholder(
                "span",
                &[attr_prop("code-dependencies")],
                &self.code_dependencies,
                context,
            ),
        );

        let output = elem_placeholder(
            "output",
            &[attr_prop("output"), attr_slot("output")],
            &self.output,
            &EncodeContext {
                inline: true,
                ..*context
            },
        );

        let errors = elem_placeholder(
            "span",
            &[attr_prop("errors"), attr_slot("errors")],
            &self.errors,
            context,
        );

        elem(
            "stencila-code-expression",
            &[
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                lang.0,
                compile_digest.0,
                execute_digest.0,
                execute_required.0,
                execute_kernel.0,
                execute_status.0,
                execute_ended.0,
                execute_duration.0,
                execute_count.0,
            ],
            &[
                lang.1,
                compile_digest.1,
                execute_digest.1,
                execute_required.1,
                execute_kernel.1,
                execute_status.1,
                execute_ended.1,
                execute_duration.1,
                execute_count.1,
                text,
                dependencies,
                output,
                errors,
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
    fn to_html(&self, context: &EncodeContext) -> String {
        let (lang_attr, lang_class, lang_meta) = match &self.programming_language {
            Some(programming_language) => (
                attr("programming-language", programming_language),
                attr("class", &["language-", programming_language].concat()),
                elem_meta("programmingLanguage", programming_language),
            ),
            None => (nothing(), nothing(), nothing()),
        };

        let text = elem(
            "code",
            &[attr_itemprop("text"), attr_slot("text"), lang_class],
            &self.text.to_html(context),
        );

        elem(
            "stencila-code-fragment",
            &[attr_itemtype::<Self>(), attr_id(&self.id), lang_attr],
            &[lang_meta, text].concat(),
        )
    }
}

impl ToHtml for Link {
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "a",
            &[
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("href", &self.target),
            ],
            &self.content.to_html(context),
        )
    }
}

impl ToHtml for Note {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem(
            "code",
            &[
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("class", "todo"),
            ],
            &json(self),
        )
    }
}

impl ToHtml for Quote {
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "q",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html(context),
        )
    }
}
