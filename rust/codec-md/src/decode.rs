use std::{
    collections::{HashMap, VecDeque},
    vec,
};

use nom::{
    branch::alt,
    bytes::complete::{
        escaped, is_not, tag, tag_no_case, take, take_until, take_while1, take_while_m_n,
    },
    character::{
        complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1, none_of},
        is_digit,
    },
    combinator::{all_consuming, map, map_res, not, opt, peek, recognize},
    multi::{fold_many0, many0, many1, many_m_n, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

use codec::{
    common::{
        eyre::{bail, Result},
        inflector::Inflector,
        json5,
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
        ..Default::default()
    };

    let mut html = Html::default();

    let mut lists = Lists::default();

    let mut tables = Tables::default();

    let mut blocks = Blocks::default();

    let mut divs = VecDeque::new();

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
                    let content = blocks.pop_mark();
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

                    let items = lists.pop_mark();

                    blocks.push_node(BlockContent::List(List {
                        items,
                        order,

                        ..Default::default()
                    }))
                }
                Tag::Item => {
                    let mut content = Vec::new();

                    let inlines = inlines.pop_mark();
                    if !inlines.is_empty() {
                        content.push(BlockContent::Paragraph(Paragraph {
                            content: inlines,
                            ..Default::default()
                        }))
                    }

                    let mut blocks = blocks.pop_mark();
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
                    let inlines = inlines.pop_mark();
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
                    let trimmed = inlines.text.trim();
                    let block = if trimmed.starts_with("$$") && trimmed.ends_with("$$") {
                        Some(BlockContent::MathBlock(MathBlock {
                            text: trimmed[2..trimmed.len() - 2].trim().to_string(),
                            math_language: "tex".to_string(),
                            ..Default::default()
                        }))
                    } else if let Ok((.., include)) = include(trimmed) {
                        Some(BlockContent::Include(include))
                    } else if let Ok((.., call)) = call(trimmed) {
                        Some(BlockContent::Call(call))
                    } else if let Ok((.., for_)) = for_(trimmed) {
                        blocks.push_div();
                        divs.push_back(BlockContent::For(for_));
                        None
                    } else if let Ok((.., form)) = form(trimmed) {
                        blocks.push_div();
                        divs.push_back(BlockContent::Form(form));
                        None
                    } else if let Ok((.., (true, if_clause))) = if_elif(trimmed) {
                        blocks.push_div();
                        divs.push_back(BlockContent::If(If {
                            clauses: vec![if_clause],
                            ..Default::default()
                        }));
                        None
                    } else if let Ok((.., (false, if_clause))) = if_elif(trimmed) {
                        if let Some(BlockContent::If(if_)) = divs.back_mut() {
                            let content = blocks.pop_div();
                            if let Some(last) = if_.clauses.last_mut() {
                                last.content = content;
                            } else {
                                tracing::error!(
                                    "Expected there to be at least one if clause already"
                                )
                            }
                            if_.clauses.push(if_clause);

                            blocks.push_div();
                            None
                        } else {
                            tracing::warn!("Found an `::: elif` without a preceding `::: if`");
                            Some(BlockContent::Paragraph(Paragraph {
                                content: vec![InlineContent::String(trimmed.to_string())],
                                ..Default::default()
                            }))
                        }
                    } else if let Ok(..) = else_(trimmed) {
                        if let Some(div) = divs.back_mut() {
                            match div {
                                // Create a placeholder to indicate that when the else finishes
                                // the tail of blocks should be popped to the `otherwise` of the current
                                // `For`
                                BlockContent::For(for_) => {
                                    for_.otherwise = Some(Vec::new());
                                }
                                // Add a last clause of `If` with no text or language
                                BlockContent::If(if_) => {
                                    let content = blocks.pop_div();
                                    if let Some(last) = if_.clauses.last_mut() {
                                        last.content = content;
                                    } else {
                                        tracing::error!(
                                            "Expected there to be at least one if clause already"
                                        )
                                    }
                                    if_.clauses.push(IfClause::default());
                                }
                                _ => {
                                    tracing::warn!("Found an `::: else` without a preceding `::: if` or `::: for`");
                                }
                            }
                        }
                        blocks.push_div();
                        None
                    } else if let Ok((.., div)) = division(trimmed) {
                        blocks.push_div();
                        divs.push_back(BlockContent::Division(div));
                        None
                    } else if let Ok(..) = div_end(trimmed) {
                        divs.pop_back().map(|div| match div {
                            BlockContent::Division(mut div) => {
                                div.content = blocks.pop_div();
                                BlockContent::Division(div)
                            }
                            BlockContent::For(mut for_) => {
                                for_.otherwise = for_.otherwise.map(|_| blocks.pop_div());
                                for_.content = blocks.pop_div();
                                BlockContent::For(for_)
                            }
                            BlockContent::Form(mut form) => {
                                form.content = blocks.pop_div();
                                BlockContent::Form(form)
                            }
                            BlockContent::If(mut if_) => {
                                let content = blocks.pop_div();
                                if let Some(last_clause) = if_.clauses.iter_mut().last() {
                                    last_clause.content = content;
                                } else {
                                    tracing::error!(
                                        "Expected at least one if clause but there was none"
                                    );
                                }

                                BlockContent::If(if_)
                            }
                            _ => BlockContent::Paragraph(Paragraph {
                                content: inlines.pop_all(),
                                ..Paragraph::default()
                            }),
                        })
                    } else {
                        Some(BlockContent::Paragraph(Paragraph {
                            content: inlines.pop_all(),
                            ..Default::default()
                        }))
                    };
                    if let Some(block) = block {
                        blocks.push_node(block);
                    }
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
                            Some("asciimath") | Some("mathml") | Some("latex") | Some("tex") => {
                                BlockContent::MathBlock(MathBlock {
                                    text,
                                    math_language: lang.unwrap_or_default(),
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
                    let content = inlines.pop_mark();
                    inlines.push_node(InlineContent::Emphasis(Emphasis {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strong => {
                    let content = inlines.pop_mark();
                    inlines.push_node(InlineContent::Strong(Strong {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strikethrough => {
                    let content = inlines.pop_mark();
                    inlines.push_node(InlineContent::Strikeout(Strikeout {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Link(_link_type, url, title) => {
                    let content = inlines.pop_mark();
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
                    let caption = inlines.pop_mark();
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
                inlines.push_text(&["`", &value, "`"].concat())
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
                let mut content = html.handle_html(&content);
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
#[derive(Default)]
struct Blocks {
    nodes: Vec<BlockContent>,
    marks: Vec<usize>,
    divs: Vec<usize>,
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
    fn pop_mark(&mut self) -> Vec<BlockContent> {
        match self.marks.pop() {
            Some(n) => self.nodes.split_off(n),
            None => Vec::new(),
        }
    }

    /// Push a div marker
    fn push_div(&mut self) {
        self.divs.push(self.nodes.len())
    }

    /// Pop the nodes since the last div marker
    fn pop_div(&mut self) -> Vec<BlockContent> {
        match self.divs.pop() {
            Some(n) => self.nodes.split_off(n),
            None => Vec::new(),
        }
    }

    /// Pop all the nodes
    fn pop_all(&mut self) -> Vec<BlockContent> {
        self.nodes.split_off(0)
    }
}

/// Stores list items
///
/// It is necessary to maintain marks to handle nested lists
#[derive(Default)]
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
    fn pop_mark(&mut self) -> Vec<ListItem> {
        if self.marks.is_empty() {
            vec![]
        } else {
            let n = self.marks.pop().expect("Unable to pop marks!");
            self.items.split_off(n)
        }
    }
}

/// Stores table rows and cells
#[derive(Default)]
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
#[derive(Default)]
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
    fn pop_mark(&mut self) -> Vec<InlineContent> {
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
            button,
            code_attrs,
            code_expr,
            cite_group,
            cite,
            math,
            parameter,
            span,
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
                    let lang = attrs.first().and_then(|item| {
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
                    Some("asciimath") | Some("mathml") | Some("latex") | Some("tex") => {
                        InlineContent::MathFragment(MathFragment {
                            text,
                            math_language: lang.unwrap_or_default(),
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

/// Parse a `Span`.
pub fn span(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        tuple((
            delimited(char('['), is_not("]"), char(']')),
            delimited(char('`'), is_not("`"), char('`')),
            opt(delimited(char('{'), is_not("}"), char('}'))),
        )),
        |(content, text, lang): (&str, &str, Option<&str>)| -> Result<InlineContent> {
            Ok(InlineContent::Span(Span {
                programming_language: lang.map_or_else(String::new, String::from),
                guess_language: lang.is_none().then_some(true),
                text: text.to_string(),
                content: vec![InlineContent::String(content.to_string())],
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse a `Parameter`.
pub fn parameter(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        pair(
            delimited(tag("&["), opt(symbol), char(']')),
            opt(curly_attrs),
        ),
        |(name, attrs)| -> Result<InlineContent> {
            let attrs = attrs.unwrap_or_default();
            let first = attrs
                .first()
                .map(|(name, ..)| Some(Node::String(name.clone())));
            let mut options: HashMap<String, Option<Node>> = attrs.into_iter().collect();

            let typ = options
                .get("type")
                .or(first.as_ref())
                .and_then(|value| value.as_ref())
                .map(|node| node.to_txt());
            let typ = typ.as_deref();

            let label = options
                .remove("label")
                .and_then(|node| node)
                .map(|node| Box::new(node.to_txt()));

            let validator = if matches!(typ, Some("boolean")) || matches!(typ, Some("bool")) {
                Some(ValidatorTypes::BooleanValidator(BooleanValidator::default()))
            } else if matches!(typ, Some("enum")) {
                let values = options
                    .remove("values")
                    .or_else(|| options.remove("vals"))
                    .and_then(|values| values);
                let values = match values {
                    Some(node) => match node {
                        // Usually the supplied node is an array, which we need to convert
                        // to a vector of `Node`s
                        Node::Array(array) => array
                            .into_iter()
                            .map(|primitive| primitive.to_node())
                            .collect(),
                        _ => vec![node],
                    },
                    None => vec![],
                };
                Some(ValidatorTypes::EnumValidator(EnumValidator {
                    values,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("integer")) || matches!(typ, Some("int")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let exclusive_minimum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("exmin"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let exclusive_maximum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("exmax"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let multiple_of = options
                    .remove("multiple_of")
                    .or_else(|| options.remove("mult"))
                    .or_else(|| options.remove("step"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                Some(ValidatorTypes::IntegerValidator(IntegerValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("number")) || matches!(typ, Some("num")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let exclusive_minimum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("exmin"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let exclusive_maximum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("exmax"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                let multiple_of = options
                    .remove("multiple_of")
                    .or_else(|| options.remove("mult"))
                    .and_then(|node| node)
                    .and_then(node_to_option_number);
                Some(ValidatorTypes::NumberValidator(NumberValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("string")) || matches!(typ, Some("str")) {
                let min_length = options
                    .remove("min_length")
                    .or_else(|| options.remove("minlength"))
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_u32);
                let max_length = options
                    .remove("max_length")
                    .or_else(|| options.remove("maxlength"))
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_u32);
                let pattern = options
                    .remove("pattern")
                    .or_else(|| options.remove("regex"))
                    .and_then(|value| value)
                    .map(|node| match node {
                        Node::String(string) => Box::new(string),
                        _ => Box::new(node.to_txt()),
                    });
                Some(ValidatorTypes::StringValidator(StringValidator {
                    min_length,
                    max_length,
                    pattern,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("date")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_date);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_date);
                Some(ValidatorTypes::DateValidator(DateValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("time")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_time);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_time);
                Some(ValidatorTypes::TimeValidator(TimeValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("datetime")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_datetime);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_datetime);
                Some(ValidatorTypes::DateTimeValidator(DateTimeValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("timestamp")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_timestamp);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_timestamp);
                Some(ValidatorTypes::TimestampValidator(TimestampValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("duration")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .and_then(|node| node)
                    .and_then(node_to_option_duration);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .and_then(|node| node)
                    .and_then(node_to_option_duration);
                Some(ValidatorTypes::DurationValidator(DurationValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else {
                None
            }
            .map(Box::new);

            let derived_from = options
                .remove("derived-from")
                .or_else(|| options.remove("from"))
                .and_then(|value| value)
                .and_then(node_to_option_string)
                .map(Box::new);

            let name = name
                .or_else(|| {
                    derived_from
                        .clone()
                        .map(|from| from.split('.').last().unwrap_or(from.as_str()).to_string())
                })
                .unwrap_or_else(|| "unnamed".to_string());

            let default = options
                .remove("default")
                .or_else(|| options.remove("def"))
                .and_then(|value| value)
                .map(Box::new);

            let value = options
                .remove("value")
                .or_else(|| options.remove("val"))
                .and_then(|value| value)
                .map(Box::new);

            Ok(InlineContent::Parameter(Parameter {
                name,
                label,
                validator,
                default,
                value,
                derived_from,
                ..Default::default()
            }))
        },
    )(input)
}

/// Parse a `Button`
pub fn button(input: &str) -> IResult<&str, InlineContent> {
    map_res(
        pair(
            delimited(tag("#["), is_not("]"), char(']')),
            opt(curly_attrs),
        ),
        |(label, options)| -> Result<InlineContent> {
            let mut options: HashMap<String, Option<Node>> =
                options.unwrap_or_default().into_iter().collect();

            let name = options
                .remove("name")
                .and_then(|value| value)
                .and_then(node_to_option_string)
                .unwrap_or_else(|| label.to_snake_case());

            Ok(InlineContent::Button(Button {
                name,
                label: Some(Box::new(label.to_string())),
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
                    attrs.first().map(|item| item.to_string())
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
                math_language: "tex".to_string(),
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

/// Parse attributes inside curly braces
///
/// Curly braced attributes are used to specify options on various inline
/// attributes.
///
/// This is lenient to the form of attributes and consumes everything
/// until the closing bracket. Attribute names are converted to snake_case
/// (so that users don't have to remember which case to use).
fn curly_attrs(input: &str) -> IResult<&str, Vec<(String, Option<Node>)>> {
    alt((
        map(tag("{}"), |_| Vec::new()),
        delimited(
            terminated(char('{'), multispace0),
            separated_list0(multispace1, curly_attr),
            preceded(multispace0, char('}')),
        ),
    ))(input)
}

/// Parse an attribute inside a curly braced attributes into a string/node pair
///
/// Attributes can be single values (i.e. flags) or key-value pairs (separated
/// by `=` or `:`).
fn curly_attr(input: &str) -> IResult<&str, (String, Option<Node>)> {
    map_res(
        alt((
            map(
                tuple((
                    is_not(" =:}"),
                    tuple((multispace0, alt((tag("="), tag(":"))), multispace0)),
                    alt((primitive_node, unquoted_string_node)),
                )),
                |(name, _s, value)| (name, Some(value)),
            ),
            map(is_not(" =:}"), |name| (name, None)),
        )),
        |(name, value): (&str, Option<Node>)| -> Result<(String, Option<Node>)> {
            // Previously this used snake case by that was problematic for attributes such as "json5"
            // (got converted to "json_5"). Instead, we replace dashes with underscores.
            Ok((name.replace('-', "_"), value))
        },
    )(input)
}

/// Parse a true/false (case-insensitive) into a `Boolean` node
fn boolean_node(input: &str) -> IResult<&str, Node> {
    map_res(
        alt((tag_no_case("true"), tag_no_case("false"))),
        |str: &str| -> Result<Node> { Ok(Node::Boolean(str == "true")) },
    )(input)
}

/// Parse one or more digits into an `Integer` node
fn integer_node(input: &str) -> IResult<&str, Node> {
    map_res(
        // The peek avoids a float input being partially parsed as an integer
        // There is probably a better way/place to do this.
        tuple((opt(tag("-")), digit1, peek(is_not(".")))),
        |(sign, digits, _peek): (Option<&str>, &str, _)| -> Result<Node> {
            Ok(Node::Integer(
                (sign.unwrap_or_default().to_string() + digits).parse()?,
            ))
        },
    )(input)
}

/// Parse one or more digits into an `Number` node
fn number_node(input: &str) -> IResult<&str, Node> {
    map_res(double, |num| -> Result<Node> {
        Ok(Node::Number(Number(num)))
    })(input)
}

/// Parse a single quoted string into a `String` node
fn single_quoted_string_node(input: &str) -> IResult<&str, &str> {
    let escaped = escaped(none_of("\\\'"), '\\', char('\''));
    let empty = tag("");
    delimited(char('\''), alt((escaped, empty)), char('\''))(input)
}

/// Parse a double quoted string into a `String` node
fn double_quoted_string_node(input: &str) -> IResult<&str, &str> {
    let escaped = escaped(none_of("\\\""), '\\', char('"'));
    let empty = tag("");
    delimited(char('"'), alt((escaped, empty)), char('"'))(input)
}

/// Parse characters into string into a `String` node
///
/// Excludes character that may be significant in places that this is used.
fn unquoted_string_node(input: &str) -> IResult<&str, Node> {
    map_res(is_not(" }"), |str: &str| -> Result<Node> {
        Ok(Node::String(str.to_string()))
    })(input)
}

/// Parse a single or double quoted string into a `String` node
fn string_node(input: &str) -> IResult<&str, Node> {
    map_res(
        alt((single_quoted_string_node, double_quoted_string_node)),
        |str: &str| -> Result<Node> { Ok(Node::String(str.to_string())) },
    )(input)
}

/// Parse a YYYY-mm-dd date
fn date_node(input: &str) -> IResult<&str, Node> {
    map_res(
        recognize(tuple((
            take_while_m_n(4, 4, |c| is_digit(c as u8)),
            char('-'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
            char('-'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
        ))),
        |str: &str| -> Result<Node> { Ok(Node::Date(Date::from(str.to_string()))) },
    )(input)
}

/// Parse a HH::MM::SS time
fn time_node(input: &str) -> IResult<&str, Node> {
    map_res(
        recognize(tuple((
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
            char(':'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
            char(':'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
        ))),
        |str: &str| -> Result<Node> { Ok(Node::Time(Time::from(str.to_string()))) },
    )(input)
}

/// Parse a YYYY-mm-ddTHH::MM::SS datetime
fn datetime_node(input: &str) -> IResult<&str, Node> {
    map_res(
        recognize(terminated(
            tuple((date_node, char('T'), time_node)),
            opt(char('Z')),
        )),
        |str: &str| -> Result<Node> { Ok(Node::DateTime(DateTime::from(str.to_string()))) },
    )(input)
}

/// Parse a JSON5-style square bracketed array into an `Array` node
///
/// Inner closing brackets can be escaped.
fn array_node(input: &str) -> IResult<&str, Node> {
    let escaped = escaped(none_of("\\]"), '\\', tag("]"));
    let empty = tag("");
    map_res(
        delimited(tag("["), alt((escaped, empty)), tag("]")),
        |inner: &str| -> Result<Node> { Ok(json5::from_str(&["[", inner, "]"].concat())?) },
    )(input)
}

/// Parse a JSON5-style curly braced object into an `Object` node
///
/// Inner closing braces can be escaped.
fn object_node(input: &str) -> IResult<&str, Node> {
    let escaped = escaped(none_of("\\}"), '\\', tag("}"));
    let empty = tag("");
    map_res(
        delimited(tag("{"), alt((escaped, empty)), tag("}")),
        |inner: &str| -> Result<Node> { Ok(json5::from_str(&["{", inner, "}"].concat())?) },
    )(input)
}

/// Any primitive node
fn primitive_node(input: &str) -> IResult<&str, Node> {
    alt((
        object_node,
        array_node,
        datetime_node,
        date_node,
        time_node,
        string_node,
        integer_node,
        number_node,
        boolean_node,
    ))(input)
}

/// Accumulate characters into a `String` node
///
/// Will greedily take as many characters as possible, excluding those that appear at the
/// start of other inline parsers e.g. '$', '['
fn string(input: &str) -> IResult<&str, InlineContent> {
    const CHARS: &str = "~@#$^&[{`";
    map_res(
        take_while1(|chr: char| !CHARS.contains(chr)),
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

/// Parse a string into an `Include` node
fn include(input: &str) -> IResult<&str, Include> {
    map_res(
        all_consuming(preceded(
            char('/'),
            // Exclude '(' from source to avoid clash with a `Call`
            tuple((is_not("({"), opt(curly_attrs))),
        )),
        |(source, options)| -> Result<Include> {
            let mut options: HashMap<String, _> = options.unwrap_or_default().into_iter().collect();
            Ok(Include {
                source: source.to_string(),
                media_type: options
                    .remove("format")
                    .and_then(|option| option)
                    .map(|node| Box::new(node.to_txt())),
                select: options
                    .remove("select")
                    .and_then(|option| option)
                    .map(|node| Box::new(node.to_txt())),
                execute_auto: options
                    .remove("autorun")
                    .and_then(|option| option)
                    .and_then(|node| node.to_txt().parse().ok()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a string into an `Call` node
fn call(input: &str) -> IResult<&str, Call> {
    map_res(
        all_consuming(preceded(
            char('/'),
            tuple((
                is_not("("),
                delimited(char('('), separated_list0(multispace1, call_arg), char(')')),
                opt(curly_attrs),
            )),
        )),
        |(source, args, options)| -> Result<Call> {
            let mut options: HashMap<String, _> = options.unwrap_or_default().into_iter().collect();
            Ok(Call {
                source: source.to_string(),
                arguments: if args.is_empty() { None } else { Some(args) },
                media_type: options
                    .remove("format")
                    .and_then(|option| option)
                    .map(|node| Box::new(node.to_txt())),
                select: options
                    .remove("select")
                    .and_then(|option| option)
                    .map(|node| Box::new(node.to_txt())),
                execute_auto: options
                    .remove("autorun")
                    .and_then(|option| option)
                    .and_then(|node| node.to_txt().parse().ok()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse an argument inside a set of curly braced arguments.
///
/// Arguments must be key-value or key-symbol pairs separated by `=`.
fn call_arg(input: &str) -> IResult<&str, CallArgument> {
    #[allow(clippy::large_enum_variant)]
    enum CallArgumentValue {
        Node(Node),
        Symbol(String),
    }
    map_res(
        tuple((
            symbol,
            delimited(multispace0, tag("="), multispace0),
            alt((
                map(primitive_node, CallArgumentValue::Node),
                map(symbol, CallArgumentValue::Symbol),
            )),
        )),
        |(name, _delim, node)| -> Result<CallArgument> {
            let (value, symbol) = match node {
                CallArgumentValue::Node(node) => (Some(Box::new(node)), None),
                CallArgumentValue::Symbol(symbol) => (None, Some(Box::new(symbol))),
            };
            Ok(CallArgument {
                name,
                value,
                text: symbol,
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a symbol (e.g. the name of a `Parameter` or `CallArgument`)
///
/// Will only recognize names that are valid (in most programming languages). An alternative is to be more
/// permissive here and to check validity of symbol names elsewhere.
fn symbol(input: &str) -> IResult<&str, String> {
    map_res(
        recognize(tuple((
            many1(alt((alpha1, tag("_")))),
            many0(alt((alphanumeric1, tag("_")))),
        ))),
        |str: &str| -> Result<String> { Ok(str.to_string()) },
    )(input)
}

/// Parse a `Division` node
///
/// Note: This and the following 'div like' parsers are all consuming because they are used
/// to test a match against a whole line.
fn division(input: &str) -> IResult<&str, Division> {
    map_res(
        all_consuming(preceded(
            tuple((semis3plus, multispace0)),
            alt((
                // TODO use similar approach as for if etc of only escaping with backticks if needed
                // and guessing languages
                // TODO allow for divs with no style
                tuple((
                    delimited(char('`'), is_not("`"), char('`')),
                    delimited(char('{'), is_not("}"), char('}')),
                )),
                map(is_not("\r\n"), |text| (text, "tailwind")),
            )),
        )),
        |(text, programming_language)| -> Result<Division> {
            Ok(Division {
                programming_language: programming_language.to_string(),
                text: text.to_string(),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a `For` node
fn for_(input: &str) -> IResult<&str, For> {
    map_res(
        all_consuming(preceded(
            tuple((semis3plus, multispace0, tag("for"), multispace1)),
            tuple((
                separated_pair(
                    symbol,
                    tuple((multispace1, tag("in"), multispace1)),
                    is_not("{"),
                ),
                opt(preceded(
                    multispace0,
                    delimited(char('{'), is_not("}"), char('}')),
                )),
            )),
        )),
        |((symbol, expr), lang)| -> Result<For> {
            Ok(For {
                symbol,
                text: expr.trim().to_string(),
                programming_language: lang.map_or_else(String::new, |lang| lang.trim().to_string()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a `Form` node
fn form(input: &str) -> IResult<&str, Form> {
    map_res(
        all_consuming(preceded(
            tuple((semis3plus, multispace0, tag("form"), multispace0)),
            opt(curly_attrs),
        )),
        |options| -> Result<Form> {
            let mut options: HashMap<_, _> = options.unwrap_or_default().into_iter().collect();

            let derive_from = options
                .remove("from")
                .and_then(|value| value)
                .and_then(node_to_option_string)
                .map(Box::new);

            let derive_action = options
                .remove("action")
                .and_then(|value| value)
                .and_then(node_to_option_string)
                .and_then(|value| match value.to_lowercase().as_str() {
                    "create" => Some(FormDeriveAction::Create),
                    "update" => Some(FormDeriveAction::Update),
                    "delete" => Some(FormDeriveAction::Delete),
                    _ => None,
                });

            let derive_item = options
                .remove("item")
                .and_then(|value| value)
                .and_then(|node| match node {
                    Node::Integer(int) => Some(FormDeriveItem::Integer(int)),
                    Node::String(string) => Some(FormDeriveItem::String(string)),
                    _ => None,
                })
                .map(Box::new);

            Ok(Form {
                derive_from,
                derive_action,
                derive_item,
                ..Default::default()
            })
        },
    )(input)
}

/// Parse an `if` or `elif` section into an `IfClause`
fn if_elif(input: &str) -> IResult<&str, (bool, IfClause)> {
    map_res(
        all_consuming(preceded(
            tuple((semis3plus, multispace0)),
            tuple((
                alt((tag("if"), tag("elif"))),
                alt((
                    preceded(
                        multispace1,
                        delimited(char('`'), escaped(none_of("`"), '\\', char('`')), char('`')),
                    ),
                    preceded(multispace1, is_not("{")),
                    multispace0,
                )),
                opt(curly_attrs),
            )),
        )),
        |(tag, expr, options)| -> Result<(bool, IfClause)> {
            let text = expr.trim().to_string();
            let lang = options
                .iter()
                .flatten()
                .next()
                .map(|tuple| tuple.0.trim().to_string())
                .unwrap_or_default();
            Ok((
                tag == "if",
                IfClause {
                    guess_language: lang.is_empty().then_some(true),
                    programming_language: lang,
                    text,
                    ..Default::default()
                },
            ))
        },
    )(input)
}

/// Parse an `else` section
fn else_(input: &str) -> IResult<&str, &str> {
    all_consuming(recognize(tuple((
        semis3plus,
        multispace0,
        tag("else"),
        // Allow for, but ignore, trailing content
        opt(pair(multispace1, is_not(""))),
    ))))(input)
}

/// Parse the end of a division
fn div_end(input: &str) -> IResult<&str, &str> {
    all_consuming(recognize(tuple((semis3plus, multispace0))))(input)
}

/// Parse at least three semicolons
fn semis3plus(input: &str) -> IResult<&str, &str> {
    recognize(many_m_n(3, 100, char(':')))(input)
}

fn node_to_option_string(node: Node) -> Option<String> {
    match node {
        Node::String(num) => Some(num),
        _ => Some(node.to_txt()),
    }
}
fn node_to_option_number(node: Node) -> Option<Number> {
    match node {
        Node::Number(num) => Some(num),
        Node::Integer(num) => Some(Number(num as f64)),
        _ => node.to_txt().parse().ok(),
    }
}
fn node_to_option_u32(node: Node) -> Option<u32> {
    match node {
        Node::Integer(int) => Some(int as u32),
        _ => node.to_txt().parse().ok(),
    }
}
fn node_to_option_date(node: Node) -> Option<Date> {
    match node {
        Node::Date(date) => Some(date),
        Node::String(string) => Some(Date::from(string)),
        _ => None,
    }
}
fn node_to_option_time(node: Node) -> Option<Time> {
    match node {
        Node::Time(time) => Some(time),
        Node::String(string) => Some(Time::from(string)),
        _ => None,
    }
}
fn node_to_option_datetime(node: Node) -> Option<DateTime> {
    match node {
        Node::DateTime(datetime) => Some(datetime),
        Node::String(string) => Some(DateTime::from(string)),
        _ => None,
    }
}

fn node_to_option_timestamp(node: Node) -> Option<Box<Timestamp>> {
    match node {
        Node::Timestamp(timestamp) => Some(Box::new(timestamp)),
        // TODO Node::DateTime(datetime) => Some(Timestamp::from(datetime)),
        // TODO Node::String(string) => Some(Timestamp::from(string)),
        _ => None,
    }
}

fn node_to_option_duration(node: Node) -> Option<Box<Duration>> {
    match node {
        Node::Duration(duration) => Some(Box::new(duration)),
        // TODO Node::String(string) => Some(Duration::from(string)),
        _ => None,
    }
}

/// Stores and parses HTML content
///
/// Simply accumulates HTML until tags balance, at which point the HTML is parsed,
/// with text content being parsed as Markdown by calling back to `decode_fragment`.
#[derive(Default)]
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
    use test_utils::{assert_json_eq, pretty_assertions::assert_eq};

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
        let (_, res) = single_quoted_string_node(r#"' \' 🤖 '"#).unwrap();
        assert_eq!(res, r#" \' 🤖 "#);
        let (_, res) = single_quoted_string_node("' → x'").unwrap();
        assert_eq!(res, " → x");
        let (_, res) = single_quoted_string_node("'  '").unwrap();
        assert_eq!(res, "  ");
        let (_, res) = single_quoted_string_node("''").unwrap();
        assert_eq!(res, "");
    }

    #[test]
    fn test_double_quoted() {
        let (_, res) = double_quoted_string_node(r#"" \" 🤖 ""#).unwrap();
        assert_eq!(res, r#" \" 🤖 "#);
        let (_, res) = double_quoted_string_node(r#"" → x""#).unwrap();
        assert_eq!(res, " → x");
        let (_, res) = double_quoted_string_node(r#""  ""#).unwrap();
        assert_eq!(res, "  ");
        let (_, res) = double_quoted_string_node(r#""""#).unwrap();
        assert_eq!(res, "");
    }

    #[test]
    fn test_square_bracketed() {
        let (_, res) = array_node("[1,2,3]").unwrap();
        assert_json_eq!(res, [1, 2, 3]);
        let (_, res) = array_node("['a', 'b']").unwrap();
        assert_json_eq!(res, ["a", "b"]);
        let (_, res) = array_node("[\"string \\] with closing bracket\"]").unwrap();
        assert_json_eq!(res, ["string ] with closing bracket"]);
    }

    #[test]
    fn test_curly_attrs() {
        assert_eq!(
            curly_attrs(r#"{a}"#).unwrap().1,
            vec![("a".to_string(), None),]
        );

        assert_json_eq!(
            curly_attrs(r#"{a=1 b='2' c:-3 d = 4.0}"#).unwrap().1,
            vec![
                ("a", Some(Node::Integer(1))),
                ("b", Some(Node::String("2".to_string()))),
                ("c", Some(Node::Integer(-3))),
                ("d", Some(Node::Number(Number(4.0))))
            ]
        );

        assert_json_eq!(
            curly_attrs(r#"{date min=2022-09-01 max='2022-09-30' def="2022-09-15"}"#)
                .unwrap()
                .1,
            vec![
                ("date", None),
                (
                    "min",
                    Some(Node::Date(Date::from("2022-09-01".to_string())))
                ),
                ("max", Some(Node::String("2022-09-30".to_string()))),
                ("def", Some(Node::String("2022-09-15".to_string()))),
            ]
        );

        // Multiple spaces are fine
        assert_json_eq!(
            curly_attrs(r#"{   a     b=21 c : 1.234 d="   Internal  spaces "  }"#)
                .unwrap()
                .1,
            vec![
                ("a", None),
                ("b", Some(Node::Integer(21))),
                ("c", Some(Node::Number(Number(1.234)))),
                ("d", Some(Node::String("   Internal  spaces ".to_string())))
            ]
        );
    }

    #[test]
    fn test_spans() {
        assert_eq!(
            span(r#"[some string content]{text-red-300}"#).unwrap().1,
            InlineContent::Span(Span {
                programming_language: "tailwind".to_string(),
                text: "text-red-300".to_string(),
                content: vec![InlineContent::String("some string content".to_string())],
                ..Default::default()
            })
        );

        assert_eq!(
            span(r#"[content]`f"text-{color}-300"`{python}"#).unwrap().1,
            InlineContent::Span(Span {
                programming_language: "python".to_string(),
                text: "f\"text-{color}-300\"".to_string(),
                content: vec![InlineContent::String("content".to_string())],
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_parameters() {
        assert_eq!(
            parameter(r#"&[name]{}"#).unwrap().1,
            InlineContent::Parameter(Parameter {
                name: "name".to_string(),
                ..Default::default()
            })
        );

        assert_eq!(
            parameter(r#"&[name]{bool}"#).unwrap().1,
            InlineContent::Parameter(Parameter {
                name: "name".to_string(),
                validator: Some(Box::new(ValidatorTypes::BooleanValidator(
                    BooleanValidator::default()
                ))),
                ..Default::default()
            })
        );

        assert_eq!(
            parameter(r#"&[name_with_under7scoresAnd_digits_3]{}"#)
                .unwrap()
                .1,
            InlineContent::Parameter(Parameter {
                name: "name_with_under7scoresAnd_digits_3".to_string(),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_calls() {
        assert_eq!(
            call("/file.md()").unwrap().1,
            Call {
                source: "file.md".to_string(),
                ..Default::default()
            }
        );
        assert_eq!(
            call("/file.md(a=1)").unwrap().1,
            Call {
                source: "file.md".to_string(),
                arguments: Some(vec![CallArgument {
                    name: "a".to_string(),
                    value: Some(Box::new(Node::Integer(1))),
                    ..Default::default()
                }]),
                ..Default::default()
            }
        );
        assert_eq!(
            call(r#"/file.md(parAm_eter_1="string")"#).unwrap().1,
            Call {
                source: "file.md".to_string(),
                arguments: Some(vec![CallArgument {
                    name: "parAm_eter_1".to_string(),
                    value: Some(Box::new(Node::String("string".to_string()))),
                    ..Default::default()
                }]),
                ..Default::default()
            }
        );
        assert_eq!(
            call("/file.md(a=1.23 b=symbol c='string')").unwrap().1,
            Call {
                source: "file.md".to_string(),
                arguments: Some(vec![
                    CallArgument {
                        name: "a".to_string(),
                        value: Some(Box::new(Node::Number(Number(1.23)))),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "b".to_string(),
                        symbol: Some(Box::new("symbol".to_string())),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "c".to_string(),
                        value: Some(Box::new(Node::String("string".to_string()))),
                        ..Default::default()
                    }
                ]),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_for() {
        // Simple
        assert_eq!(
            for_("::: for item in expr").unwrap().1,
            For {
                symbol: "item".to_string(),
                text: "expr".to_string(),
                ..Default::default()
            }
        );

        // With less/extra spacing
        assert_eq!(
            for_(":::for item  in    expr").unwrap().1,
            For {
                symbol: "item".to_string(),
                text: "expr".to_string(),
                ..Default::default()
            }
        );

        // With language specified
        assert_eq!(
            for_("::: for item in expr {python}").unwrap().1,
            For {
                symbol: "item".to_string(),
                text: "expr".to_string(),
                programming_language: "python".to_string(),
                ..Default::default()
            }
        );

        // With more complex expression
        assert_eq!(
            for_("::: for i in 1:10").unwrap().1,
            For {
                symbol: "i".to_string(),
                text: "1:10".to_string(),
                ..Default::default()
            }
        );
        assert_eq!(
            for_("::: for row in select * from table { sql }")
                .unwrap()
                .1,
            For {
                symbol: "row".to_string(),
                text: "select * from table".to_string(),
                programming_language: "sql".to_string(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_form() {
        assert_eq!(form("::: form").unwrap().1, Form::default());
    }

    #[test]
    fn test_if() {
        // Simple
        assert_eq!(
            if_elif_else("::: if expr").unwrap().1 .1,
            If {
                text: "expr".to_string(),
                ..Default::default()
            }
        );

        // With less/extra spacing
        assert_eq!(
            if_elif_else(":::if    expr").unwrap().1 .1,
            If {
                text: "expr".to_string(),
                ..Default::default()
            }
        );

        // With language specified
        assert_eq!(
            if_elif_else("::: if expr {python}").unwrap().1 .1,
            If {
                text: "expr".to_string(),
                programming_language: "python".to_string(),
                ..Default::default()
            }
        );

        // With more complex expression
        assert_eq!(
            if_elif_else("::: if a > 1 and b[8] < 1.23").unwrap().1 .1,
            If {
                text: "a > 1 and b[8] < 1.23".to_string(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_divend() {
        assert!(div_end(":::").is_ok());
        assert!(div_end("::::").is_ok());
        assert!(div_end("::::::").is_ok());

        assert!(div_end(":::some chars").is_err());
        assert!(div_end("::").is_err());
        assert!(div_end(":").is_err());
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
