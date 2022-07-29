use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take, take_till, take_until, take_while1},
    character::complete::{alphanumeric1, char, digit1, multispace0, multispace1, none_of},
    combinator::{map, map_res, not, opt, peek},
    multi::{fold_many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

use codec::{
    common::{
        eyre::{bail, Result},
        inflector::Inflector,
        once_cell::sync::Lazy,
        regex::Regex,
        serde_json, serde_yaml, tracing,
    },
    stencila_schema::*,
};
use codec_txt::ToTxt;
use formats::{match_path, FormatNodeType};
use node_coerce::coerce;
use node_transform::Transform;

/// Decode a Markdown document to a `Node`
///
/// Intended for decoding an entire document, this function extracts
/// YAML front matter, parses the Markdown, and returns a `Node::Article` variant.
pub fn decode(md: &str) -> Result<Node> {
    let (end, node) = decode_frontmatter(md)?;

    let md = match end {
        Some(end) => &md[end..],
        None => md,
    };

    let mut node = match node {
        Some(node) => node,
        None => Node::Article(Article::default()),
    };

    let content = decode_fragment(md, None);
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
/// Any front matter will be coerced into an `Node`, defaulting to the
/// `Node::Article` variant, if `type` is not defined. This allows
/// properties such as `authors` to be coerced properly.
///
/// If there is no front matter detected, will return `None`.
pub fn decode_frontmatter(md: &str) -> Result<(Option<usize>, Option<Node>)> {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new("^-{3,}((.|\\n)*)?\\n-{3,}").expect("Unable to create regex"));

    if let Some(captures) = REGEX.captures(md) {
        let end = Some(captures[0].len());

        let yaml = captures[1].trim().to_string();
        if yaml.is_empty() {
            return Ok((end, None));
        }

        let node = match serde_yaml::from_str(&yaml) {
            Ok(serde_json::Value::Object(mut node)) => {
                if node.get("type").is_none() {
                    node.insert(
                        "type".to_string(),
                        serde_json::Value::String("Article".to_string()),
                    );
                }
                serde_json::Value::Object(node)
            }
            Ok(_) => {
                tracing::warn!("YAML frontmatter is not an object, will be ignored");
                return Ok((end, None));
            }
            Err(error) => {
                tracing::warn!(
                    "Error while parsing YAML frontmatter (will be ignored): {}",
                    error
                );
                return Ok((end, None));
            }
        };

        let node = coerce(node, None)?;
        Ok((end, Some(node)))
    } else {
        Ok((None, None))
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
///
/// # Arguments
///
/// - `default_lang`: The default programming language to use on executable code
///                   nodes e.g. `CodeExpression` which do not explicitly se a language.
pub fn decode_fragment(md: &str, default_lang: Option<String>) -> Vec<BlockContent> {
    let mut inlines = Inlines {
        default_lang: default_lang.clone(),
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

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    // Not enabled because currently not handled
    // options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    // Not enabled as messes with single or double quoting values in `curly_attrs`
    // options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(md, options);
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                // Block nodes with block content or special handling
                // (these should all pop the mark when they end)
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
                // (these should all pop the mark when they end)
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
                        Some(ListOrder::Ascending)
                    } else {
                        Some(ListOrder::Unordered)
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
                Tag::Heading(depth, id, _classes) => {
                    blocks.push_node(BlockContent::Heading(Heading {
                        id: id.map(|id| Box::new(id.to_string())),
                        depth: Some(depth as u8),
                        content: inlines.pop_all(),
                        ..Default::default()
                    }))
                }
                Tag::Paragraph => {
                    let node = if inlines.text.starts_with("$$") && inlines.text.ends_with("$$") {
                        BlockContent::MathBlock(MathBlock {
                            text: inlines.text[2..inlines.text.len() - 2].trim().to_string(),
                            math_language: Some(Box::new("tex".to_string())),
                            ..Default::default()
                        })
                    } else {
                        BlockContent::Paragraph(Paragraph {
                            content: inlines.pop_all(),
                            ..Default::default()
                        })
                    };
                    blocks.push_node(node);
                }
                Tag::CodeBlock(kind) => {
                    let (mut lang, exec) = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            let lang = lang.to_string();
                            if !lang.is_empty() {
                                let (lang, exec) = if let Some(lang) = lang.strip_suffix("exec") {
                                    (lang.trim().to_string(), true)
                                } else {
                                    (lang.to_string(), false)
                                };
                                (Some(lang), exec)
                            } else {
                                (None, false)
                            }
                        }
                        _ => (None, false),
                    };

                    // Apply default lang for executable code only
                    if exec && lang.is_none() && default_lang.is_some() {
                        lang = default_lang.clone()
                    }

                    let text = inlines.pop_text().trim_end_matches('\n').to_string();

                    let node = match exec {
                        true => BlockContent::CodeChunk(CodeChunk {
                            text,
                            programming_language: lang.unwrap_or_default(),
                            ..Default::default()
                        }),
                        false => match lang.as_deref() {
                            Some("asciimath") | Some("latex") | Some("tex") => {
                                BlockContent::MathBlock(MathBlock {
                                    text,
                                    math_language: lang.map(Box::new),
                                    ..Default::default()
                                })
                            }
                            _ => BlockContent::CodeBlock(CodeBlock {
                                text,
                                programming_language: lang.map(Box::new),
                                ..Default::default()
                            }),
                        },
                    };

                    blocks.push_node(node)
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
                    inlines.push_node(InlineContent::Strikeout(Strikeout {
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
                    let caption = inlines.pop_tail();
                    let caption = if caption.is_empty() {
                        None
                    } else {
                        // Content is reduced to a string. Media object do not often have other, more
                        // complicated, Markdown content in any case.
                        let txt = caption.to_txt();
                        Some(Box::new(txt))
                    };

                    let title = {
                        let title = title.to_string();
                        if !title.is_empty() {
                            Some(Box::new(CreativeWorkTitle::String(title)))
                        } else {
                            None
                        }
                    };

                    let media_object = match match_path(&url.to_string()).spec().node_type {
                        FormatNodeType::AudioObject => {
                            InlineContent::AudioObject(AudioObjectSimple {
                                caption,
                                content_url: url.to_string(),
                                title,
                                ..Default::default()
                            })
                        }
                        FormatNodeType::VideoObject => {
                            InlineContent::VideoObject(VideoObjectSimple {
                                caption,
                                content_url: url.to_string(),
                                title,
                                ..Default::default()
                            })
                        }
                        _ => InlineContent::ImageObject(ImageObjectSimple {
                            caption,
                            content_url: url.to_string(),
                            title,
                            ..Default::default()
                        }),
                    };

                    inlines.push_node(media_object)
                }

                Tag::FootnoteDefinition(..) => {
                    // TODO: Handle footnote definitions
                    tracing::debug!("Markdown footnote definitions are not yet handled")
                }
            },
            Event::Code(value) => {
                // Because we allow for attributes on code, we push back the
                // code in back ticks for it to be parsed again later.
                inlines.push_text(&["`", &value.to_string(), "`"].concat())
            }
            Event::Rule => blocks.push_node(BlockContent::ThematicBreak(ThematicBreak {
                ..Default::default()
            })),
            Event::Text(value) => {
                // Text gets accumulated to HTML when we're inside a tag, to inlines otherwise
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
                        inlines.append_nodes(&mut content.to_inlines())
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
        let mut cells = self.cells.split_off(0);
        cells
            .iter_mut()
            .for_each(|cell| cell.cell_type = Some(TableCellCellType::Header));
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
    default_lang: Option<String>,
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
                Ok((_, mut inlines)) => {
                    // Set the programming language on code expressions if necessary
                    if let Some(default_lang) = self.default_lang.as_ref() {
                        for node in inlines.iter_mut() {
                            if let InlineContent::CodeExpression(expr) = node {
                                if expr.programming_language.is_empty() {
                                    expr.programming_language = default_lang.clone()
                                }
                            }
                        }
                    }
                    inlines
                }
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
        self.marks.push(self.nodes.len());
        self.active = true;
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
            code_attrs,
            code_expr,
            cite_group,
            cite,
            math,
            parameter,
            subscript,
            superscript,
            string,
            character,
        )),
        Vec::new,
        |mut vec: Vec<InlineContent>, node| {
            if let InlineContent::String(string) = &node {
                match vec.last_mut() {
                    Some(InlineContent::String(last)) => last.push_str(string),
                    _ => vec.push(node),
                }
            } else {
                vec.push(node)
            }
            vec
        },
    )(input)
}

/// Parse inline code with attributes in curly braces
/// e.g. `\`code\`{attr1 attr2}` into a `CodeFragment`, `CodeExpression`
/// or `MathFragment` node
pub fn code_attrs(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        pair(
            delimited(char('`'), take_until("`"), char('`')),
            opt(delimited(char('{'), take_until("}"), char('}'))),
        ),
        |res: (&str, Option<&str>)| -> Result<InlineContent> {
            let text = res.0.to_string();
            let (lang, exec) = match res.1 {
                Some(attrs) => {
                    let attrs = attrs.split_whitespace().collect::<Vec<&str>>();
                    let lang = attrs.get(0).and_then(|item| {
                        if *item == "exec" {
                            None
                        } else {
                            Some(item.to_string())
                        }
                    });
                    let exec = attrs.contains(&"exec");
                    (lang, exec)
                }
                None => (None, false),
            };
            let node = match exec {
                true => InlineContent::CodeExpression(CodeExpression {
                    text,
                    programming_language: lang.unwrap_or_default(),
                    ..Default::default()
                }),
                _ => match lang.as_deref() {
                    Some("asciimath") | Some("latex") | Some("tex") => {
                        InlineContent::MathFragment(MathFragment {
                            text,
                            math_language: lang.map(Box::new),
                            ..Default::default()
                        })
                    }
                    _ => InlineContent::CodeFragment(CodeFragment {
                        text,
                        programming_language: lang.map(Box::new),
                        ..Default::default()
                    }),
                },
            };
            Ok(node)
        },
    )(input)
}

/// Parse forward slash pairs into a `Parameter`.
pub fn parameter(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        pair(delimited(char('/'), alphanumeric1, char('/')), curly_attrs),
        |(name, pairs)| -> Result<InlineContent> {
            let options: HashMap<String, Option<String>> = pairs.into_iter().collect();

            let typ = options
                .get("type")
                .and_then(|value| value.as_ref())
                .map(|value| value.as_str());

            let validator = if matches!(typ, Some("boolean"))
                || matches!(typ, Some("bool"))
                || options.get("boolean").is_some()
                || options.get("bool").is_some()
            {
                Some(ValidatorTypes::BooleanValidator(BooleanValidator::default()))
            } else if matches!(typ, Some("integer"))
                || matches!(typ, Some("int"))
                || options.get("integer").is_some()
                || options.get("int").is_some()
            {
                // TODO: Add properties to `IntegerValidator`
                Some(ValidatorTypes::IntegerValidator(IntegerValidator::default()))
            } else if matches!(typ, Some("number"))
                || matches!(typ, Some("num"))
                || options.get("number").is_some()
                || options.get("num").is_some()
            {
                let minimum = options
                    .get("minimum")
                    .or_else(|| options.get("min"))
                    .and_then(|value| value.as_ref())
                    .and_then(|value| value.parse().ok());
                let maximum = options
                    .get("maximum")
                    .or_else(|| options.get("max"))
                    .and_then(|value| value.as_ref())
                    .and_then(|value| value.parse().ok());
                let multiple_of = options
                    .get("multiple_of")
                    .or_else(|| options.get("step"))
                    .and_then(|value| value.as_ref())
                    .and_then(|value| value.parse().ok());
                Some(ValidatorTypes::NumberValidator(NumberValidator {
                    minimum,
                    maximum,
                    multiple_of,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("string"))
                || matches!(typ, Some("str"))
                || options.get("string").is_some()
                || options.get("str").is_some()
            {
                let min_length = options
                    .get("min_length")
                    .or_else(|| options.get("minlength"))
                    .and_then(|value| value.as_ref())
                    .and_then(|value| value.parse().ok());
                let max_length = options
                    .get("max_length")
                    .or_else(|| options.get("maxlength"))
                    .and_then(|value| value.as_ref())
                    .and_then(|value| value.parse().ok());
                let pattern = options
                    .get("pattern")
                    .or_else(|| options.get("regex"))
                    .and_then(|value| value.as_ref())
                    .map(|value| Box::new(value.clone()));
                Some(ValidatorTypes::StringValidator(StringValidator {
                    min_length,
                    max_length,
                    pattern,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("enum")) || options.get("enum").is_some() {
                let values = options
                    .get("values")
                    .or_else(|| options.get("vals"))
                    .and_then(|value| value.as_ref())
                    .map(|string| {
                        let json = match string.starts_with('[') && string.ends_with('[') {
                            true => string.clone(),
                            false => ["[", string, "]"].concat(),
                        };
                        match json5::from_str::<Vec<Node>>(&json) {
                            Ok(array) => array,
                            Err(..) => string
                                .split(',')
                                .map(|item| Node::String(item.trim().to_string()))
                                .collect(),
                        }
                    });
                Some(ValidatorTypes::EnumValidator(EnumValidator {
                    values,
                    ..Default::default()
                }))
            } else {
                None
            }
            .map(Box::new);

            let default = options
                .get("default")
                .and_then(|value| value.as_ref())
                .map(|string| {
                    json5::from_str::<Node>(string).unwrap_or_else(|_| Node::String(string.clone()))
                })
                .map(Box::new);

            let value = options
                .get("value")
                .and_then(|value| value.as_ref())
                .map(|string| {
                    json5::from_str::<Node>(string).unwrap_or_else(|_| Node::String(string.clone()))
                })
                .map(Box::new);

            Ok(InlineContent::Parameter(Parameter {
                name: name.into(),
                validator,
                default,
                value,
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse double brace surrounded text into a `CodeExpression`.
///
/// This supports the Jupyter "Python Markdown" extension syntax for
/// interpolated variables / expressions: `{{x}}`
///
/// Does not support the single curly brace syntax (as in Python, Rust and JSX) i.e. `{x}`
/// given that is less specific and could conflict with other user content.
///
/// Does not support JavaScript style "dollared-brace" syntax i.e. `${x}` since some
/// at least some Markdown parsers seem to parse that as TeX math (even though there
/// is no closing brace).
///
/// The language of the code expression can be added in a curly brace suffix.
/// e.g. `{{2 * 2}}{r}` is equivalent to `\`r 2 * 2\``{r exec} in Markdown or to
/// `\`r 2 * 2\` in R Markdown.
pub fn code_expr(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        pair(
            delimited(tag("{{"), take_until("}}"), tag("}}")),
            opt(delimited(char('{'), take_until("}"), char('}'))),
        ),
        |res: (&str, Option<&str>)| -> Result<InlineContent> {
            let text = res.0.to_string();
            let lang = match res.1 {
                Some(attrs) => {
                    let attrs = attrs.split_whitespace().collect::<Vec<&str>>();
                    attrs.get(0).map(|item| item.to_string())
                }
                None => None,
            };
            Ok(InlineContent::CodeExpression(CodeExpression {
                text,
                programming_language: lang.unwrap_or_else(|| "".to_string()),
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse a string into a narrative `Cite` node
///
/// This attempts to follow Pandoc's citation handling as closely as possible
/// (see <https://pandoc.org/MANUAL.html#citations>).
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
/// (see <https://pandoc.org/MANUAL.html#math>).
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
                math_language: Some(Box::new("tex".to_string())),
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse a string into a `Subscript` node
pub fn subscript(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        delimited(
            // Only match single tilde, because doubles are for `Strikeout`
            tuple((char('~'), peek(not(char('~'))))),
            take_until("~"),
            char('~'),
        ),
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

type Attrs = Vec<(String, Option<String>)>;

/// Parse attributes inside curly braces
///
/// Curly braced attributes are used to specify options on various inline
/// attributes.
///
/// This is lenient to the form of attributes and consumes everything
/// until the closing bracket. Attribute names are converted to snake_case
/// (so that users don't have to remember which case to use).
fn curly_attrs(input: &str) -> IResult<&str, Attrs> {
    alt((
        map(tag("{}"), |_| Vec::new()),
        delimited(
            char('{'),
            separated_list0(multispace1, curly_attr),
            char('}'),
        ),
    ))(input)
}

/// Parse an attribute inside a set of curly braced attributes.
///
/// Attributes can be single values (i.e. flags) or key-value pairs (separated
/// by `=` or `:`).
fn curly_attr(input: &str) -> IResult<&str, (String, Option<String>)> {
    map_res(
        tuple((
            take_till(|c| c == ' ' || c == '=' || c == ':'),
            opt(preceded(
                tuple((multispace0, alt((tag("="), tag(":"))), multispace0)),
                alt((
                    single_quoted,
                    double_quoted,
                    square_bracketed,
                    take_till(|c| c == ' ' || c == '}'),
                )),
            )),
        )),
        |(name, value): (&str, Option<&str>)| -> Result<(String, Option<String>)> {
            Ok((name.to_snake_case(), value.map(|value| value.to_string())))
        },
    )(input)
}

/// Parse a single quoted string
fn single_quoted(input: &str) -> IResult<&str, &str> {
    let escaped = escaped(none_of("\\\'"), '\\', tag("'"));
    let empty = tag("");
    delimited(tag("'"), alt((escaped, empty)), tag("'"))(input)
}

/// Parse a double quoted string
fn double_quoted(input: &str) -> IResult<&str, &str> {
    let escaped = escaped(none_of("\\\""), '\\', tag("\""));
    let empty = tag("");
    delimited(tag("\""), alt((escaped, empty)), tag("\""))(input)
}

/// Parse a JSON-style square bracketed array (inner closing brackets can be escaped)
/// Does not return the outer brackets
fn square_bracketed(input: &str) -> IResult<&str, &str> {
    let escaped = escaped(none_of("\\]"), '\\', tag("]"));
    let empty = tag("");
    delimited(tag("["), alt((escaped, empty)), tag("]"))(input)
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

        let start = START_REGEX.captures(html);
        let end = END_REGEX.captures(html);

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
            codec_html::decode_fragment(&html, Some(Box::new(|text| decode_fragment(text, None))))
        } else {
            self.html.push_str(html);
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures_content};
    use test_utils::pretty_assertions::assert_eq;

    #[test]
    fn md_frontmatter() -> Result<()> {
        assert!(decode_frontmatter("")?.0.is_none());
        assert!(decode_frontmatter("--")?.0.is_none());
        assert!(decode_frontmatter("---")?.0.is_none());

        let (end, node) = decode_frontmatter("---\n---\n")?;
        assert_eq!(end, Some(7));
        assert!(node.is_none());

        let (end, node) = decode_frontmatter("---\ntitle: The title\n---")?;
        assert!(end == Some(24));
        if let Some(Node::Article(_)) = node {
        } else {
            bail!("Expected an article")
        }

        Ok(())
    }

    #[test]
    fn test_single_quoted() {
        let (_, res) = single_quoted(r#"' \' 🤖 '"#).unwrap();
        assert_eq!(res, r#" \' 🤖 "#);
        let (_, res) = single_quoted("' → x'").unwrap();
        assert_eq!(res, " → x");
        let (_, res) = single_quoted("'  '").unwrap();
        assert_eq!(res, "  ");
        let (_, res) = single_quoted("''").unwrap();
        assert_eq!(res, "");
    }

    #[test]
    fn test_square_bracketed() {
        let (_, res) = square_bracketed("[1,2,3]").unwrap();
        assert_eq!(res, "1,2,3");
        let (_, res) = square_bracketed("['a', 'b', null]").unwrap();
        assert_eq!(res, "'a', 'b', null");
        let (_, res) = square_bracketed("[\\]]").unwrap();
        assert_eq!(res, "\\]");
    }

    #[test]
    fn test_curly_attrs() {
        let res = curly_attrs(r#"{a=1 b='2' c:3 d = 4}"#).unwrap();
        assert_eq!(res.1[0], ("a".to_string(), Some("1".to_string())));
        assert_eq!(res.1[1], ("b".to_string(), Some("2".to_string())));
        assert_eq!(res.1[2], ("c".to_string(), Some("3".to_string())));
        assert_eq!(res.1[3], ("d".to_string(), Some("4".to_string())));
    }

    #[test]
    fn decode_md_articles() {
        snapshot_fixtures_content("articles/*.md", |content| {
            assert_json_snapshot!(decode(content).expect("Unable to decode Markdown"));
        });
    }

    #[test]
    fn decode_md_fragments() {
        snapshot_fixtures_content("fragments/md/*.md", |content| {
            assert_json_snapshot!(decode_fragment(content, None));
        });
    }
}
