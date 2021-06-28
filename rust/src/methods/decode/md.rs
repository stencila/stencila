use eyre::Result;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map_res, not, peek},
    multi::{fold_many0, separated_list1},
    sequence::{delimited, preceded, tuple},
    IResult,
};
use stencila_schema::{
    Article, BlockContent, Cite, CiteGroup, CodeBlock, CodeFragment, Delete, Emphasis, Heading,
    ImageObjectSimple, InlineContent, Link, List, ListItem, ListItemContent, MathFragment, Node,
    Paragraph, QuoteBlock, Strong, Subscript, Superscript, ThematicBreak,
};

/// Decode a Markdown document to a `Node`
///
/// Intended for decoding an entire document, this function extracts
/// YAML front matter, parses the Markdown, and returns a `Node::Article` variant.
pub fn decode(md: &str) -> Result<Node> {
    let content = decode_fragment(md);

    let article = Article {
        content: Some(content),
        ..Default::default()
    };

    Ok(Node::Article(article))
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
    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

    let mut inlines = Inlines {
        text: String::new(),
        nodes: Vec::new(),
        marks: Vec::new(),
    };

    let mut lists = Lists {
        items: Vec::new(),
        marks: Vec::new(),
    };

    let mut blocks = Blocks {
        nodes: Vec::new(),
        marks: Vec::new(),
    };

    let parser = Parser::new_ext(md, Options::all());
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                // Block nodes with block content
                Tag::BlockQuote => blocks.push_mark(),
                Tag::List(_) => lists.push_mark(),
                Tag::Item => {
                    inlines.push_mark();
                    blocks.push_mark()
                }

                // Block nodes with inline content
                Tag::Heading(_) => inlines.clear(),
                Tag::Paragraph => inlines.clear(),
                Tag::CodeBlock(_) => inlines.clear(),

                // Inline nodes with inline content
                Tag::Emphasis => inlines.push_mark(),
                Tag::Strong => inlines.push_mark(),
                Tag::Strikethrough => inlines.push_mark(),
                Tag::Link(_, _, _) => inlines.push_mark(),
                Tag::Image(_, _, _) => inlines.push_mark(),

                // Currently unhandled
                Tag::Table(_) => (),
                Tag::TableHead => (),
                Tag::TableRow => (),
                Tag::TableCell => (),
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
                        Some(Box::new(stencila_schema::ListOrder::Ascending))
                    } else {
                        None
                    };

                    let items = lists.pop_tail();

                    blocks.push_node(BlockContent::List(List {
                        order,
                        items,
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

                    let content = Some(Box::new(ListItemContent::VecBlockContent(content)));

                    lists.push_item(ListItem {
                        content,
                        ..Default::default()
                    })
                }

                // Block nodes with inline content
                Tag::Heading(depth) => blocks.push_node(BlockContent::Heading(Heading {
                    depth: Some(Box::new(depth as i64)),
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

                // Currently unhandled
                Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell
                | Tag::FootnoteDefinition(_) => tracing::warn!("Unhandled Markdown tag {:?}", tag),
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
                // Accumulate to text
                inlines.push_text(&value.to_string())
            }
            Event::SoftBreak => {
                // A soft line break event occurs between lines of a multi-line paragraph
                // (between a `Text` event for each line). This inserts the Unicode soft break
                // character so that, when inlines are decoded a space can be added if
                // necessary.
                inlines.push_text("\u{2029}")
            }
            Event::HardBreak => {
                tracing::warn!("Unhandled Markdown element HardBreak");
            }
            Event::Html(value) => {
                tracing::warn!("Unhandled Markdown element Html {}", value);
            }
            Event::FootnoteReference(value) => {
                tracing::warn!("Unhandled Markdown element FootnoteReference {}", value);
            }
            Event::TaskListMarker(value) => {
                tracing::warn!("Unhandled Markdown element TaskListMarker {}", value);
            }
        };
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
struct Lists {
    items: Vec<ListItem>,
    marks: Vec<usize>,
}

impl Lists {
    /// Push a list item
    fn push_item(&mut self, item: ListItem) {
        self.items.push(item)
    }

    /// Push a mark at the start of a list
    fn push_mark(&mut self) {
        self.marks.push(self.items.len())
    }

    /// Pop the items since the last mark
    fn pop_tail(&mut self) -> Vec<ListItem> {
        let n = self.marks.pop().expect("Unable to pop marks!");
        self.items.split_off(n)
    }
}

/// Stores and parses inline content
struct Inlines {
    text: String,
    nodes: Vec<InlineContent>,
    marks: Vec<usize>,
}

impl Inlines {
    /// Clear all content (usually at the start of a block node)
    fn clear(&mut self) {
        self.text.clear();
        self.nodes.clear();
        self.marks.clear()
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
                    tracing::warn!("While parsing inline content: {}", error);
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

    /// Push a mark (usually at the start of an inline node)
    fn push_mark(&mut self) {
        self.parse_text();
        self.marks.push(self.nodes.len())
    }

    /// Pop the nodes since the last mark
    fn pop_tail(&mut self) -> Vec<InlineContent> {
        self.parse_text();
        let n = self.marks.pop().expect("Unable to pop marks!");
        self.nodes.split_off(n)
    }

    /// Pop all the nodes
    fn pop_all(&mut self) -> Vec<InlineContent> {
        self.parse_text();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn fragments() {
        snapshot_content("fragments/md/*.md", |content| {
            assert_json_snapshot!(decode_fragment(&content));
        });
    }
}
