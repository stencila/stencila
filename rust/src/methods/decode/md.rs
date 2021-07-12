use super::html;
use crate::{methods::coerce::coerce, traits::ToVecInlineContent};
use eyre::{bail, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map_res, not, peek},
    multi::{fold_many0, separated_list1},
    sequence::{delimited, preceded, tuple},
    IResult,
};
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use regex::Regex;
use stencila_schema::{
    Article, BlockContent, Cite, CiteGroup, CodeBlock, CodeFragment, Delete, Emphasis, Heading,
    ImageObjectSimple, InlineContent, Link, List, ListItem, ListItemContent, MathFragment, Node,
    Paragraph, QuoteBlock, Strong, Subscript, Superscript, TableCell, TableCellContent, TableRow,
    TableRowRowType, TableSimple, ThematicBreak,
};

/// Decode a Markdown document to a `Node`
///
/// Intended for decoding an entire document, this function extracts
/// YAML front matter, parses the Markdown, and returns a `Node::Article` variant.
pub fn decode(md: &str) -> Result<Node> {
    let (md, mut node) = if let Some((end, node)) = decode_frontmatter(&md)? {
        (&md[end..], node)
    } else {
        (md, Node::Article(Article::default()))
    };

    let content = decode_fragment(md);
    if !content.is_empty() {
        let content = Some(content);
        match &mut node {
            Node::Article(article) => article.content = content,
            _ => bail!("Unsupported node type {:?}", node),
        }
    }

    Ok(node)
}

/// Decode any front matter in a Markdown document into a `Node`
///
/// Any front matter will be coerced into a `Node`, defaulting to the
/// `Node::Article` variant, if `type` is not defined.
/// If there is no front matter detected, will return `None`.
pub fn decode_frontmatter(md: &str) -> Result<Option<(usize, Node)>> {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new("^-{3,}((.|\\n)*)?\\n-{3,}").expect("Unable to create regex"));

    if let Some(captures) = REGEX.captures(md) {
        let yaml = captures[1].trim().to_string();
        if yaml.is_empty() {
            return Ok(None);
        }

        let mut value: serde_json::Value = serde_yaml::from_str(&yaml)?;
        if value.get("type").is_none() {
            value["type"] = serde_json::Value::String("Article".to_string());
        };

        let node = coerce(value)?;
        Ok(Some((captures[0].len(), node)))
    } else {
        Ok(None)
    }
}

/// Decode a Markdown fragment to a vector of `BlockContent`
///
/// Intended for decoding a fragment of Markdown (e.g. a Markdown cell in
/// a Jupyter Notebook) and inserting it into a larger document.
///
/// Uses the `pulldown_cmark` and transforms its `Event`s into
/// `Vec<BlockContent>`. Text is further parsed using `nom` based parsers
/// to handle the elements that `pulldown_cmark` does not handle (e.g. math).
pub fn decode_fragment(md: &str) -> Vec<BlockContent> {
    let mut inlines = Inlines {
        active: false,
        text: String::new(),
        nodes: Vec::new(),
        marks: Vec::new(),
    };

    let mut html = Html {
        html: String::new(),
        tags: Vec::new(),
    };

    let mut lists = Lists {
        items: Vec::new(),
        marks: Vec::new(),
        is_checked: None,
    };

    let mut tables = Tables {
        rows: Vec::new(),
        cells: Vec::new(),
    };

    let mut blocks = Blocks {
        nodes: Vec::new(),
        marks: Vec::new(),
    };

    let parser = Parser::new_ext(md, Options::all());
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                // Block nodes with block content or special handling
                Tag::BlockQuote => blocks.push_mark(),
                Tag::List(..) => lists.push_mark(),
                Tag::Item => {
                    inlines.push_mark();
                    blocks.push_mark()
                }
                Tag::Table(..) => (),
                Tag::TableHead => (),
                Tag::TableRow => (),
                Tag::TableCell => {
                    inlines.push_mark();
                    blocks.push_mark()
                }

                // Block nodes with inline content
                Tag::Heading(..) => inlines.clear_all(),
                Tag::Paragraph => inlines.clear_all(),
                Tag::CodeBlock(..) => inlines.clear_all(),

                // Inline nodes with inline content
                Tag::Emphasis => inlines.push_mark(),
                Tag::Strong => inlines.push_mark(),
                Tag::Strikethrough => inlines.push_mark(),
                Tag::Link(..) => inlines.push_mark(),
                Tag::Image(..) => inlines.push_mark(),

                // Currently unhandled
                Tag::FootnoteDefinition(_) => (),
            },
            Event::End(tag) => match tag {
                // Block nodes with block content
                Tag::BlockQuote => {
                    let content = blocks.pop_tail();
                    blocks.push_node(BlockContent::QuoteBlock(QuoteBlock {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::List(start) => {
                    let order = if start.is_some() {
                        Some(stencila_schema::ListOrder::Ascending)
                    } else {
                        None
                    };

                    let items = lists.pop_tail();

                    blocks.push_node(BlockContent::List(List {
                        items,
                        order,

                        ..Default::default()
                    }))
                }
                Tag::Item => {
                    let mut content = Vec::new();

                    let inlines = inlines.pop_tail();
                    if !inlines.is_empty() {
                        content.push(BlockContent::Paragraph(Paragraph {
                            content: inlines,
                            ..Default::default()
                        }))
                    }

                    let mut blocks = blocks.pop_tail();
                    content.append(&mut blocks);

                    let content = Some(ListItemContent::VecBlockContent(content));

                    lists.push_item(ListItem {
                        content,
                        ..Default::default()
                    })
                }
                Tag::Table(_) => blocks.push_node(BlockContent::Table(TableSimple {
                    rows: tables.pop_rows(),
                    ..Default::default()
                })),
                Tag::TableHead => tables.push_header(),
                Tag::TableRow => tables.push_row(),
                Tag::TableCell => {
                    let inlines = inlines.pop_tail();
                    let content = if inlines.is_empty() {
                        None
                    } else {
                        Some(TableCellContent::VecInlineContent(inlines))
                    };

                    tables.push_cell(TableCell {
                        content,
                        ..Default::default()
                    })
                }

                // Block nodes with inline content
                Tag::Heading(depth) => blocks.push_node(BlockContent::Heading(Heading {
                    depth: Some(depth as u8),
                    content: inlines.pop_all(),
                    ..Default::default()
                })),
                Tag::Paragraph => blocks.push_node(BlockContent::Paragraph(Paragraph {
                    content: inlines.pop_all(),
                    ..Default::default()
                })),
                Tag::CodeBlock(kind) => {
                    let text = inlines.pop_text();
                    blocks.push_node(BlockContent::CodeBlock(CodeBlock {
                        text,
                        programming_language: match kind {
                            CodeBlockKind::Fenced(lang) => {
                                let lang = lang.to_string();
                                if !lang.is_empty() {
                                    Some(Box::new(lang))
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                        ..Default::default()
                    }))
                }

                // Inline nodes with inline content
                Tag::Emphasis => {
                    let content = inlines.pop_tail();
                    inlines.push_node(InlineContent::Emphasis(Emphasis {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strong => {
                    let content = inlines.pop_tail();
                    inlines.push_node(InlineContent::Strong(Strong {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strikethrough => {
                    let content = inlines.pop_tail();
                    inlines.push_node(InlineContent::Delete(Delete {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Link(_link_type, url, title) => {
                    let content = inlines.pop_tail();
                    let title = {
                        let title = title.to_string();
                        if !title.is_empty() {
                            Some(Box::new(title))
                        } else {
                            None
                        }
                    };
                    inlines.push_node(InlineContent::Link(Link {
                        content,
                        target: url.to_string(),
                        title,
                        ..Default::default()
                    }))
                }
                Tag::Image(_link_type, url, title) => {
                    let title = {
                        let title = title.to_string();
                        if !title.is_empty() {
                            Some(Box::new(title))
                        } else {
                            None
                        }
                    };
                    inlines.push_node(InlineContent::ImageObject(ImageObjectSimple {
                        content_url: url.to_string(),
                        caption: title,
                        ..Default::default()
                    }))
                }

                Tag::FootnoteDefinition(..) => {
                    // TODO: Handle footnote definitions
                    tracing::debug!("Markdown footnote definitions are not yet handled")
                }
            },
            Event::Code(value) => {
                inlines.push_node(InlineContent::CodeFragment(CodeFragment {
                    text: value.to_string(),
                    ..Default::default()
                }));
            }
            Event::Rule => blocks.push_node(BlockContent::ThematicBreak(ThematicBreak {
                ..Default::default()
            })),
            Event::Text(value) => {
                // Text gets accumulated to HTML we're inside a tag, to inlines otherwise
                let value = value.to_string();
                if html.tags.is_empty() {
                    inlines.push_text(&value)
                } else {
                    html.html.push_str(&value)
                }
            }
            Event::SoftBreak => {
                // A soft line break event occurs between lines of a multi-line paragraph
                // (between a `Text` event for each line). This inserts the Unicode soft break
                // character so that, when inlines are decoded a space can be added if
                // necessary.
                inlines.push_text("\u{2029}")
            }
            Event::HardBreak => {
                tracing::debug!("Markdown HardBreaks are not yet handled");
            }
            Event::Html(content) => {
                let mut content = html.handle_html(&content.to_string());
                if !content.is_empty() {
                    if inlines.active {
                        inlines.append_nodes(&mut content.to_vec_inline_content())
                    } else {
                        blocks.append_nodes(&mut content)
                    }
                }
            }
            Event::FootnoteReference(..) => {
                // TODO: Handle footnote references
                tracing::debug!("Markdown footnote references are not yet handled");
            }
            Event::TaskListMarker(is_checked) => lists.is_checked = Some(is_checked),
        };
    }

    if !html.tags.is_empty() {
        tracing::warn!("Unclosed HTML tags: {:?}", html.tags)
    }

    blocks.pop_all()
}

/// Stores block content
struct Blocks {
    nodes: Vec<BlockContent>,
    marks: Vec<usize>,
}

impl Blocks {
    /// Push a node
    fn push_node(&mut self, node: BlockContent) {
        self.nodes.push(node)
    }

    /// Append nodes
    fn append_nodes(&mut self, nodes: &mut Vec<BlockContent>) {
        self.nodes.append(nodes)
    }

    /// Push a mark (usually at the start of a block node)
    fn push_mark(&mut self) {
        self.marks.push(self.nodes.len())
    }

    /// Pop the nodes since the last mark
    fn pop_tail(&mut self) -> Vec<BlockContent> {
        let n = self.marks.pop().expect("Unable to pop marks!");
        self.nodes.split_off(n)
    }

    /// Pop all the nodes
    fn pop_all(&mut self) -> Vec<BlockContent> {
        self.nodes.split_off(0)
    }
}

/// Stores list items
///
/// It is necessary to maintain marks to handle nested lists
struct Lists {
    /// Stack of list items
    items: Vec<ListItem>,

    /// Marks in the stack indicating the start of a list
    marks: Vec<usize>,

    /// Whether or not the current item has check box / is checked
    is_checked: Option<bool>,
}

impl Lists {
    /// Push a list item
    fn push_item(&mut self, mut item: ListItem) {
        item.is_checked = self.is_checked;
        self.items.push(item);
        self.is_checked = None;
    }

    /// Push a mark at the start of a list
    fn push_mark(&mut self) {
        self.marks.push(self.items.len())
    }

    /// Pop the items since the last mark
    fn pop_tail(&mut self) -> Vec<ListItem> {
        if self.marks.is_empty() {
            vec![]
        } else {
            let n = self.marks.pop().expect("Unable to pop marks!");
            self.items.split_off(n)
        }
    }
}

/// Stores table rows and cells
struct Tables {
    rows: Vec<TableRow>,
    cells: Vec<TableCell>,
}

impl Tables {
    /// Push a cell
    fn push_cell(&mut self, cell: TableCell) {
        self.cells.push(cell)
    }

    /// Pop cells into a pushed header row
    fn push_header(&mut self) {
        let cells = self.cells.split_off(0);
        self.rows.push(TableRow {
            cells,
            row_type: Some(TableRowRowType::Header),
            ..Default::default()
        })
    }

    /// Pop cells into a pushed row
    fn push_row(&mut self) {
        let cells = self.cells.split_off(0);
        self.rows.push(TableRow {
            cells,
            ..Default::default()
        })
    }

    /// Pop all rows
    fn pop_rows(&mut self) -> Vec<TableRow> {
        self.rows.split_off(0)
    }
}

/// Stores and parses inline content
struct Inlines {
    active: bool,
    text: String,
    nodes: Vec<InlineContent>,
    marks: Vec<usize>,
}

impl Inlines {
    /// Clear all content and mark as "active"
    /// (usually at the start of a block node with inline content)
    fn clear_all(&mut self) {
        self.text.clear();
        self.nodes.clear();
        self.marks.clear();
        self.active = true;
    }

    /// Push some text content so it can be processed later
    ///
    /// If the new text is a soft break and the existing text does not end
    /// with whitespace, will add a single space.
    fn push_text(&mut self, text: &str) {
        if text == "\u{2029}" && !self.text.ends_with(|chr: char| chr.is_whitespace()) {
            self.text.push(' ')
        } else {
            self.text.push_str(text)
        }
    }

    /// Pop all the text content (usually for use in a node e.g `CodeBlock`)
    fn pop_text(&mut self) -> String {
        self.text.split_off(0)
    }

    /// Parse the accumulated text into accumulated `InlineContent` nodes
    ///
    /// This is the entry point into `nom` Markdown parsing functions.
    /// It is infallible in that if there is a parse error,
    /// the original input string is returned as the only item
    /// in the vector (with a warning).
    fn parse_text(&mut self) {
        if !self.text.is_empty() {
            let text = self.pop_text();
            let mut nodes = match inline_content(&text) {
                Ok((_, content)) => content,
                Err(error) => {
                    tracing::warn!("While parsing inline Markdown: {}", error);
                    vec![InlineContent::String(text)]
                }
            };
            self.nodes.append(&mut nodes)
        }
    }

    /// Push a node
    fn push_node(&mut self, node: InlineContent) {
        self.parse_text();
        self.nodes.push(node)
    }

    /// Append nodes
    fn append_nodes(&mut self, nodes: &mut Vec<InlineContent>) {
        self.parse_text();
        self.nodes.append(nodes)
    }

    /// Push a mark (usually at the start of an inline node)
    fn push_mark(&mut self) {
        self.parse_text();
        self.marks.push(self.nodes.len())
    }

    /// Pop the nodes since the last mark
    fn pop_tail(&mut self) -> Vec<InlineContent> {
        self.parse_text();
        if self.marks.is_empty() {
            vec![]
        } else {
            let n = self.marks.pop().expect("Unable to pop marks!");
            self.nodes.split_off(n)
        }
    }

    /// Pop all the nodes and mark as "inactive"
    fn pop_all(&mut self) -> Vec<InlineContent> {
        self.parse_text();
        self.active = false;
        self.nodes.split_off(0)
    }
}

/// Parse a string into a vector of `InlineContent` nodes
///
/// Whilst accumulating, will combine adjacent `String` nodes.
/// This is necessary because of the catch all `character` parser.
fn inline_content(input: &str) -> IResult<&str, Vec<InlineContent>> {
    fold_many0(
        alt((
            cite_group,
            cite,
            math,
            subscript,
            superscript,
            string,
            character,
        )),
        Vec::new(),
        |mut vec: Vec<InlineContent>, node| {
            if let InlineContent::String(string) = &node {
                match vec.last_mut() {
                    Some(InlineContent::String(last)) => last.push_str(&string),
                    _ => vec.push(node),
                }
            } else {
                vec.push(node)
            }
            vec
        },
    )(input)
}

/// Parse a string into a narrative `Cite` node
///
/// This attempts to follow Pandoc's citation handling as closely as possible
/// (see https://pandoc.org/MANUAL.html#citations).
///
/// The following properties of a `Cite` are parsed:
///   - [x] target
///   - [ ] citation_mode
///   - [ ] page_start
///   - [ ] page_end
///   - [ ] pagination
///   - [ ] citation_prefix
///   - [ ] citation_suffix
///   - [ ] citation_intent
pub fn cite(input: &str) -> IResult<&str, InlineContent> {
    // TODO: Parse more properties of citations
    map_res(
        preceded(char('@'), take_while1(|chr: char| chr.is_alphanumeric())),
        |res: &str| -> Result<InlineContent> {
            let target = res.into();
            Ok(InlineContent::Cite(Cite {
                target,
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse a string into a `CiteGroup` node or parenthetical `Cite` node.
///
/// If there is only one citation within square brackets then a parenthetical `Cite` node is
/// returned. Otherwise, the `Cite` nodes are grouped into into a `CiteGroup`.
pub fn cite_group(input: &str) -> IResult<&str, InlineContent> {
    let cite = map_res(
        preceded(char('@'), take_while1(|chr: char| chr.is_alphanumeric())),
        |res: &str| -> Result<InlineContent> {
            let target = res.into();
            Ok(InlineContent::Cite(Cite {
                target,
                ..Default::default()
            }))
        },
    );

    map_res(
        delimited(
            char('['),
            separated_list1(tuple((multispace0, tag(";"), multispace0)), cite),
            char(']'),
        ),
        |items: Vec<InlineContent>| -> Result<InlineContent> {
            if items.len() == 1 {
                Ok(items[0].clone())
            } else {
                Ok(InlineContent::CiteGroup(CiteGroup {
                    items: items
                        .iter()
                        .filter_map(|item| match item {
                            InlineContent::Cite(cite) => Some(cite),
                            _ => None,
                        })
                        .cloned()
                        .collect(),
                    ..Default::default()
                }))
            }
        },
    )(input)
}

/// Parse a string into an `InlineContent` node
///
/// This attempts to follow Pandoc's match parsing as closely as possible
/// (see https://pandoc.org/MANUAL.html#math).
pub fn math(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        delimited(
            // Pandoc: "opening $ must have a non-space character immediately to its right"
            tuple((char('$'), peek(not(multispace1)))),
            take_until("$"),
            // Pandoc: "the closing $ must have a non-space character immediately to its left,
            // and must not be followed immediately by a digit"
            tuple((peek(not(multispace1)), char('$'), peek(not(digit1)))),
        ),
        |res: &str| -> Result<InlineContent> {
            Ok(InlineContent::MathFragment(MathFragment {
                text: res.into(),
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse a string into a `Subscript` node
pub fn subscript(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        delimited(char('~'), take_until("~"), char('~')),
        |res: &str| -> Result<InlineContent> {
            Ok(InlineContent::Subscript(Subscript {
                content: vec![InlineContent::String(res.into())],
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse a string into a `Superscript` node
pub fn superscript(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        delimited(char('^'), take_until("^"), char('^')),
        |res: &str| -> Result<InlineContent> {
            Ok(InlineContent::Superscript(Superscript {
                content: vec![InlineContent::String(res.into())],
                ..Default::default()
            }))
        },
    )(input)
}

/// Accumulate characters into a `String` node
///
/// Will greedily take as many characters as possible, excluding those that appear at the
/// start of other inline parsers e.g. '$', '['
fn string(input: &str) -> IResult<&str, InlineContent> {
    const CHARS: &str = "@^~$[";
    map_res(
        take_while1(|chr: char| CHARS.contains(chr)),
        |res: &str| -> Result<InlineContent> { Ok(InlineContent::String(String::from(res))) },
    )(input)
}

/// Take a single character into a `String` node
///
/// Necessary so that the characters no consumed by `string` are not lost.
fn character(input: &str) -> IResult<&str, InlineContent> {
    map_res(take(1usize), |res: &str| -> Result<InlineContent> {
        Ok(InlineContent::String(String::from(res)))
    })(input)
}

/// Stores and parses HTML content
///
/// Simply accumulates HTML until tags balance, at which point the HTML is parsed,
/// with text content being parsed as Markdown by calling back to `decode_fragment`.
struct Html {
    html: String,
    tags: Vec<String>,
}

impl Html {
    /// Handle a HTML tag by either storing it or, if it balances previous tags, by
    /// returning accumulated HTML for parsing
    fn handle_html(&mut self, html: &str) -> Vec<BlockContent> {
        // Regex to match tags at the start of the HTML
        static START_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"^<(/?)(\w+)[^/>]*?(/?)>"#).expect("Unable to create regex"));
        static END_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"<(/?)(\w+)[^/>]*?(/?)>\s*$"#).expect("Unable to create regex")
        });

        let start = START_REGEX.captures(&html);
        let end = END_REGEX.captures(&html);

        // Get opening and closing tags (if any)
        let opens = if let Some(start) = start {
            if start.get(1).unwrap().as_str() == "" && start.get(3).unwrap().as_str() == "" {
                Some(start.get(2).unwrap().as_str().to_string())
            } else {
                None
            }
        } else {
            None
        };
        let closes = if let Some(end) = end {
            let tag = end.get(2).unwrap().as_str();
            if end.get(1).unwrap().as_str() == "/"
                || end.get(3).unwrap().as_str() == "/"
                || [
                    // "Self-closing" elements (that can not have child nodes)
                    // https://developer.mozilla.org/en-US/docs/Glossary/Empty_element
                    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link",
                    "meta", "param", "source", "track", "wbr",
                ]
                .contains(&tag)
            {
                Some(tag.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Update tags
        match (opens, closes) {
            (Some(opens), Some(closes)) => {
                if opens != closes {
                    self.tags.push(opens)
                }
            }
            (Some(open), None) => self.tags.push(open),
            (None, Some(close)) => {
                if let Some(last) = self.tags.last() {
                    if *last == close {
                        self.tags.pop();
                    }
                }
            }
            (None, None) => {}
        }

        if self.tags.is_empty() {
            let html = self.html.clone() + html;
            self.html.clear();
            html::decode_fragment(
                &html,
                html::Options {
                    decode_markdown: true,
                },
            )
        } else {
            self.html.push_str(html);
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn md_frontmatter() -> Result<()> {
        assert!(decode_frontmatter("")?.is_none());
        assert!(decode_frontmatter("--")?.is_none());
        assert!(decode_frontmatter("---")?.is_none());
        assert!(decode_frontmatter("---\n---\n")?.is_none());

        let (end, node) = decode_frontmatter("---\ntitle: The title\n---")?.unwrap();
        assert!(end == 24);
        if let Node::Article(_) = node {
        } else {
            bail!("Expected an article")
        }

        Ok(())
    }

    #[test]
    fn md_articles() {
        snapshot_content("articles/*.md", |content| {
            assert_json_snapshot!(decode(&content).expect("Unable to decode Markdown"));
        });
    }

    #[test]
    fn md_fragments() {
        snapshot_content("fragments/md/*.md", |content| {
            assert_json_snapshot!(decode_fragment(&content));
        });
    }
}
