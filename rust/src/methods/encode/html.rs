use super::txt::ToTxt;
use super::Options;
use eyre::Result;
use html_escape::{encode_double_quoted_attribute, encode_safe};
use itertools::Itertools;
use std::cmp::min;
use std::collections::HashMap;
use std::fs;
use std::{collections::BTreeMap, path::PathBuf};
use stencila_schema::*;

/// Encode a `Node` to a HTML document
pub fn encode(node: &Node, options: Option<Options>) -> Result<String> {
    let Options {
        bundle,
        theme,
        standalone,
    } = options.unwrap_or_default();

    let context = Context { root: node, bundle };

    let html = node.to_html(&context);

    if standalone {
        Ok(wrap_standalone(&html, &theme))
    } else {
        Ok(html)
    }
}

/// Wrap generated HTML so that it is standalone
pub fn wrap_standalone(html: &str, theme: &str) -> String {
    let theme = if theme.is_empty() { "stencila" } else { &theme };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
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
            type="text/javascript" nomodule=""></script>
        <style>
            .todo {{
                font-family: mono;
                color: #f88;
                background: #fff2f2;
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

    /// Whether <img>, <audio> and <video> elements should
    /// use dataURIs
    bundle: bool,
}

/// Trait for encoding a node as HTML
///
/// Follows the Rust [convention](https://rust-lang.github.io/api-guidelines/naming.html#ad-hoc-conversions-follow-as_-to_-into_-conventions-c-conv)
/// of using `to_` for expensive conversions.
trait ToHtml {
    fn to_html(&self, context: &Context) -> String;
}

macro_rules! slice_to_html {
    ($type:ty) => {
        impl ToHtml for $type {
            fn to_html(&self, context: &Context) -> String {
                self.iter()
                    .map(|item| item.to_html(context))
                    .collect::<Vec<String>>()
                    .join("")
            }
        }
    };
}
slice_to_html!([Node]);
slice_to_html!([InlineContent]);
slice_to_html!([BlockContent]);

/// Encode a HTML attribute, ensuring that the value is escaped correctly
fn encode_attr(name: &str, value: &str) -> String {
    [
        name,
        "=\"",
        encode_double_quoted_attribute(&value).as_ref(),
        "\"",
    ]
    .concat()
}

/// Encode a `Node` to HTML
///
/// Intended to be used at the top-level. All node types that have an
/// `impl ToHtml` below should be listed here. Not all node types
/// are supported, in which case this function returns HTML
/// indicating that that is the case.
impl ToHtml for Node {
    fn to_html(&self, context: &Context) -> String {
        match self {
            Node::Array(node) => node.to_html(context),
            Node::Article(node) => node.to_html(context),
            Node::AudioObject(node) => node.to_html(context),
            Node::Boolean(node) => node.to_html(context),
            Node::Cite(node) => node.to_html(context),
            Node::CiteGroup(node) => node.to_html(context),
            Node::Claim(node) => node.to_html(context),
            Node::CodeBlock(node) => node.to_html(context),
            Node::CodeChunk(node) => node.to_html(context),
            Node::CodeExpression(node) => node.to_html(context),
            Node::CodeFragment(node) => node.to_html(context),
            Node::Delete(node) => node.to_html(context),
            Node::Emphasis(node) => node.to_html(context),
            Node::Figure(node) => node.to_html(context),
            Node::Heading(node) => node.to_html(context),
            Node::ImageObject(node) => node.to_html(context),
            Node::Integer(node) => node.to_html(context),
            Node::Link(node) => node.to_html(context),
            Node::List(node) => node.to_html(context),
            Node::MathBlock(node) => node.to_html(context),
            Node::MathFragment(node) => node.to_html(context),
            Node::NontextualAnnotation(node) => node.to_html(context),
            Node::Note(node) => node.to_html(context),
            Node::Null => null_to_html(),
            Node::Number(node) => node.to_html(context),
            Node::Object(node) => node.to_html(context),
            Node::Paragraph(node) => node.to_html(context),
            Node::Quote(node) => node.to_html(context),
            Node::QuoteBlock(node) => node.to_html(context),
            Node::String(node) => node.to_html(context),
            Node::Strong(node) => node.to_html(context),
            Node::Subscript(node) => node.to_html(context),
            Node::Superscript(node) => node.to_html(context),
            Node::Table(node) => node.to_html(context),
            Node::ThematicBreak(node) => node.to_html(context),
            Node::VideoObject(node) => node.to_html(context),
            _ => format!(
                r#"<div class="unsupported">{json}</div>"#,
                json = serde_json::to_string_pretty(self).unwrap_or_else(|_| "".into())
            ),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Inline content
///////////////////////////////////////////////////////////////////////////////

impl ToHtml for InlineContent {
    fn to_html(&self, context: &Context) -> String {
        match self {
            InlineContent::AudioObject(node) => node.to_html(context),
            InlineContent::Boolean(node) => node.to_html(context),
            InlineContent::Cite(node) => node.to_html(context),
            InlineContent::CiteGroup(node) => node.to_html(context),
            InlineContent::CodeExpression(node) => node.to_html(context),
            InlineContent::CodeFragment(node) => node.to_html(context),
            InlineContent::Delete(node) => node.to_html(context),
            InlineContent::Emphasis(node) => node.to_html(context),
            InlineContent::ImageObject(node) => node.to_html(context),
            InlineContent::Integer(node) => node.to_html(context),
            InlineContent::Link(node) => node.to_html(context),
            InlineContent::MathFragment(node) => node.to_html(context),
            InlineContent::NontextualAnnotation(node) => node.to_html(context),
            InlineContent::Note(node) => node.to_html(context),
            InlineContent::Null => null_to_html(),
            InlineContent::Number(node) => node.to_html(context),
            InlineContent::Quote(node) => node.to_html(context),
            InlineContent::String(node) => node.to_html(context),
            InlineContent::Strong(node) => node.to_html(context),
            InlineContent::Subscript(node) => node.to_html(context),
            InlineContent::Superscript(node) => node.to_html(context),
            InlineContent::VideoObject(node) => node.to_html(context),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Inline content: primitives
///////////////////////////////////////////////////////////////////////////////

fn null_to_html() -> String {
    r#"<span itemtype="http://schema.stenci.la/Null">null</span>"#.to_string()
}

macro_rules! atomic_to_html {
    ($type:ident, $itemtype:literal) => {
        impl ToHtml for $type {
            fn to_html(&self, _context: &Context) -> String {
                format!(
                    r#"<span itemtype="{itemtype}">{content}</span>"#,
                    itemtype = $itemtype,
                    content = self.to_string()
                )
            }
        }
    };
}
atomic_to_html!(bool, "http://schema.org/Boolean");
atomic_to_html!(i64, "http://schema.stenci.la/Integer");
atomic_to_html!(f64, "http://schema.org/Number");

/// Encode a string to HTML
///
/// This escapes characters so that the generated HTML can be safely interpolated
/// within HTML, including within quoted attributes.
impl ToHtml for String {
    fn to_html(&self, _context: &Context) -> String {
        html_escape::encode_safe(self).into()
    }
}

impl ToHtml for Vec<Primitive> {
    fn to_html(&self, context: &Context) -> String {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "".into());
        format!(
            r#"<code itemtype="http://schema.stenci.la/Array">{content}</code>"#,
            content = json.to_html(context) // Ensure string is escaped
        )
    }
}

impl ToHtml for BTreeMap<String, Primitive> {
    fn to_html(&self, context: &Context) -> String {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "".into());
        format!(
            r#"<code itemtype="http://schema.stenci.la/Object">{content}</code>"#,
            content = json.to_html(context) // Ensure string is escaped
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// Inline content: marks
///////////////////////////////////////////////////////////////////////////////

macro_rules! mark_to_html {
    ($type:ident, $tag:literal) => {
        impl ToHtml for $type {
            fn to_html(&self, context: &Context) -> String {
                format!(
                    r#"<{tag} itemtype="http://schema.stenci.la/{itemtype}">{content}</{tag}>"#,
                    tag = $tag,
                    itemtype = stringify!($type),
                    content = self.content.to_html(context)
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

            format!("data:{mime};base64,{data}", mime = mime, data = data)
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
    encode_attr("src", &url)
}

impl ToHtml for AudioObjectSimple {
    fn to_html(&self, context: &Context) -> String {
        let src_attr = content_url_to_src_attr(&self.content_url, context);
        [
            "<audio itemtype=\"http://schema.org/AudioObject\" controls ",
            &src_attr,
            "></audio>",
        ]
        .concat()
    }
}

impl ToHtml for ImageObjectSimple {
    fn to_html(&self, context: &Context) -> String {
        let src_attr = content_url_to_src_attr(&self.content_url, context);
        [
            "<img itemtype=\"http://schema.org/ImageObject\" ",
            &src_attr,
            "/>",
        ]
        .concat()
    }
}

impl ToHtml for VideoObjectSimple {
    fn to_html(&self, context: &Context) -> String {
        let src_attr = content_url_to_src_attr(&self.content_url, context);
        let type_attr = match &self.media_type {
            Some(media_type) => encode_attr("type", &media_type),
            None => "".to_string(),
        };
        [
            "<video itemtype=\"http://schema.org/VideoObject\" controls><source ",
            &src_attr,
            " ",
            &type_attr,
            "></source></video>",
        ]
        .concat()
    }
}

impl ToHtml for Cite {
    fn to_html(&self, context: &Context) -> String {
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
        format!(
            "<cite itemtype=\"http://schema.stenci.la/Cite\"><a href=\"#{target}\">{content}</a></cite>",
            target = self.target.to_html(context),
            content = content
        )
    }
}

impl ToHtml for CiteGroup {
    fn to_html(&self, context: &Context) -> String {
        format!(
            r#"<span itemtype="http://schema.stenci.la/CiteGroup">{items}</span>"#,
            items = concat(&self.items, |cite| cite.to_html(context))
        )
    }
}

impl ToHtml for CodeExpression {
    fn to_html(&self, _context: &Context) -> String {
        format!(
            r#"<code class="todo">{json}</code>"#,
            json = serde_json::to_string(self).unwrap_or_else(|_| "".into())
        )
    }
}

impl ToHtml for CodeFragment {
    fn to_html(&self, context: &Context) -> String {
        let class = match &self.programming_language {
            None => String::new(),
            Some(lang) => format!(r#"class="language-{}""#, lang.to_html(context)),
        };

        format!(
            r#"<code itemtype="http://schema.stenci.la/CodeFragment" {class}>{text}</code>"#,
            class = class,
            text = self.text.to_html(context)
        )
    }
}

impl ToHtml for Link {
    fn to_html(&self, context: &Context) -> String {
        format!(
            r#"<a itemtype="http://schema.stenci.la/Link" href="{target}">{content}</a>"#,
            target = self.target.to_html(context),
            content = self.content.to_html(context)
        )
    }
}

impl ToHtml for MathFragment {
    fn to_html(&self, _context: &Context) -> String {
        format!(
            r#"<code class="todo">{json}</code>"#,
            json = serde_json::to_string(self).unwrap_or_else(|_| "".into())
        )
    }
}

impl ToHtml for Note {
    fn to_html(&self, _context: &Context) -> String {
        format!(
            r#"<code class="todo">{json}</code>"#,
            json = serde_json::to_string(self).unwrap_or_else(|_| "".into())
        )
    }
}

impl ToHtml for Quote {
    fn to_html(&self, context: &Context) -> String {
        format!(
            r#"<q itemtype="http://schema.stenci.la/Quote">{content}</q>"#,
            content = self.content.to_html(context)
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// Block content
///////////////////////////////////////////////////////////////////////////////

impl ToHtml for BlockContent {
    fn to_html(&self, context: &Context) -> String {
        match self {
            BlockContent::Claim(node) => node.to_html(context),
            BlockContent::CodeBlock(node) => node.to_html(context),
            BlockContent::CodeChunk(node) => node.to_html(context),
            BlockContent::Collection(node) => node.to_html(context),
            BlockContent::Figure(node) => node.to_html(context),
            BlockContent::Heading(node) => node.to_html(context),
            BlockContent::List(node) => node.to_html(context),
            BlockContent::MathBlock(node) => node.to_html(context),
            BlockContent::Paragraph(node) => node.to_html(context),
            BlockContent::QuoteBlock(node) => node.to_html(context),
            BlockContent::Table(node) => node.to_html(context),
            BlockContent::ThematicBreak(node) => node.to_html(context),
        }
    }
}

impl ToHtml for ClaimSimple {
    fn to_html(&self, _context: &Context) -> String {
        format!(
            r#"<div class="todo">{json}</div>"#,
            json = serde_json::to_string(self).unwrap_or_else(|_| "".into())
        )
    }
}

impl ToHtml for CodeBlock {
    fn to_html(&self, context: &Context) -> String {
        let class = match &self.programming_language {
            None => String::new(),
            Some(lang) => format!(r#"class="language-{}""#, lang.to_html(context)),
        };

        format!(
            r#"<pre><code itemtype="http://schema.stenci.la/CodeBlock" {class}>{text}</code></pre>"#,
            class = class,
            text = self.text.to_html(context)
        )
    }
}

impl ToHtml for CodeChunk {
    fn to_html(&self, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => format!(
                r#"<label data-itemprop="label">{label}</label>"#,
                label = label
            ),
        };

        let caption = match &self.caption {
            None => String::new(),
            Some(boxed) => match &**boxed {
                CodeChunkCaption::String(string) => string.clone(),
                CodeChunkCaption::VecBlockContent(content) => content.to_html(context),
            },
        };

        let lang = match &self.programming_language {
            None => String::new(),
            Some(boxed) => *boxed.clone(),
        };

        let text = format!(
            r#"<pre slot="text"><code>{text}</code></pre>"#,
            text = self.text
        );

        let outputs = match &self.outputs {
            None => String::new(),
            Some(_outputs) => {
                r#"<pre slot="outputs"><code class="todo">outputs</code></pre>"#.into()
            }
        };

        format!(
            r#"<figure id="{id}" figure itemtype="http://schema.stenci.la/Figure" itemscope>
    {label}
    <stencila-code-chunk itemtype="http://schema.stenci.la/CodeChunk" itemscope data-programminglanguage="{lang}">
    {text}
    {outputs}
    </stencila-code-chunk>
    {caption}
</figure>"#,
            id = self.id.clone().unwrap_or_default(),
            label = label,
            lang = lang,
            text = text,
            outputs = outputs,
            caption = caption
        )
    }
}

impl ToHtml for CollectionSimple {
    fn to_html(&self, context: &Context) -> String {
        format!(
            r#"<ol itemtype="http://schema.org/Collection">{parts}</ol>"#,
            parts = concat(&self.parts, |part| part.to_html(context))
        )
    }
}

impl ToHtml for FigureSimple {
    fn to_html(&self, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => format!(
                r#"<label data-itemprop="label">{label}</label>"#,
                label = label
            ),
        };

        let content = match &self.content {
            None => String::new(),
            Some(nodes) => nodes.to_html(context),
        };

        let caption = match &self.caption {
            None => String::new(),
            Some(boxed) => match &**boxed {
                FigureCaption::String(string) => string.clone(),
                FigureCaption::VecBlockContent(content) => content.to_html(context),
            },
        };

        format!(
            r#"<figure itemtype="http://schema.stenci.la/Figure" itemscope>{label}{content}{caption}</figure>"#,
            label = label,
            content = content,
            caption = caption
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
    fn to_html(&self, context: &Context) -> String {
        let depth = match &self.depth {
            Some(depth) => min(*depth + 1, 6),
            None => 2,
        };
        format!(
            r#"<h{depth} itemtype="http://schema.stenci.la/Heading">{content}</h{depth}>"#,
            depth = depth,
            content = self.content.to_html(context)
        )
    }
}

impl ToHtml for List {
    fn to_html(&self, context: &Context) -> String {
        let tag = match &self.order {
            Some(ListOrder::Ascending) => "ol",
            _ => "ul",
        };
        let items = concat(&self.items, |item| item.to_html(context));

        [
            "<",
            tag,
            " itemtype=\"http://schema.org/ItemList\">",
            &items,
            "</",
            tag,
            ">",
        ]
        .concat()
    }
}

impl ToHtml for ListItem {
    fn to_html(&self, context: &Context) -> String {
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => InlineContent::String("☑ ".to_string()),
            false => InlineContent::String("☐ ".to_string()),
        });
        let content = match &self.content {
            Some(content) => match content {
                ListItemContent::VecInlineContent(inlines) => match checkbox {
                    Some(checkbox) => [vec![checkbox], inlines.clone()].concat().to_html(context),
                    None => inlines.to_html(context),
                },
                ListItemContent::VecBlockContent(blocks) => match checkbox {
                    Some(checkbox) => {
                        // Check box is only added is the first block is a paragraph
                        if let Some(BlockContent::Paragraph(paragraph)) = blocks.first() {
                            let mut paragraph = paragraph.clone();
                            paragraph.content.insert(0, checkbox);
                            [paragraph.to_html(context), blocks[1..].to_html(context)].concat()
                        } else {
                            blocks.to_html(context)
                        }
                    }
                    None => blocks.to_html(context),
                },
            },
            None => "".to_string(),
        };
        [
            "<li itemtype=\"http://schema.org/ListItem\">",
            &content,
            "</li>",
        ]
        .concat()
    }
}

impl ToHtml for MathBlock {
    fn to_html(&self, _context: &Context) -> String {
        format!(
            r#"<div class="todo">{json}</div>"#,
            json = serde_json::to_string(self).unwrap_or_else(|_| "".into())
        )
    }
}

impl ToHtml for Paragraph {
    fn to_html(&self, context: &Context) -> String {
        format!(
            r#"<p itemtype="http://schema.stenci.la/Paragraph">{content}</p>"#,
            content = self.content.to_html(context)
        )
    }
}

impl ToHtml for QuoteBlock {
    fn to_html(&self, context: &Context) -> String {
        format!(
            r#"<blockquote itemtype="http://schema.stenci.la/QuoteBlock">{content}</blockquote>"#,
            content = self.content.to_html(context)
        )
    }
}

impl ToHtml for TableSimple {
    fn to_html(&self, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => format!(
                r#"<label data-itemprop="label">{label}</label>"#,
                label = label
            ),
        };

        let caption = match &self.caption {
            None => String::new(),
            Some(boxed) => match &**boxed {
                TableCaption::String(string) => string.clone(),
                TableCaption::VecBlockContent(content) => content.to_html(context),
            },
        };

        let caption = format!(
            r#"<caption>{label}{caption}</caption>"#,
            label = label,
            caption = caption
        );

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
        let head = format!(
            r#"<thead>{rows}</thead>"#,
            rows = table_rows_to_html(&head, TableCellCellType::Header, context)
        );
        let body = format!(
            r#"<tbody>{rows}</tbody>"#,
            rows = table_rows_to_html(&body, TableCellCellType::Data, context)
        );
        let foot = format!(
            r#"<tfoot>{rows}</tfoot>"#,
            rows = table_rows_to_html(&foot, TableCellCellType::Header, context)
        );

        format!(
            r#"<table itemtype="http://schema.stenci.la/Table" itemscope>{caption}{head}{body}{foot}</table>"#,
            caption = caption,
            head = head,
            body = body,
            foot = foot
        )
    }
}

fn table_rows_to_html(
    rows: &[&TableRow],
    cell_type: TableCellCellType,
    context: &Context,
) -> String {
    concat(&rows, |row| {
        table_row_to_html(row, cell_type.clone(), context)
    })
}

fn table_row_to_html(row: &TableRow, cell_type: TableCellCellType, context: &Context) -> String {
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
                TableCellContent::VecInlineContent(nodes) => nodes.to_html(context),
                TableCellContent::VecBlockContent(nodes) => nodes.to_html(context),
            },
        };
        format!(r#"<{tag}>{content}</{tag}>"#, tag = tag, content = content)
    });

    format!(r#"<tr>{cells}</tr>"#, cells = cells)
}

impl ToHtml for ThematicBreak {
    fn to_html(&self, _context: &Context) -> String {
        r#"<hr itemtype="http://schema.stenci.la/ThematicBreak">"#.to_string()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Creative works
///////////////////////////////////////////////////////////////////////////////

impl ToHtml for CreativeWorkTypes {
    fn to_html(&self, context: &Context) -> String {
        match self {
            CreativeWorkTypes::Article(node) => node.to_html(context),
            CreativeWorkTypes::AudioObject(node) => node.to_html(context),
            CreativeWorkTypes::Claim(node) => node.to_html(context),
            CreativeWorkTypes::Collection(node) => node.to_html(context),
            CreativeWorkTypes::Figure(node) => node.to_html(context),
            CreativeWorkTypes::ImageObject(node) => node.to_html(context),
            CreativeWorkTypes::Table(node) => node.to_html(context),
            CreativeWorkTypes::VideoObject(node) => node.to_html(context),
            _ => format!(
                r#"<div class="unsupported">{json}</div>"#,
                json = serde_json::to_string_pretty(self).unwrap_or_else(|_| "".into())
            ),
        }
    }
}

impl ToHtml for CreativeWorkContent {
    fn to_html(&self, context: &Context) -> String {
        match self {
            CreativeWorkContent::String(node) => node.to_html(context),
            CreativeWorkContent::VecNode(nodes) => nodes.to_html(context),
        }
    }
}

impl ToHtml for Article {
    fn to_html(&self, context: &Context) -> String {
        let title = match &self.title {
            Some(title) => {
                let title = match &**title {
                    CreativeWorkTitle::String(title) => title.to_html(context),
                    CreativeWorkTitle::VecInlineContent(title) => title.to_html(context),
                };
                ["<h1 itemprop=\"headline\">", &title, "</h1>"].concat()
            }
            None => "".to_string(),
        };

        // Create a map of organization name to Organization, in the order
        // they appear in affiliations.
        let orgs: HashMap<String, &Organization> = match &self.authors {
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
            None => HashMap::new(),
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
                ["<ol data-itemprop=\"authors\">", &authors, "</ol>"].concat()
            }
            None => "".to_string(),
        };

        let affiliations = if !orgs.is_empty() {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            [
                "<ol data-itemprop=\"affiliations\">",
                    &concat(&orgs, |org| affiliation_org_to_html(org)),
                "</ol>",
            ]
            .concat()
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
                    .to_html(context),
                    ThingDescription::VecInlineContent(inlines) => Paragraph {
                        content: inlines.clone(),
                        ..Default::default()
                    }
                    .to_html(context),
                    ThingDescription::VecBlockContent(blocks) => blocks.to_html(context),
                };
                #[cfg_attr(rustfmt, rustfmt_skip)]
                [
                    "<section data-itemprop=\"description\">",
                        "<h2>Abstract</h2>",
                        "<meta itemprop=\"description\"", &encode_attr("content", &meta), ">",
                        &content,
                    "</section>",
                ]
                .concat()
            }
            None => "".to_string(),
        };

        let content = match &self.content {
            Some(content) => content.to_html(context),
            None => "".to_string(),
        };

        #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            "<article itemtype=\"http://schema.org/Article\" itemscope data-itemscope=\"root\">",
                &title,
                &authors,
                &affiliations,
                &abstract_,
                &content,
            "</article>",
        ]
        .concat()
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
            "<meta itemprop=\"name\"", &encode_attr("content", &name_string), ">",
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
                        "<a itemprop=\"email\"", &encode_attr("href", &["mailto:", email].concat()), ">",
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
                        "<a itemprop=\"affiliation\"", &encode_attr("href", &position), ">",
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
    fn to_html(&self, context: &Context) -> String {
        let simple = AudioObjectSimple {
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(context);
        ["<main data-itemscope=\"root\">", &simple, "</main>"].concat()
    }
}

impl ToHtml for ImageObject {
    fn to_html(&self, context: &Context) -> String {
        let simple = ImageObjectSimple {
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(context);
        ["<main data-itemscope=\"root\">", &simple, "</main>"].concat()
    }
}

impl ToHtml for VideoObject {
    fn to_html(&self, context: &Context) -> String {
        let simple = VideoObjectSimple {
            media_type: self.media_type.clone(),
            content_url: self.content_url.clone(),
            ..Default::default()
        }
        .to_html(context);
        ["<main data-itemscope=\"root\">", &simple, "</main>"].concat()
    }
}

impl ToHtml for Collection {
    fn to_html(&self, context: &Context) -> String {
        let Collection { parts, .. } = self;
        let simple = CollectionSimple {
            parts: parts.clone(),
            ..Default::default()
        };
        simple.to_html(context)
    }
}

impl ToHtml for Claim {
    fn to_html(&self, context: &Context) -> String {
        let Claim { content, .. } = self;
        let simple = ClaimSimple {
            content: content.clone(),
            ..Default::default()
        };
        simple.to_html(context)
    }
}

impl ToHtml for Figure {
    fn to_html(&self, context: &Context) -> String {
        let Figure {
            caption, content, ..
        } = self;
        let simple = FigureSimple {
            caption: caption.clone(),
            content: content.clone(),
            ..Default::default()
        };
        simple.to_html(context)
    }
}

impl ToHtml for Table {
    fn to_html(&self, context: &Context) -> String {
        let Table { caption, rows, .. } = self;
        let simple = TableSimple {
            caption: caption.clone(),
            rows: rows.clone(),
            ..Default::default()
        };
        simple.to_html(context)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Helper functions
///////////////////////////////////////////////////////////////////////////////

/// Iterate over a vector of node, call a string generating function on each item
/// and concatenate the strings
pub fn concat<T, F>(vec: &[T], func: F) -> String
where
    F: FnMut(&T) -> String,
{
    vec.iter().map(func).collect::<Vec<String>>().concat()
}

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    // Encode article fixtures to HTML for previewing
    // Currently the snapshots are not committed for "gold master testing"
    #[test]
    fn article_fixtures() -> Result<()> {
        let home = &PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let articles = home.join("..").join("fixtures").join("articles");
        let snapshots = home.join("snapshots");

        for file in vec!["elife-small.json", "era-plotly.json"] {
            let fixture_path = &articles.join(file);
            let json = fs::read_to_string(fixture_path)?;
            let article: Node = serde_json::from_str(&json)?;

            let html = encode(&article, None)?;

            let snapshot_path = snapshots.join(format!(
                "{}.html",
                fixture_path.file_stem().unwrap().to_str().unwrap()
            ));
            fs::write(snapshot_path, html)?;
        }
        Ok(())
    }
}
