use std::collections::VecDeque;

use codec::{
    common::{
        derive_more::{Deref, DerefMut},
        once_cell::sync::Lazy,
        regex::Regex,
        tracing,
    },
    format::Format,
    schema::{
        shortcuts::{cb, cc, em, mb, ol, p, qb, strike, strong, table, tb, td, text, ul},
        transforms::blocks_to_inlines,
        AudioObject, Block, Heading, If, IfClause, ImageObject, Inline, Link, ListItem, TableCell,
        TableRow, TableRowType, VideoObject,
    },
    Losses,
};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

use super::{
    blocks::{call, division, else_, end, for_, form, if_elif, include, math_block, section},
    inlines::inlines,
};

/// Decode Markdown content to a vector of [`Block`]s
///
/// Intended for decoding a fragment of Markdown (e.g. a Markdown cell in
/// a Jupyter Notebook) and inserting it into a larger document.
pub fn decode_blocks(md: &str) -> (Vec<Block>, Losses) {
    // Set Markdown parsing options
    // The ENABLE_SMART_PUNCTUATION option is not enabled as messes with
    // single or double quoting values in `curly_attrs`.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    // Collections of node types used in Markdown event processing
    let mut inlines = Inlines::default();
    let mut blocks = Blocks::default();
    let mut tables = Tables::default();
    let mut lists = Lists::default();
    let mut divs = Divs::default();
    let mut html = Html::default();

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
                    blocks.push_block(qb(content))
                }
                Tag::List(start) => {
                    let items = lists.pop_mark();
                    blocks.push_block(if start.is_some() {
                        ol(items)
                    } else {
                        ul(items)
                    })
                }
                Tag::Item => {
                    let mut content = Vec::new();

                    let inlines = inlines.pop_mark();
                    if !inlines.is_empty() {
                        content.push(p(inlines))
                    }

                    let mut blocks = blocks.pop_mark();
                    content.append(&mut blocks);

                    lists.push_item(ListItem::new(content))
                }
                Tag::Table(_) => blocks.push_block(table(tables.pop_rows())),
                Tag::TableHead => tables.push_header(),
                Tag::TableRow => tables.push_row(),
                Tag::TableCell => tables.push_cell(td(inlines.pop_mark())),

                // Block nodes with inline content
                Tag::Heading(level, id, _classes) => blocks.push_block(Block::Heading(Heading {
                    id: id.map(|id| id.into()),
                    level: level as i64,
                    content: inlines.pop_all(),
                    ..Default::default()
                })),
                Tag::Paragraph => {
                    let trimmed = inlines.text.trim();

                    // The
                    let block = if let Ok((.., math_block)) = math_block(trimmed) {
                        Some(Block::MathBlock(math_block))
                    } else if let Ok((.., include)) = include(trimmed) {
                        Some(Block::Include(include))
                    } else if let Ok((.., call)) = call(trimmed) {
                        Some(Block::Call(call))
                    } else if let Ok((.., section)) = section(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::Section(section));
                        None
                    } else if let Ok((.., div)) = division(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::Division(div));
                        None
                    } else if let Ok((.., for_)) = for_(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::For(for_));
                        None
                    } else if let Ok((.., form)) = form(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::Form(form));
                        None
                    } else if let Ok((.., (true, if_clause))) = if_elif(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::If(If {
                            clauses: vec![if_clause],
                            ..Default::default()
                        }));
                        None
                    } else if let Ok((.., (false, if_clause))) = if_elif(trimmed) {
                        if let Some(Block::If(if_)) = divs.back_mut() {
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
                            Some(p([text(trimmed)]))
                        }
                    } else if else_(trimmed).is_ok() {
                        if let Some(div) = divs.back_mut() {
                            match div {
                                // Create a placeholder to indicate that when the else finishes
                                // the tail of blocks should be popped to the `otherwise` of the current
                                // `For`
                                Block::For(for_) => {
                                    for_.otherwise = Some(Vec::new());
                                }
                                // Add a last clause of `If` with no text or language
                                Block::If(if_) => {
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
                    } else if end(trimmed).is_ok() {
                        divs.pop_back().map(|div| match div {
                            Block::Section(mut section) => {
                                section.content = blocks.pop_div();
                                Block::Section(section)
                            }
                            Block::Division(mut div) => {
                                div.content = blocks.pop_div();
                                Block::Division(div)
                            }
                            Block::For(mut for_) => {
                                for_.otherwise = for_.otherwise.map(|_| blocks.pop_div());
                                for_.content = blocks.pop_div();
                                Block::For(for_)
                            }
                            Block::Form(mut form) => {
                                form.content = blocks.pop_div();
                                Block::Form(form)
                            }
                            Block::If(mut if_) => {
                                let content = blocks.pop_div();
                                if let Some(last_clause) = if_.clauses.iter_mut().last() {
                                    last_clause.content = content;
                                } else {
                                    tracing::error!(
                                        "Expected at least one if clause but there was none"
                                    );
                                }

                                Block::If(if_)
                            }
                            _ => p(inlines.pop_all()),
                        })
                    } else {
                        Some(p(inlines.pop_all()))
                    };

                    if let Some(block) = block {
                        blocks.push_block(block);
                    }
                }
                Tag::CodeBlock(kind) => {
                    let (lang, exec) = match kind {
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

                    let code = inlines.pop_text();

                    let block = match exec {
                        true => cc(code, lang),
                        false => match lang.as_deref() {
                            Some("asciimath") | Some("mathml") | Some("latex") | Some("tex") => {
                                mb(code, lang.unwrap_or_default())
                            }
                            _ => cb(code, lang),
                        },
                    };

                    blocks.push_block(block)
                }

                // Inline nodes with inline content
                Tag::Emphasis => {
                    let content = inlines.pop_mark();
                    inlines.push_inline(em(content))
                }
                Tag::Strong => {
                    let content = inlines.pop_mark();
                    inlines.push_inline(strong(content))
                }
                Tag::Strikethrough => {
                    let content = inlines.pop_mark();
                    inlines.push_inline(strike(content))
                }
                Tag::Link(_link_type, url, title) => {
                    let content = inlines.pop_mark();
                    let title = {
                        let title = title.to_string();
                        if !title.is_empty() {
                            Some(title)
                        } else {
                            None
                        }
                    };
                    inlines.push_inline(Inline::Link(Link {
                        content,
                        target: url.to_string(),
                        title,
                        ..Default::default()
                    }))
                }
                Tag::Image(_link_type, url, title) => {
                    let caption = inlines.pop_mark();
                    let caption = if caption.is_empty() {
                        Some(caption)
                    } else {
                        None
                    };

                    let title = if !title.is_empty() {
                        Some(vec![text(title.to_string())])
                    } else {
                        None
                    };

                    let content_url = url.to_string();
                    let media_object = if let Ok(format) = Format::from_string(url) {
                        if format.is_audio() {
                            Inline::AudioObject(AudioObject {
                                content_url,
                                caption,
                                title,
                                ..Default::default()
                            })
                        } else if format.is_video() {
                            Inline::VideoObject(VideoObject {
                                content_url,
                                caption,
                                title,
                                ..Default::default()
                            })
                        } else {
                            Inline::ImageObject(ImageObject {
                                content_url,
                                caption,
                                title,
                                ..Default::default()
                            })
                        }
                    } else {
                        Inline::ImageObject(ImageObject {
                            content_url,
                            caption,
                            title,
                            ..Default::default()
                        })
                    };

                    inlines.push_inline(media_object)
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
            Event::Rule => blocks.push_block(tb()),
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
                        inlines.append_inlines(&mut blocks_to_inlines(content))
                    } else {
                        blocks.append_blocks(&mut content)
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

    (blocks.pop_all(), Losses::todo())
}

/// Decode Markdown content to a vector of [`Inline`]s
pub fn decode_inlines(md: &str) -> (Vec<Inline>, Losses) {
    let (blocks, losses) = decode_blocks(md);
    let inlines = blocks_to_inlines(blocks);
    (inlines, losses)
}

/// Stores [`Inline`] nodes so they can be used to create multi-inline
/// node types (e.g. [`Paragraph`], [`Strong`]) on [`Event::End`] events.
#[derive(Default)]
struct Inlines {
    /// Inline text content which may be parsed further
    text: String,

    /// A stack of inline nodes
    inlines: Vec<Inline>,

    /// Positions in the stack indicating the start of the parent node
    marks: Vec<usize>,

    /// Whether currently in inline content
    active: bool,
}

impl Inlines {
    /// Clear all content and mark as "active"
    /// (usually at the start of a block node with inline content)
    fn clear_all(&mut self) {
        self.text.clear();
        self.inlines.clear();
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

    /// Parse the accumulated text into accumulated `Inline` nodes
    ///
    /// This is the entry point into `nom` inline Markdown parsing functions.
    /// It is infallible in that if there is a parse error,
    /// the original input string is returned as the only item
    /// in the vector (with a warning).
    fn parse_text(&mut self) {
        if !self.text.is_empty() {
            let text_ = self.pop_text();
            let mut nodes = match inlines(&text_) {
                Ok((_, inlines)) => inlines,
                Err(error) => {
                    tracing::warn!("While parsing inline Markdown: {}", error);
                    vec![text(text_)]
                }
            };
            self.inlines.append(&mut nodes)
        }
    }

    /// Push an [`Inline`] node
    fn push_inline(&mut self, inline: Inline) {
        self.parse_text();
        self.inlines.push(inline)
    }

    /// Append [`Inline`] nodes
    fn append_inlines(&mut self, inlines: &mut Vec<Inline>) {
        self.parse_text();
        self.inlines.append(inlines)
    }

    /// Push a mark (usually at the start of an inline node)
    fn push_mark(&mut self) {
        self.parse_text();
        self.marks.push(self.inlines.len());
        self.active = true;
    }

    /// Pop the nodes since the last mark
    fn pop_mark(&mut self) -> Vec<Inline> {
        self.parse_text();
        if self.marks.is_empty() {
            vec![]
        } else {
            let n = self.marks.pop().expect("Unable to pop marks!");
            self.inlines.split_off(n)
        }
    }

    /// Pop all the nodes and mark as "inactive"
    fn pop_all(&mut self) -> Vec<Inline> {
        self.parse_text();
        self.active = false;
        self.inlines.split_off(0)
    }
}

/// Stores [`Block`] nodes so they can be used to create multi-block
/// node types (e.g. `BlockQuote`) on an [`Event::End`] events.
#[derive(Default)]
struct Blocks {
    /// Stack of blocks
    blocks: Vec<Block>,

    /// Positions in the stack indicating the start of the parent node
    marks: Vec<usize>,

    /// Marks in the stack indicating the start of a [`Division`] node
    divs: Vec<usize>,
}

impl Blocks {
    /// Push a [`Block`] node
    fn push_block(&mut self, block: Block) {
        self.blocks.push(block)
    }

    /// Append [`Block`] nodes
    fn append_blocks(&mut self, blocks: &mut Vec<Block>) {
        self.blocks.append(blocks)
    }

    /// Push a mark (usually at the start of a block node)
    fn push_mark(&mut self) {
        self.marks.push(self.blocks.len())
    }

    /// Pop the nodes since the last mark
    fn pop_mark(&mut self) -> Vec<Block> {
        match self.marks.pop() {
            Some(n) => self.blocks.split_off(n),
            None => Vec::new(),
        }
    }

    /// Push a div marker
    fn push_div(&mut self) {
        self.divs.push(self.blocks.len())
    }

    /// Pop the nodes since the last div marker
    fn pop_div(&mut self) -> Vec<Block> {
        match self.divs.pop() {
            Some(n) => self.blocks.split_off(n),
            None => Vec::new(),
        }
    }

    /// Pop all the nodes
    fn pop_all(&mut self) -> Vec<Block> {
        self.blocks.split_off(0)
    }
}

/// Stores [`ListItem`] nodes for building a [`List`] node
/// on an [`Event::End`] events for [`Tag::List`].
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

/// Stores [`TableRow`] and [`TableCell`] nodes for building a [`Table`] node
/// on an [`Event::End`] events for [`Tag::Table`].
#[derive(Default)]
struct Tables {
    /// Stack of table cells
    cells: Vec<TableCell>,

    /// Stack of table rows
    rows: Vec<TableRow>,
}

impl Tables {
    /// Push a cell
    fn push_cell(&mut self, cell: TableCell) {
        self.cells.push(cell)
    }

    /// Pop all cells, put them into a header row, and push the header row
    fn push_header(&mut self) {
        self.rows.push(TableRow {
            cells: self.cells.split_off(0),
            row_type: Some(TableRowType::Header),
            ..Default::default()
        })
    }

    /// Pop all cells, put them into a row, and pushed the row
    fn push_row(&mut self) {
        self.rows.push(TableRow {
            cells: self.cells.split_off(0),
            ..Default::default()
        })
    }

    /// Pop all rows
    fn pop_rows(&mut self) -> Vec<TableRow> {
        self.rows.split_off(0)
    }
}

/// Stores [`Block`] nodes that use fenced div syntax
#[derive(Default, Deref, DerefMut)]
struct Divs {
    /// Stack of division nodes
    divs: VecDeque<Block>,
}

/// Stores and parses HTML content
///
/// Simply accumulates HTML until tags balance, at which point the HTML is parsed,
/// with text content being parsed as Markdown by calling back to `decode_fragment`.
#[derive(Default)]
struct Html {
    /// The collected HTML
    html: String,

    /// A stack of HTML tag names used to determine whether to parse collected HTML
    tags: Vec<String>,
}

impl Html {
    /// Handle a HTML tag by either storing it or, if it balances previous tags, by
    /// returning accumulated HTML for parsing
    fn handle_html(&mut self, html: &str) -> Vec<Block> {
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
            let _html = self.html.clone() + html;
            self.html.clear();
            vec![]
            // TODO!
            //codec_html::decode_fragment(&html, Some(Box::new(|text| decode_fragment(text, None))))
        } else {
            self.html.push_str(html);
            vec![]
        }
    }
}
