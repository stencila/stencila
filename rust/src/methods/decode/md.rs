use eyre::Result;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map_res, not, peek},
    multi::{fold_many0, separated_list0},
    sequence::{delimited, preceded, tuple},
    IResult,
};
use stencila_schema::{
    Article, BlockContent, Cite, CiteGroup, CodeBlock, CodeFragment, Delete, Emphasis, Heading,
    InlineContent, Link, MathFragment, Node, Paragraph, Strong, Subscript, Superscript,
    ThematicBreak,
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

    // Holds any text content
    let mut texts = String::new();
    // Holds and manages inline content nodes
    struct Inlines {
        nodes: Vec<InlineContent>,
        marks: Vec<usize>,
    }

    impl Inlines {
        fn clear(&mut self) {
            self.nodes.clear();
            self.marks.clear()
        }

        fn mark(&mut self) {
            self.marks.push(self.nodes.len())
        }

        fn push(&mut self, node: InlineContent) {
            self.nodes.push(node)
        }

        fn append(&mut self, nodes: &mut Vec<InlineContent>) {
            self.nodes.append(nodes)
        }

        fn condense(mut nodes: Vec<InlineContent>) -> Vec<InlineContent> {
            let mut index = 1;
            while index < nodes.len() {
                // FIXME: Avoid this clone?
                let curr = nodes[index].clone();
                let prev = &mut nodes[index - 1];
                match (prev, curr) {
                    (InlineContent::String(prev), InlineContent::String(curr)) => {
                        if prev.ends_with(|chr: char| !chr.is_whitespace()) && curr == "\u{2029}" {
                            prev.push(' ');
                        } else {
                            prev.push_str(&curr);
                        }
                        nodes.remove(index);
                    }
                    _ => index += 1,
                }
            }
            nodes
        }

        fn take_tail(&mut self) -> Vec<InlineContent> {
            let n = self.marks.pop().expect("Unable to pop marks!");
            Inlines::condense(self.nodes.split_off(n))
        }

        fn take_all(&mut self) -> Vec<InlineContent> {
            Inlines::condense(self.nodes.split_off(0))
        }
    }

    let mut inlines = Inlines {
        nodes: Vec::new(),
        marks: Vec::new(),
    };

    // Holds the block content nodes
    let mut blocks: Vec<BlockContent> = Vec::new();

    let parser = Parser::new_ext(md, Options::all());
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading(_) => inlines.clear(),
                Tag::Paragraph => inlines.clear(),
                Tag::CodeBlock(_) => (),
                Tag::Emphasis => inlines.mark(),
                Tag::Strong => inlines.mark(),
                Tag::Strikethrough => inlines.mark(),
                Tag::Link(_, _, _) => inlines.mark(),
                _ => (),
            },
            Event::End(tag) => match tag {
                Tag::Heading(depth) => blocks.push(BlockContent::Heading(Heading {
                    depth: Some(Box::new(depth as i64)),
                    content: inlines.take_all(),
                    ..Default::default()
                })),
                Tag::Paragraph => blocks.push(BlockContent::Paragraph(Paragraph {
                    content: inlines.take_all(),
                    ..Default::default()
                })),
                Tag::CodeBlock(kind) => {
                    let text = texts.clone();
                    texts.clear();
                    blocks.push(BlockContent::CodeBlock(CodeBlock {
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
                Tag::Emphasis => {
                    let content = inlines.take_tail();
                    inlines.push(InlineContent::Emphasis(Emphasis {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strong => {
                    let content = inlines.take_tail();
                    inlines.push(InlineContent::Strong(Strong {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strikethrough => {
                    let content = inlines.take_tail();
                    inlines.push(InlineContent::Delete(Delete {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Link(_link_type, url, title) => {
                    let content = inlines.take_tail();
                    let title = {
                        let title = title.to_string();
                        if !title.is_empty() {
                            Some(Box::new(title))
                        } else {
                            None
                        }
                    };
                    inlines.push(InlineContent::Link(Link {
                        content,
                        target: url.to_string(),
                        title,
                        ..Default::default()
                    }))
                }
                _ => {
                    tracing::warn!("Unhandled Markdown tag {:?}", tag);
                }
            },
            Event::Text(value) => {
                // Accumulate to inline content after parsing
                let mut content = decode_inline_content(&value);
                inlines.append(&mut content);

                // Accumulate to text (needed for indented (unfenced) code blocks)
                texts.push_str(&value.to_string())
            }
            Event::Code(value) => {
                inlines.push(InlineContent::CodeFragment(CodeFragment {
                    text: value.to_string(),
                    ..Default::default()
                }));
            }
            Event::Html(value) => {
                tracing::warn!("Unhandled Markdown element Html {}", value);
            }
            Event::FootnoteReference(value) => {
                tracing::warn!("Unhandled Markdown element FootnoteReference {}", value);
            }
            Event::SoftBreak => {
                // A soft line break event occurs between lines of a multi-line paragraph
                // (between a `Text` event for each line). This inserts the Unicode soft break
                // character so that, when inlines are condensed a space can be added if
                // necessary.
                inlines.push(InlineContent::String("\u{2029}".to_string()))
            }
            Event::HardBreak => {
                tracing::warn!("Unhandled Markdown element HardBreak");
            }
            Event::Rule => blocks.push(BlockContent::ThematicBreak(ThematicBreak {
                ..Default::default()
            })),
            Event::TaskListMarker(value) => {
                tracing::warn!("Unhandled Markdown element TaskListMarker {}", value);
            }
        };
    }

    blocks
}

/// Decode a string into a vector of `InlineContent` nodes
///
/// This is the entry point into `nom` Markdown parsing functions.
/// It is infallible in that if there is a parse error,
/// the original input string is returned as the only item
/// in the vector (with a warning).
fn decode_inline_content(input: &str) -> Vec<InlineContent> {
    match inline_content(input) {
        Ok((_, content)) => content,
        Err(error) => {
            tracing::warn!("While parsing inline content: {}", error);
            vec![InlineContent::String(String::from(input))]
        }
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

/// Parse a string into a `CiteGroup` node
///
/// Simply collects several `Cite` notes into a `CiteGroup`.
pub fn cite_group(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        delimited(
            char('['),
            separated_list0(delimited(multispace0, tag(";"), multispace0), cite),
            char(']'),
        ),
        |items: Vec<InlineContent>| -> Result<InlineContent> {
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
        },
    )(input)
}

/// Parse a string into a `Cite` node
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
    map_res(
        take_while1(|chr: char| chr != '^' && chr != '~' && chr != '$' && chr != '['),
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
