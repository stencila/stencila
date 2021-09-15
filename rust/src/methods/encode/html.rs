use super::txt::ToTxt;
use super::Options;
use crate::errors::{self, Error};
use crate::methods::transform::Transform;
use crate::patches::{Address, Slot};
use eyre::Result;
use html_escape::{encode_double_quoted_attribute, encode_safe};
use itertools::Itertools;
use serde::Serialize;
use std::any::type_name;
use std::cmp::min;
use std::{collections::BTreeMap, fs, path::PathBuf};
use stencila_schema::*;

/// Encode a `Node` to a HTML document
pub fn encode(node: &Node, options: Option<Options>) -> Result<String> {
    let html = encode_address(node, Address::new(), options.clone());

    let Options {
        theme, standalone, ..
    } = options.unwrap_or_default();

    let html = if standalone {
        wrap_standalone("", &theme, &html)
    } else {
        html
    };

    Ok(html)
}

/// Generate the HTML fragment for an address within a node
///
/// This function is used when translating a `Operation` (where any value of
/// the operation is a `Node` and the operation is applied to a `Node`) to a `DomOperation`
/// (where any value is either a HTML or JSON string and the operation is applied to a browser DOM).
pub fn encode_address(node: &Node, address: Address, options: Option<Options>) -> String {
    let Options {
        bundle, compact, ..
    } = options.unwrap_or_default();

    let mut address = address.clone();
    let slot = Slot::Name("root".to_string());
    let context = Context { root: node, bundle };
    let html = node.to_html(slot, &mut address, &context);

    if compact {
        html
    } else {
        indent(&html)
    }
}

/// Indent generated HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(html: &str) -> String {
    use quick_xml::events::Event;
    use quick_xml::{Reader, Writer};

    let mut buf = Vec::new();

    let mut reader = Reader::from_str(html);
    reader.trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        let ev = reader.read_event(&mut buf);

        match ev {
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        .expect("Failed to parse XML");

        buf.clear();
    }

    std::str::from_utf8(&*writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string()
}

/// Wrap generated HTML so that it is standalone
pub fn wrap_standalone(title: &str, theme: &str, html: &str) -> String {
    let title = if title.is_empty() { "Untitled" } else { &title };
    let theme = if theme.is_empty() { "stencila" } else { &theme };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>{title}</title>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link
            href="https://unpkg.com/@stencila/thema/dist/themes/{theme}/styles.css"
            rel="stylesheet">
        <script
            src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.esm.js"
            type="module"></script>
        <script
            src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.js"
            nomodule=""></script>
        <style>
            .error {{
                font-family: mono;
                color: #9e0000;
                background: #ffd9d9;
            }}
            .todo {{
                font-family: mono;
                color: #9e9b00;
                background: #faf9de;
            }}
            .unsupported {{
                font-family: mono;
                color: #777;
                background: #eee;
            }}
        </style>
    </head>
    <body>
        {html}
    </body>
</html>"#,
        title = title,
        theme = theme,
        html = html
    )
}

/// The encoding context.
///
/// Used by child nodes to retrieve necessary information about the
/// parent nodes when rendering themselves.
struct Context<'a> {
    /// The root node being encoded
    root: &'a Node,

    /// Whether <img>, <audio> and <video> elements should use dataURIs
    bundle: bool,
}

/// Trait for encoding a node as HTML
trait ToHtml {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String;
}

/// Encode a HTML element
///
/// Use this function for creating HTML strings for elements.
/// This, and other functions below, us slice `concat`, rather than `format!`
/// for performance (given that HTML generation may be done on every, or nearly every, keystroke).
fn elem(name: &str, attrs: &[String], content: &str) -> String {
    [
        "<",
        name,
        if attrs.is_empty() { "" } else { " " },
        attrs.join(" ").trim(),
        ">",
        content,
        "</",
        name,
        ">",
    ]
    .concat()
}

/// Encode an "empty" HTML element
///
/// An empty (a.k.a self-closing) element has no closing tag.
/// See https://developer.mozilla.org/en-US/docs/Glossary/Empty_element
fn elem_empty(name: &str, attrs: &[String]) -> String {
    [
        "<",
        name,
        if attrs.is_empty() { "" } else { " " },
        attrs.join(" ").trim(),
        "/>",
    ]
    .concat()
}

/// Encode a HTML element attribute, ensuring that the value is escaped correctly
fn attr(name: &str, value: &str) -> String {
    [
        name,
        "=\"",
        encode_double_quoted_attribute(&value).as_ref(),
        "\"",
    ]
    .concat()
}

/// Encode a node `id` as the "id" attribute of an HTML element
fn id(id: &Option<Box<String>>) -> String {
    match id.as_deref() {
        Some(id) => attr("id", id),
        None => "".to_string(),
    }
}

/// Encode the "itemtype" attribute of an HTML element
///
/// Note: there should always be a sibling "itemscope" attribute on the
/// element.
fn itemtype_string(name: &str) -> String {
    let itemtype = match name {
        // TODO: complete list of schema.org types
        "Article" | "AudioObject" | "ImageObject" | "VideoObject" => {
            ["https://schema.org/", name].concat()
        }
        _ => ["https://stenci.la/", name].concat(),
    };
    [&attr("itemtype", &itemtype), " itemscope"].concat()
}

/// Encode the "itemtype" attribute of an HTML element using the type of node
fn itemtype<Type>(_value: &Type) -> String {
    let name = type_name::<Type>();
    let name = if let Some(name) = name.strip_prefix("stencila_schema::types::") {
        name
    } else {
        tracing::error!("Unexpected type: {}", name);
        name
    };
    itemtype_string(name)
}

/// Encode an "itemprop" attribute of an HTML element
fn itemprop(itemprop: &str) -> String {
    attr("itemprop", itemprop)
}

/// Encode a "data-itemprop" attribute of an HTML element
fn data_itemprop(itemprop: &str) -> String {
    attr("data-itemprop", itemprop)
}

trait ToAttr {
    fn to_attr(self) -> String;
}

impl ToAttr for Slot {
    /// Encode a `Slot` as an attribute
    fn to_attr(self) -> String {
        let value = match self {
            Slot::Name(name) => name,
            Slot::Index(index) => index.to_string(),
            _ => return String::new(),
        };
        attr("slot", &value)
    }
}

/// Encode a node as JSON
///
/// Several of the below implementations use this, mainly as a placeholder,
/// until a complete implementation is finished. Ensures that the JSON is
/// properly escaped
fn json(node: &impl Serialize) -> String {
    encode_safe(&serde_json::to_string_pretty(node).unwrap_or_default()).to_string()
}

/// Iterate over a vector of nodes, call a string generating function on each item
/// and concatenate the strings
pub fn concat<T, F>(vec: &[T], func: F) -> String
where
    F: FnMut(&T) -> String,
{
    vec.iter().map(func).collect::<Vec<String>>().concat()
}

/// Report an error and generate a HTML error message
///
/// This is used for errors related to invalid addresses, these should never occur (in tested code :)
/// but this approach is better than panicking.
fn report_error(error: Error) -> String {
    let message = error.to_string();
    errors::report(error);
    ["<span class=\"error\">", &message, "</span>"].concat()
}

/// Report an error and generate HTML message for an invalid slot variant
fn invalid_slot_variant<Type>(variant: &str, node: Type) -> String {
    report_error(errors::invalid_slot_variant(variant, node))
}

/// Report an error and generate HTML message for an invalid slot index
fn invalid_slot_index<Type>(index: usize, node: Type) -> String {
    report_error(errors::invalid_slot_index(index, node))
}

/// Report an error and generate HTML message for an invalid slot name
fn invalid_slot_name<Type>(name: &str, node: Type) -> String {
    report_error(errors::invalid_slot_name(name, node))
}

/// Encode a slice to HTML
macro_rules! slice_to_html {
    ($type:ty) => {
        impl ToHtml for $type {
            fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
                if let Some(slot) = address.pop_front() {
                    if let Slot::Index(index) = slot {
                        if let Some(item) = self.get(index) {
                            item.to_html(Slot::None, address, context)
                        } else {
                            invalid_slot_index(index, self)
                        }
                    } else {
                        invalid_slot_variant(&slot.to_string(), self)
                    }
                } else {
                    if !matches!(slot, Slot::None) {
                        return invalid_slot_variant(&slot.to_string(), self);
                    }
                    self.iter()
                        .enumerate()
                        .map(|(index, item)| item.to_html(Slot::Index(index), address, context))
                        .collect::<Vec<String>>()
                        .join("")
                }
            }
        }
    };
}
slice_to_html!([Node]);
slice_to_html!([InlineContent]);
slice_to_html!([BlockContent]);

/// Encode a `Node` to HTML
///
/// Intended to be used at the top-level. All node types that have an
/// `impl ToHtml` below should be listed here. Not all node types
/// are supported, in which case this function returns HTML
/// indicating that that is the case.
impl ToHtml for Node {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        match self {
            Node::Array(node) => node.to_html(slot, address, context),
            Node::Article(node) => node.to_html(slot, address, context),
            Node::AudioObject(node) => node.to_html(slot, address, context),
            Node::Boolean(node) => node.to_html(slot, address, context),
            Node::Cite(node) => node.to_html(slot, address, context),
            Node::CiteGroup(node) => node.to_html(slot, address, context),
            Node::Claim(node) => node.to_html(slot, address, context),
            Node::CodeBlock(node) => node.to_html(slot, address, context),
            Node::CodeChunk(node) => node.to_html(slot, address, context),
            Node::CodeExpression(node) => node.to_html(slot, address, context),
            Node::CodeFragment(node) => node.to_html(slot, address, context),
            Node::Delete(node) => node.to_html(slot, address, context),
            Node::Emphasis(node) => node.to_html(slot, address, context),
            Node::Figure(node) => node.to_html(slot, address, context),
            Node::Heading(node) => node.to_html(slot, address, context),
            Node::ImageObject(node) => node.to_html(slot, address, context),
            Node::Integer(node) => node.to_html(slot, address, context),
            Node::Link(node) => node.to_html(slot, address, context),
            Node::List(node) => node.to_html(slot, address, context),
            Node::MathBlock(node) => node.to_html(slot, address, context),
            Node::MathFragment(node) => node.to_html(slot, address, context),
            Node::NontextualAnnotation(node) => node.to_html(slot, address, context),
            Node::Note(node) => node.to_html(slot, address, context),
            Node::Null => null_to_html(),
            Node::Number(node) => node.to_html(slot, address, context),
            Node::Object(node) => node.to_html(slot, address, context),
            Node::Paragraph(node) => node.to_html(slot, address, context),
            Node::Quote(node) => node.to_html(slot, address, context),
            Node::QuoteBlock(node) => node.to_html(slot, address, context),
            Node::String(node) => node.to_html(slot, address, context),
            Node::Strong(node) => node.to_html(slot, address, context),
            Node::Subscript(node) => node.to_html(slot, address, context),
            Node::Superscript(node) => node.to_html(slot, address, context),
            Node::Table(node) => node.to_html(slot, address, context),
            Node::ThematicBreak(node) => node.to_html(slot, address, context),
            Node::VideoObject(node) => node.to_html(slot, address, context),
            _ => elem("div", &[attr("class", "unsupported")], &json(self)),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Inline content
///////////////////////////////////////////////////////////////////////////////

impl ToHtml for InlineContent {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        match self {
            InlineContent::AudioObject(node) => node.to_html(slot, address, context),
            InlineContent::Boolean(node) => node.to_html(slot, address, context),
            InlineContent::Cite(node) => node.to_html(slot, address, context),
            InlineContent::CiteGroup(node) => node.to_html(slot, address, context),
            InlineContent::CodeExpression(node) => node.to_html(slot, address, context),
            InlineContent::CodeFragment(node) => node.to_html(slot, address, context),
            InlineContent::Delete(node) => node.to_html(slot, address, context),
            InlineContent::Emphasis(node) => node.to_html(slot, address, context),
            InlineContent::ImageObject(node) => node.to_html(slot, address, context),
            InlineContent::Integer(node) => node.to_html(slot, address, context),
            InlineContent::Link(node) => node.to_html(slot, address, context),
            InlineContent::MathFragment(node) => node.to_html(slot, address, context),
            InlineContent::NontextualAnnotation(node) => node.to_html(slot, address, context),
            InlineContent::Note(node) => node.to_html(slot, address, context),
            InlineContent::Null => null_to_html(),
            InlineContent::Number(node) => node.to_html(slot, address, context),
            InlineContent::Parameter(node) => node.to_html(slot, address, context),
            InlineContent::Quote(node) => node.to_html(slot, address, context),
            InlineContent::String(node) => node.to_html(slot, address, context),
            InlineContent::Strong(node) => node.to_html(slot, address, context),
            InlineContent::Subscript(node) => node.to_html(slot, address, context),
            InlineContent::Superscript(node) => node.to_html(slot, address, context),
            InlineContent::VideoObject(node) => node.to_html(slot, address, context),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Inline content: primitives
///////////////////////////////////////////////////////////////////////////////

fn null_to_html() -> String {
    elem("span", &[itemtype_string("Null")], &"null".to_string())
}

/// Encode an atomic to HTML
macro_rules! atomic_to_html {
    ($type:ident) => {
        impl ToHtml for $type {
            fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
                elem(
                    "span",
                    &[slot.to_attr(), itemtype_string(stringify!($type))],
                    &self.to_string(),
                )
            }
        }
    };
}
atomic_to_html!(Boolean);
atomic_to_html!(Integer);
atomic_to_html!(Number);

/// Encode a string to HTML
///
/// This is the only node type where an `itemtype` attribute, in this case `http://schema.org/String`,
/// is NOT added to the element.
///
/// The string is escaped so that the generated HTML can be safely interpolated
/// within HTML.
impl ToHtml for String {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem("span", &[slot.to_attr()], &encode_safe(self))
    }
}

/// Encode an array to HTML
impl ToHtml for Array {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "code",
            &[slot.to_attr(), itemtype_string("Array")],
            &json(self),
        )
    }
}

/// Encode an object to HTML
impl ToHtml for Object {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "code",
            &[slot.to_attr(), itemtype_string("Object")],
            &json(self),
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// Inline content: marks
///////////////////////////////////////////////////////////////////////////////

macro_rules! mark_to_html {
    ($type:ident, $tag:literal) => {
        impl ToHtml for $type {
            fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
                elem(
                    $tag,
                    &[slot.to_attr(), itemtype(self), id(&self.id)],
                    &elem(
                        "span",
                        &[attr("slot", "content")],
                        &self.content.to_html(Slot::None, address, context),
                    ),
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

///////////////////////////////////////////////////////////////////////////////
// Inline content: others
///////////////////////////////////////////////////////////////////////////////

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
    fn to_html(&self, slot: Slot, _address: &mut Address, context: &Context) -> String {
        elem(
            "audio",
            &[
                slot.to_attr(),
                itemtype_string("AudioObject"),
                id(&self.id),
                "controls".to_string(),
                content_url_to_src_attr(&self.content_url, context),
            ],
            "",
        )
    }
}

impl ToHtml for ImageObjectSimple {
    fn to_html(&self, slot: Slot, _address: &mut Address, context: &Context) -> String {
        elem_empty(
            "img",
            &[
                slot.to_attr(),
                itemtype_string("ImageObject"),
                id(&self.id),
                content_url_to_src_attr(&self.content_url, context),
            ],
        )
    }
}

impl ToHtml for VideoObjectSimple {
    fn to_html(&self, slot: Slot, _address: &mut Address, context: &Context) -> String {
        let src_attr = content_url_to_src_attr(&self.content_url, context);
        let type_attr = match &self.media_type {
            Some(media_type) => attr("type", media_type),
            None => "".to_string(),
        };
        elem(
            "video",
            &[
                slot.to_attr(),
                itemtype_string("VideoObject"),
                id(&self.id),
                "controls".to_string(),
            ],
            &elem("source", &[src_attr, type_attr], ""),
        )
    }
}

impl ToHtml for Cite {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let content = match &self.content {
            Some(nodes) => nodes.to_html(Slot::None, address, context),
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
                            CreativeWorkReferences::CreativeWorkTypes(work) => {
                                if self.target == work.id().unwrap_or_default() {
                                    Some((index, reference))
                                } else {
                                    None
                                }
                            }
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
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &elem("a", &[attr("href", &self.target)], &content),
        )
    }
}

impl ToHtml for CiteGroup {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        elem(
            "span",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &concat(&self.items, |cite| {
                cite.to_html(Slot::None, address, context)
            }),
        )
    }
}

impl ToHtml for CodeExpression {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let output = match &self.output {
            Some(output) => elem("pre", &[], &output.to_html(Slot::None, address, context)),
            None => "".to_string(),
        };
        elem(
            "stencila-code-expression",
            &[
                slot.to_attr(),
                itemtype(self),
                id(&self.id),
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
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "code",
            &[
                slot.to_attr(),
                itemtype(self),
                id(&self.id),
                match &self.programming_language {
                    Some(lang) => attr("class", &["language-", lang].concat()),
                    None => "".to_string(),
                },
            ],
            &encode_safe(&self.text),
        )
    }
}

impl ToHtml for Link {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        elem(
            "a",
            &[
                slot.to_attr(),
                itemtype(self),
                id(&self.id),
                attr("href", &self.target),
            ],
            &self.content.to_html(Slot::None, address, context),
        )
    }
}

impl ToHtml for MathFragment {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "code",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &encode_safe(&self.text),
        )
    }
}

impl ToHtml for Note {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "code",
            &[
                slot.to_attr(),
                itemtype(self),
                id(&self.id),
                attr("class", "todo"),
            ],
            &json(self),
        )
    }
}

impl ToHtml for Parameter {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "code",
            &[
                slot.to_attr(),
                itemtype(self),
                id(&self.id),
                attr("class", "todo"),
            ],
            &json(self),
        )
    }
}

impl ToHtml for Quote {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        elem(
            "q",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &self.content.to_html(Slot::None, address, context),
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// Block content
///////////////////////////////////////////////////////////////////////////////

impl ToHtml for BlockContent {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        match self {
            BlockContent::Claim(node) => node.to_html(slot, address, context),
            BlockContent::CodeBlock(node) => node.to_html(slot, address, context),
            BlockContent::CodeChunk(node) => node.to_html(slot, address, context),
            BlockContent::Collection(node) => node.to_html(slot, address, context),
            BlockContent::Figure(node) => node.to_html(slot, address, context),
            BlockContent::Heading(node) => node.to_html(slot, address, context),
            BlockContent::Include(node) => node.to_html(slot, address, context),
            BlockContent::List(node) => node.to_html(slot, address, context),
            BlockContent::MathBlock(node) => node.to_html(slot, address, context),
            BlockContent::Paragraph(node) => node.to_html(slot, address, context),
            BlockContent::QuoteBlock(node) => node.to_html(slot, address, context),
            BlockContent::Table(node) => node.to_html(slot, address, context),
            BlockContent::ThematicBreak(node) => node.to_html(slot, address, context),
        }
    }
}

impl ToHtml for ClaimSimple {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "pre",
            &[
                slot.to_attr(),
                itemtype(self),
                id(&self.id),
                attr("class", "todo"),
            ],
            &json(self),
        )
    }
}

impl ToHtml for CodeBlock {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "pre",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &elem(
                "code",
                &[match &self.programming_language {
                    Some(lang) => attr("class", &["language-", lang].concat()),
                    None => "".to_string(),
                }],
                &encode_safe(&self.text),
            ),
        )
    }
}

impl ToHtml for CodeChunk {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[data_itemprop("label")], label),
        };

        let caption = match &self.caption {
            None => String::new(),
            Some(boxed) => match &**boxed {
                CodeChunkCaption::String(string) => string.clone(),
                CodeChunkCaption::VecBlockContent(content) => {
                    content.to_html(Slot::None, address, context)
                }
            },
        };

        let text = elem("pre", &[attr("slot", "text")], &encode_safe(&self.text));

        let outputs = match &self.outputs {
            None => String::new(),
            Some(outputs) => elem(
                "pre",
                &[attr("slot", "outputs")],
                &outputs.to_html(Slot::None, address, context),
            ),
        };

        elem(
            "figure",
            &[itemtype(self)],
            &[
                label,
                elem(
                    "stencila-code-chunk",
                    &[
                        slot.to_attr(),
                        itemtype(self),
                        id(&self.id),
                        attr("programming-language", &self.programming_language),
                    ],
                    &[text, outputs].concat(),
                ),
                caption,
            ]
            .concat(),
        )
    }
}

impl ToHtml for CollectionSimple {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        elem(
            "ol",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &concat(&self.parts, |part| {
                elem("li", &[], &part.to_html(Slot::None, address, context))
            }),
        )
    }
}

impl ToHtml for FigureSimple {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[data_itemprop("label")], label),
        };

        let content = match &self.content {
            None => String::new(),
            Some(nodes) => nodes.to_html(Slot::None, address, context),
        };

        let caption = match self.caption.as_deref() {
            None => String::new(),
            Some(caption) => elem(
                "figcaption",
                &[data_itemprop("caption")],
                &match caption {
                    FigureCaption::String(string) => encode_safe(&string.clone()).to_string(),
                    FigureCaption::VecBlockContent(content) => {
                        content.to_html(Slot::None, address, context)
                    }
                },
            ),
        };

        elem(
            "figure",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &[label, content, caption].concat(),
        )
    }
}

impl ToHtml for Heading {
    /// Encode a `Heading` node to a `<h2>`, `<h3>` etc element.
    ///
    /// > Generally, it is a best practice to ensure that the beginning of a
    /// > page's main content starts with a h1 element, and also to ensure
    /// > that the page contains only one h1 element.
    /// > From https://dequeuniversity.com/rules/axe/3.5/page-has-heading-one
    ///
    /// This codec follows that recommendation and reserves `<h1>` for the
    /// `title` property of a creative work.
    ///
    /// In rare cases that there is no content in the heading, return an empty
    /// text node to avoid the 'Heading tag found with no content' accessibility error.
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let depth = match &self.depth {
            Some(depth) => min(*depth + 1, 6),
            None => 2,
        };

        elem(
            &["h", &depth.to_string()].concat(),
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &elem(
                "span",
                &[attr("slot", "content")],
                &self.content.to_html(Slot::None, address, context),
            ),
        )
    }
}

impl ToHtml for Include {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let content = self.content.as_ref().map_or_else(
            || "".to_string(),
            |content| content.to_html(Slot::None, address, context),
        );

        elem(
            "div",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &content,
        )
    }
}

impl ToHtml for List {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let tag = match &self.order {
            Some(ListOrder::Ascending) => "ol",
            _ => "ul",
        };

        let items = concat(&self.items, |item| {
            item.to_html(Slot::None, address, context)
        });

        elem(
            "div",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &elem(tag, &[attr("slot", "items")], &items),
        )
    }
}

impl ToHtml for ListItem {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => InlineContent::String("☑ ".to_string()),
            false => InlineContent::String("☐ ".to_string()),
        });

        let content = match &self.content {
            Some(content) => match content {
                ListItemContent::VecInlineContent(inlines) => match checkbox {
                    Some(checkbox) => [vec![checkbox], inlines.clone()].concat().to_html(
                        Slot::None,
                        address,
                        context,
                    ),
                    None => inlines.to_html(Slot::None, address, context),
                },
                ListItemContent::VecBlockContent(blocks) => match checkbox {
                    Some(checkbox) => {
                        // Check box is only added is the first block is a paragraph
                        if let Some(BlockContent::Paragraph(paragraph)) = blocks.first() {
                            let mut paragraph = paragraph.clone();
                            paragraph.content.insert(0, checkbox);
                            [
                                paragraph.to_html(Slot::None, address, context),
                                blocks[1..].to_html(Slot::None, address, context),
                            ]
                            .concat()
                        } else {
                            blocks.to_html(Slot::None, address, context)
                        }
                    }
                    None => blocks.to_html(Slot::None, address, context),
                },
            },
            None => "".to_string(),
        };

        elem(
            "li",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &content,
        )
    }
}

impl ToHtml for MathBlock {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem(
            "pre",
            &[
                slot.to_attr(),
                itemtype(self),
                id(&self.id),
                attr("class", "todo"),
            ],
            &encode_safe(&self.text),
        )
    }
}

impl ToHtml for Paragraph {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        elem(
            "p",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &elem(
                "span",
                &[attr("slot", "content")],
                &self.content.to_html(Slot::None, address, context),
            ),
        )
    }
}

impl ToHtml for QuoteBlock {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        elem(
            "blockquote",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &self.content.to_html(Slot::None, address, context),
        )
    }
}

impl ToHtml for TableSimple {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[data_itemprop("label")], label),
        };

        let caption = match self.caption.as_deref() {
            None => String::new(),
            Some(caption) => elem(
                "div",
                &[data_itemprop("caption")],
                &match caption {
                    TableCaption::String(string) => encode_safe(&string.clone()).to_string(),
                    TableCaption::VecBlockContent(content) => {
                        content.to_html(Slot::None, address, context)
                    }
                },
            ),
        };

        let caption = elem("caption", &[], &[label, caption].concat());

        // Partition rows into head, body and foot rows
        let mut head = Vec::new();
        let mut body = Vec::new();
        let mut foot = Vec::new();
        for row in &self.rows {
            match &row.row_type {
                Some(row_type) => match row_type {
                    TableRowRowType::Header => head.push(row),
                    TableRowRowType::Footer => foot.push(row),
                },
                _ => body.push(row),
            }
        }

        // Generate table sections with cell types defaulting to appropriate variants
        let head = elem(
            "thead",
            &[],
            &concat(&head, |row| {
                table_row_to_html(row, TableCellCellType::Header, Slot::None, address, context)
            }),
        );
        let body = elem(
            "tbody",
            &[],
            &concat(&body, |row| {
                table_row_to_html(row, TableCellCellType::Data, Slot::None, address, context)
            }),
        );
        let foot = elem(
            "tfoot",
            &[],
            &concat(&foot, |row| {
                table_row_to_html(row, TableCellCellType::Header, Slot::None, address, context)
            }),
        );

        elem(
            "table",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
            &[caption, head, body, foot].concat(),
        )
    }
}

fn table_row_to_html(
    row: &TableRow,
    cell_type: TableCellCellType,
    slot: Slot,
    address: &mut Address,
    context: &Context,
) -> String {
    let cells = concat(&row.cells, |cell| {
        let cell_type = match &cell.cell_type {
            Some(cell_type) => cell_type.clone(),
            None => cell_type.clone(),
        };
        let tag = match cell_type {
            TableCellCellType::Header => "th",
            TableCellCellType::Data => "td",
        };
        let content = match &cell.content {
            None => String::new(),
            Some(content) => match content {
                TableCellContent::VecInlineContent(nodes) => {
                    nodes.to_html(Slot::None, address, context)
                }
                TableCellContent::VecBlockContent(nodes) => {
                    nodes.to_html(Slot::None, address, context)
                }
            },
        };
        elem(tag, &[itemtype(cell)], &content)
    });

    elem("tr", &[slot.to_attr(), itemtype(row), id(&row.id)], &cells)
}

impl ToHtml for ThematicBreak {
    fn to_html(&self, slot: Slot, _address: &mut Address, _context: &Context) -> String {
        elem_empty("hr", &[slot.to_attr(), itemtype(self), id(&self.id)])
    }
}

///////////////////////////////////////////////////////////////////////////////
// Creative works
///////////////////////////////////////////////////////////////////////////////

impl ToHtml for CreativeWorkTypes {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        match self {
            CreativeWorkTypes::Article(node) => node.to_html(slot, address, context),
            CreativeWorkTypes::AudioObject(node) => node.to_html(slot, address, context),
            CreativeWorkTypes::Claim(node) => node.to_html(slot, address, context),
            CreativeWorkTypes::Collection(node) => node.to_html(slot, address, context),
            CreativeWorkTypes::Figure(node) => node.to_html(slot, address, context),
            CreativeWorkTypes::ImageObject(node) => node.to_html(slot, address, context),
            CreativeWorkTypes::Table(node) => node.to_html(slot, address, context),
            CreativeWorkTypes::VideoObject(node) => node.to_html(slot, address, context),
            _ => elem("div", &[attr("class", "unsupported")], &json(self)),
        }
    }
}

impl ToHtml for CreativeWorkContent {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        match self {
            CreativeWorkContent::String(node) => node.to_html(slot, address, context),
            CreativeWorkContent::VecNode(nodes) => nodes.to_html(slot, address, context),
        }
    }
}

impl ToHtml for Article {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let title = match &self.title {
            Some(title) => {
                let title = match &**title {
                    CreativeWorkTitle::String(title) => title.to_html(Slot::None, address, context),
                    CreativeWorkTitle::VecInlineContent(title) => {
                        title.to_html(Slot::None, address, context)
                    }
                };
                elem("h1", &[itemprop("headline")], &title)
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
                elem("ol", &[data_itemprop("authors")], &authors)
            }
            None => "".to_string(),
        };

        let affiliations = if !orgs.is_empty() {
            elem(
                "ol",
                &[data_itemprop("affiliations")],
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
                    .to_html(Slot::None, address, context),
                    ThingDescription::VecInlineContent(inlines) => Paragraph {
                        content: inlines.clone(),
                        ..Default::default()
                    }
                    .to_html(Slot::None, address, context),
                    ThingDescription::VecBlockContent(blocks) => {
                        blocks.to_html(Slot::None, address, context)
                    }
                };
                elem(
                    "section",
                    &[data_itemprop("description")],
                    &[
                        elem_empty("meta", &[itemprop("description"), attr("content", &meta)]),
                        content,
                    ]
                    .concat(),
                )
            }
            None => "".to_string(),
        };

        let content = match &self.content {
            Some(content) => elem(
                "div",
                &[attr("slot", "content")],
                &content.to_html(Slot::None, address, context),
            ),
            None => "".to_string(),
        };

        elem(
            "article",
            &[slot.to_attr(), itemtype(self), id(&self.id)],
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

// For media objects, because their simple versions generate inline HTML, wrap them in
// a <main data-itemscope="root">.

impl ToHtml for AudioObject {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        // TODO: review this approach and the need to get the itemtype correct
        Node::AudioObject(self.clone())
            .to_inline()
            .to_html(slot, address, context)
    }
}

impl ToHtml for ImageObject {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let simple = ImageObjectSimple {
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(Slot::None, address, context);
        ["<main data-itemscope=\"root\">", &simple, "</main>"].concat()
    }
}

impl ToHtml for VideoObject {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let simple = VideoObjectSimple {
            media_type: self.media_type.clone(),
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(Slot::None, address, context);
        ["<main data-itemscope=\"root\">", &simple, "</main>"].concat()
    }
}

impl ToHtml for Collection {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let Collection { parts, .. } = self;
        let simple = CollectionSimple {
            parts: parts.clone(),
            ..Default::default()
        };
        simple.to_html(Slot::None, address, context)
    }
}

impl ToHtml for Claim {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let Claim { content, .. } = self;
        let simple = ClaimSimple {
            content: content.clone(),
            ..Default::default()
        };
        simple.to_html(Slot::None, address, context)
    }
}

impl ToHtml for Figure {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let Figure {
            caption, content, ..
        } = self;
        let simple = FigureSimple {
            caption: caption.clone(),
            content: content.clone(),
            ..Default::default()
        };
        simple.to_html(Slot::None, address, context)
    }
}

impl ToHtml for Table {
    fn to_html(&self, slot: Slot, address: &mut Address, context: &Context) -> String {
        let Table { caption, rows, .. } = self;
        let simple = TableSimple {
            caption: caption.clone(),
            rows: rows.clone(),
            ..Default::default()
        };
        simple.to_html(Slot::None, address, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_json_eq,
        methods::decode::html::decode,
        utils::tests::{home, skip_slow_tests, snapshot_fixtures},
    };
    use eyre::bail;
    use insta::assert_display_snapshot;
    use serde_json::json;

    /// Encode the HTML fragment fixtures
    #[test]
    fn html_fragments() {
        snapshot_fixtures("fragments/html/*.html", |_path, content| {
            let decoded = decode(&content, false).unwrap();
            let encoded = encode(
                &decoded,
                Some(Options {
                    compact: false,
                    ..Default::default()
                }),
            )
            .unwrap();
            assert_display_snapshot!(encoded);
        });
    }

    /// Validate HTML against https://validator.github.io/validator/
    ///
    /// To run locally using the validator's Docker image:
    ///
    ///  docker run -it --rm -p 8888:8888 ghcr.io/validator/validator
    ///  RUN_SLOW_TESTS=1 HTML_VALIDATOR=http://localhost:8888 cargo test
    ///
    /// See https://github.com/validator/validator/wiki/Service-%C2%BB-Input-%C2%BB-POST-body
    /// for more on the API.
    #[tokio::test]
    async fn nu_validate() -> Result<()> {
        if skip_slow_tests() {
            return Ok(());
        }

        // Read the existing snapshot
        // We only do this for one, kitchen sink like, snapshot.
        let html = fs::read_to_string(
            home().join("rust/src/methods/encode/snapshots/html_fragments@heading.html.snap"),
        )?;
        let decoded = decode(&html, false).unwrap();
        let html = encode(
            &decoded,
            Some(Options {
                standalone: true,
                compact: false,
                ..Default::default()
            }),
        )
        .unwrap();

        // Make the POST request
        let url = if let Ok(url) = std::env::var("HTML_VALIDATOR") {
            url
        } else {
            "https://validator.w3.org/nu".to_string()
        };
        let client = reqwest::Client::new();
        let response = client
            .post([&url, "?out=json"].concat())
            .header("Content-Type", "text/html; charset=UTF-8")
            .header(
                "User-Agent",
                "Stencila tests (https://github.com/stencila/stencila/)",
            )
            .body(html)
            .send()
            .await?;
        let response = match response.error_for_status() {
            Ok(response) => response,
            Err(error) => bail!(error),
        };
        let json = response.text().await?;

        // Parse the result so it's easier to read any messages
        let result: serde_json::Value = serde_json::from_str(&json)?;
        assert_json_eq!(result, json!({"messages": []}));

        Ok(())
    }
}
