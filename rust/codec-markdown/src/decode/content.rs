use std::collections::{HashMap, VecDeque};

use codec::{
    common::{
        derive_more::{Deref, DerefMut},
        once_cell::sync::Lazy,
        regex::Regex,
        tracing,
    },
    format::Format,
    schema::{
        shortcuts::{cb, dei, em, isi, mb, ol, p, qb, qi, stg, stk, t, tb, tbl, u, ul},
        transforms::blocks_to_inlines,
        AudioObject, Block, CodeChunk, Cord, DeleteBlock, Heading, IfBlock, IfBlockClause,
        ImageObject, Inline, InsertBlock, InstructionBlock, InstructionBlockOptions,
        InstructionMessage, LabelType, Link, ListItem, MessagePart, ModifyBlock, Note, NoteType,
        Paragraph, ReplaceBlock, SuggestionBlockType, Table, TableCell, TableRow, TableRowType,
        VideoObject,
    },
    Losses,
};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::decode::inlines::inlines_or_text;

use super::{
    blocks::{
        admonition, call_block, claim, code_chunk, delete_block, else_block, end, figure,
        for_block, form, if_elif, include_block, insert_block, instruct_block_start, math_block,
        modify_block, modify_block_separator, replace_block, replace_block_separator, section,
        styled_block, table,
    },
    inlines::inlines,
};

/// Decode Markdown content to a vector of [`Block`]s
pub fn decode_content(md: &str) -> (Vec<Block>, Losses) {
    // If there are no admonitions or footnotes then as a performance optimization
    // skip the following, more complex handling of footnotes
    if !md.contains("[!") && !md.contains("[^") {
        return decode_blocks(md, None);
    }

    // Split the content into footnotes and other content
    static ADMONITION_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^((>\s*)+)\[\!\w+\]").expect("Unable to create regex"));
    static FOOTNOTE_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s{0,3}\[\^\w+\]:").expect("Unable to create regex"));
    let mut footnotes_md = String::new();
    let mut other_md = String::new();
    let mut in_note = false;
    for line in md.lines() {
        if let Some(captures) = ADMONITION_REGEX.captures(line) {
            other_md.push_str(line);
            other_md.push('\n');
            // Add a blank line (with > prefix) to ensure separate first paragraph
            other_md.push_str(captures[1].into());
            other_md.push('\n');
        } else if let Some(captures) = FOOTNOTE_REGEX.captures(line) {
            in_note = true;
            footnotes_md.push_str(line);
            footnotes_md.push('\n');
            // `pulldown_cmark` requires all footnote references to have a matching footnote definition
            // so add a minimal definition here (as an optimization keep it minimal)
            other_md.push_str(captures[0].into())
        } else if in_note {
            if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
                in_note = false;
                other_md.push_str(line);
                other_md.push('\n');
            } else {
                footnotes_md.push_str(line);
                footnotes_md.push('\n');
            }
        } else {
            other_md.push_str(line);
            other_md.push('\n');
        }
    }

    // Map of footnote labels to their block content
    let mut footnotes = HashMap::new();

    // Parse the note content to populate the map of notes
    let (_, mut losses) = decode_blocks(&footnotes_md, Some(&mut footnotes));

    // Now parse the main content with the populated map of notes
    let (blocks, other_losses) = decode_blocks(&other_md, Some(&mut footnotes));
    losses.merge(other_losses);

    (blocks, losses)
}

/// Decode Markdown content to a vector of blocks
pub fn decode_blocks(
    md: &str,
    mut footnotes: Option<&mut HashMap<String, Vec<Block>>>,
) -> (Vec<Block>, Losses) {
    let mut losses = Losses::none();

    // Set Markdown parsing options
    // Do not ENABLE_SMART_PUNCTUATION as it messes with
    // single or double quoting values in `curly_attrs`.
    // Do not ENABLE_STRIKETHROUGH as it messes with `subscript`.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    // Collections of node types used in Markdown event processing
    let mut inlines = Inlines::default();
    let mut blocks = Blocks::default();
    let mut tables = Tables::default();
    let mut lists = Lists::default();
    let mut divs = Divs::default();
    let mut html = Html::default();

    // Variables that need to be persisted from start to end of a tag
    let mut current_code_block_kind = CodeBlockKind::Indented;
    let mut current_url = String::new();
    let mut current_title = String::new();
    let mut current_footnote_label = String::new();

    // Parse the Markdown and iterate over events
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
                Tag::FootnoteDefinition(label) => {
                    current_footnote_label = label.to_string();
                    blocks.push_mark()
                }

                // Block nodes with inline content
                Tag::Heading { .. } => inlines.clear_all(),
                Tag::Paragraph => inlines.clear_all(),
                Tag::CodeBlock(kind) => {
                    current_code_block_kind = kind;
                    inlines.clear_all()
                }

                // Inline nodes with inline content
                // (these should all pop the mark when they end)
                Tag::Emphasis => inlines.push_mark(),
                Tag::Strong => inlines.push_mark(),
                Tag::Strikethrough => inlines.push_mark(),
                Tag::Link {
                    dest_url, title, ..
                }
                | Tag::Image {
                    dest_url, title, ..
                } => {
                    current_url = dest_url.to_string();
                    current_title = title.to_string();
                    inlines.push_mark()
                }

                Tag::HtmlBlock => (),         // TODO
                Tag::MetadataBlock(..) => (), // TODO
            },
            Event::End(tag_end) => match tag_end {
                // Block nodes with block content
                TagEnd::BlockQuote => {
                    let mut content = blocks.pop_mark();
                    let block = match admonition(&mut content) {
                        Some(admonition) => Block::Admonition(admonition),
                        None => qb(content),
                    };
                    blocks.push_block(block)
                }
                TagEnd::List(ordered) => {
                    let items = lists.pop_mark();
                    blocks.push_block(if ordered { ol(items) } else { ul(items) })
                }
                TagEnd::Item => {
                    let mut content = Vec::new();

                    let inlines = inlines.pop_mark();
                    if !inlines.is_empty() {
                        content.push(p(inlines))
                    }

                    let mut blocks = blocks.pop_mark();
                    content.append(&mut blocks);

                    lists.push_item(ListItem::new(content))
                }
                TagEnd::Table => blocks.push_block(tbl(tables.pop_rows())),
                TagEnd::TableHead => tables.push_header(),
                TagEnd::TableRow => tables.push_row(),
                TagEnd::TableCell => {
                    let inlines = inlines.pop_mark();
                    let content = if inlines.is_empty() {
                        Vec::new()
                    } else {
                        vec![p(inlines)]
                    };
                    tables.push_cell(TableCell {
                        content,
                        ..Default::default()
                    })
                }
                TagEnd::FootnoteDefinition => {
                    if let Some(footnotes) = footnotes.as_mut() {
                        let content = blocks.pop_mark();
                        footnotes.insert(current_footnote_label.to_string(), content);
                    } else {
                        losses.add("Footnote")
                    }
                }

                // Block nodes with inline content
                TagEnd::Heading(level) => blocks.push_block(Block::Heading(Heading {
                    level: level as i64,
                    content: inlines.pop_all(),
                    ..Default::default()
                })),
                TagEnd::Paragraph => {
                    let trimmed = inlines.text.trim();

                    let block = if let Ok((.., math_block)) = math_block(trimmed) {
                        Some(Block::MathBlock(math_block))
                    } else if let Ok((.., include)) = include_block(trimmed) {
                        Some(Block::IncludeBlock(include))
                    } else if let Ok((.., call)) = call_block(trimmed) {
                        Some(Block::CallBlock(call))
                    } else if let Ok((.., (assignee, text, has_content))) =
                        instruct_block_start(trimmed)
                    {
                        let block = Block::InstructionBlock(InstructionBlock {
                            messages: vec![InstructionMessage {
                                parts: vec![MessagePart::Text(text.into())],
                                ..Default::default()
                            }],
                            options: Box::new(InstructionBlockOptions {
                                assignee: assignee.map(|handle| handle.to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                        if has_content {
                            blocks.push_div();
                            divs.push_back(block);
                            None
                        } else {
                            Some(block)
                        }
                    } else if let Ok((.., insert)) = insert_block(trimmed) {
                        if let Some(Block::InsertBlock(current)) = divs.pop_back() {
                            let insert = InsertBlock {
                                content: blocks.pop_div(),
                                ..current
                            };

                            if let Some(Block::InstructionBlock(instruction)) =
                                blocks.blocks.last_mut()
                            {
                                instruction.options.suggestion =
                                    Some(SuggestionBlockType::InsertBlock(insert));
                                None
                            } else {
                                Some(Block::InsertBlock(insert))
                            }
                        } else {
                            blocks.push_div();
                            divs.push_back(Block::InsertBlock(insert));
                            None
                        }
                    } else if let Ok((.., delete)) = delete_block(trimmed) {
                        if let Some(Block::DeleteBlock(current)) = divs.pop_back() {
                            let delete = DeleteBlock {
                                content: blocks.pop_div(),
                                ..current
                            };

                            if let Some(Block::InstructionBlock(instruction)) =
                                blocks.blocks.last_mut()
                            {
                                instruction.options.suggestion =
                                    Some(SuggestionBlockType::DeleteBlock(delete));
                                None
                            } else {
                                Some(Block::DeleteBlock(delete))
                            }
                        } else {
                            blocks.push_div();
                            divs.push_back(Block::DeleteBlock(delete));
                            None
                        }
                    } else if let Ok((.., replace)) = replace_block(trimmed) {
                        if let Some(Block::ReplaceBlock(current)) = divs.pop_back() {
                            let replace = ReplaceBlock {
                                replacement: blocks.pop_div(),
                                ..current
                            };

                            if let Some(Block::InstructionBlock(instruction)) =
                                blocks.blocks.last_mut()
                            {
                                instruction.options.suggestion =
                                    Some(SuggestionBlockType::ReplaceBlock(replace));
                                None
                            } else {
                                Some(Block::ReplaceBlock(replace))
                            }
                        } else {
                            blocks.push_div();
                            divs.push_back(Block::ReplaceBlock(replace));
                            None
                        }
                    } else if replace_block_separator(trimmed).is_ok() {
                        if let Some(Block::ReplaceBlock(current)) = divs.back_mut() {
                            current.content = blocks.pop_div();
                            blocks.push_div();
                            None
                        } else {
                            Some(p([t(trimmed)]))
                        }
                    } else if modify_block(trimmed).is_ok() {
                        if let Some(Block::ModifyBlock(current)) = divs.pop_back() {
                            // Pop a div but ignore the content since it is just a
                            // preview of the `ModifyBlock.content` property after modification
                            // is applied
                            blocks.pop_div();
                            Some(Block::ModifyBlock(ModifyBlock { ..current }))
                        } else {
                            blocks.push_div();
                            divs.push_back(Block::ModifyBlock(ModifyBlock::default()));
                            None
                        }
                    } else if modify_block_separator(trimmed).is_ok() {
                        if let Some(Block::ModifyBlock(..)) = divs.back_mut() {
                            // Pop a div but do not do anything with the content
                            // since the `ModifyBlock.content` property is read-only
                            // given that changes to it would invalidate the suggested modification
                            blocks.pop_div();
                            blocks.push_div();
                            None
                        } else {
                            Some(p([t(trimmed)]))
                        }
                    } else if let Ok((.., chunk)) = code_chunk(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::CodeChunk(chunk));
                        None
                    } else if let Ok((.., figure)) = figure(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::Figure(figure));
                        None
                    } else if let Ok((.., table)) = table(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::Table(table));
                        None
                    } else if let Ok((.., claim)) = claim(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::Claim(claim));
                        None
                    } else if let Ok((.., styled_block)) = styled_block(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::StyledBlock(styled_block));
                        None
                    } else if let Ok((.., for_block)) = for_block(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::ForBlock(for_block));
                        None
                    } else if let Ok((.., form)) = form(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::Form(form));
                        None
                    } else if let Ok((.., (true, if_clause))) = if_elif(trimmed) {
                        blocks.push_div();
                        divs.push_back(Block::IfBlock(IfBlock {
                            clauses: vec![if_clause],
                            ..Default::default()
                        }));
                        None
                    } else if let Ok((.., (false, if_clause))) = if_elif(trimmed) {
                        if let Some(Block::IfBlock(if_block)) = divs.back_mut() {
                            let content = blocks.pop_div();
                            if let Some(last) = if_block.clauses.last_mut() {
                                last.content = content;
                            } else {
                                tracing::error!(
                                    "Expected there to be at least one if clause already"
                                )
                            }
                            if_block.clauses.push(if_clause);

                            blocks.push_div();
                            None
                        } else {
                            tracing::warn!("Found an `::: elif` without a preceding `::: if`");
                            Some(p([t(trimmed)]))
                        }
                    } else if else_block(trimmed).is_ok() {
                        if let Some(div) = divs.back_mut() {
                            match div {
                                // Create a placeholder to indicate that when the else finishes
                                // the tail of blocks should be popped to the `otherwise` of the current
                                // `ForBlock`
                                Block::ForBlock(for_block) => {
                                    for_block.otherwise = Some(Vec::new());
                                }
                                // Add a last clause of `IfBlock` with no text or language
                                Block::IfBlock(if_block) => {
                                    let content = blocks.pop_div();
                                    if let Some(last) = if_block.clauses.last_mut() {
                                        last.content = content;
                                    } else {
                                        tracing::error!(
                                            "Expected there to be at least one if clause already"
                                        )
                                    }
                                    if_block.clauses.push(IfBlockClause::default());
                                }
                                _ => {
                                    tracing::warn!("Found an `::: else` without a preceding `::: if` or `::: for`");
                                }
                            }
                        }
                        blocks.push_div();
                        None
                    } else if let Ok((.., section)) = section(trimmed) {
                        // Must go after other fenced divs as will match against them
                        blocks.push_div();
                        divs.push_back(Block::Section(section));
                        None
                    } else if end(trimmed).is_ok() {
                        divs.pop_back().map(|div| match div {
                            Block::CodeChunk(mut chunk) => {
                                for block in blocks.pop_div() {
                                    if let Block::CodeChunk(inner) = block {
                                        chunk.programming_language = inner.programming_language;
                                        chunk.auto_exec = inner.auto_exec;
                                        chunk.code = inner.code;
                                    } else {
                                        match &mut chunk.caption {
                                            Some(caption) => {
                                                caption.push(block);
                                            }
                                            None => {
                                                chunk.caption = Some(vec![block]);
                                            }
                                        }
                                    }
                                }
                                Block::CodeChunk(chunk)
                            }
                            Block::Figure(mut figure) => {
                                let mut blocks = blocks.pop_div();
                                if blocks
                                    .iter()
                                    .filter(|block| matches!(block, Block::CodeChunk(..)))
                                    .count()
                                    == 1
                                {
                                    // The figure has a single code chunk so return the code chunk with label type, label,
                                    // and caption set
                                    let chunk = blocks
                                        .iter()
                                        .position(|block| matches!(block, Block::CodeChunk(..)))
                                        .expect("checked above");
                                    let Block::CodeChunk(chunk) = blocks.remove(chunk) else {
                                        unreachable!("checked above")
                                    };

                                    Block::CodeChunk(CodeChunk {
                                        label_type: Some(LabelType::FigureLabel),
                                        label: figure.label,
                                        caption: (!blocks.is_empty()).then_some(blocks),
                                        ..chunk
                                    })
                                } else {
                                    // Put all paragraphs into the caption (unless they have just a single image) and
                                    // everything else in the content
                                    let mut caption = vec![];
                                    let mut content = vec![];
                                    for block in blocks {
                                        if let Block::Paragraph(Paragraph {
                                            content: inlines,
                                            ..
                                        }) = &block
                                        {
                                            if let (1, Some(Inline::ImageObject(..))) =
                                                (inlines.len(), inlines.first())
                                            {
                                                content.push(block)
                                            } else {
                                                caption.push(block)
                                            }
                                        } else {
                                            content.push(block)
                                        }
                                    }
                                    figure.caption = (!caption.is_empty()).then_some(caption);
                                    figure.content = content;

                                    Block::Figure(figure)
                                }
                            }
                            Block::Table(mut table) => {
                                let mut blocks = blocks.pop_div();
                                if blocks
                                    .iter()
                                    .filter(|block| matches!(block, Block::CodeChunk(..)))
                                    .count()
                                    == 1
                                {
                                    // The table has a single code chunk so return the code chunk with the table label type,
                                    // and label and caption set to all other nodes
                                    let chunk = blocks
                                        .iter()
                                        .position(|block| matches!(block, Block::CodeChunk(..)))
                                        .expect("checked above");
                                    let Block::CodeChunk(chunk) = blocks.remove(chunk) else {
                                        unreachable!("checked above")
                                    };

                                    Block::CodeChunk(CodeChunk {
                                        label_type: Some(LabelType::TableLabel),
                                        label: table.label,
                                        caption: (!blocks.is_empty()).then_some(blocks),
                                        ..chunk
                                    })
                                } else {
                                    // Put all blocks before the table into caption, and after into notes.
                                    let mut before = true;
                                    for block in blocks {
                                        if let Block::Table(Table { rows, .. }) = block {
                                            table.rows = rows;
                                            before = false;
                                        } else if before {
                                            match &mut table.caption {
                                                Some(caption) => {
                                                    caption.push(block);
                                                }
                                                None => {
                                                    table.caption = Some(vec![block]);
                                                }
                                            }
                                        } else {
                                            match &mut table.notes {
                                                Some(notes) => {
                                                    notes.push(block);
                                                }
                                                None => {
                                                    table.notes = Some(vec![block]);
                                                }
                                            }
                                        }
                                    }
                                    Block::Table(table)
                                }
                            }
                            Block::Claim(mut claim) => {
                                claim.content = blocks.pop_div();
                                Block::Claim(claim)
                            }
                            Block::InstructionBlock(mut instruction) => {
                                instruction.content = Some(blocks.pop_div());
                                Block::InstructionBlock(instruction)
                            }
                            Block::Section(mut section) => {
                                section.content = blocks.pop_div();
                                Block::Section(section)
                            }
                            Block::StyledBlock(mut div) => {
                                div.content = blocks.pop_div();
                                Block::StyledBlock(div)
                            }
                            Block::ForBlock(mut for_) => {
                                for_.otherwise = for_.otherwise.map(|_| blocks.pop_div());
                                for_.content = blocks.pop_div();
                                Block::ForBlock(for_)
                            }
                            Block::Form(mut form) => {
                                form.content = blocks.pop_div();
                                Block::Form(form)
                            }
                            Block::IfBlock(mut if_) => {
                                let content = blocks.pop_div();
                                if let Some(last_clause) = if_.clauses.iter_mut().last() {
                                    last_clause.content = content;
                                } else {
                                    tracing::error!(
                                        "Expected at least one if clause but there was none"
                                    );
                                }

                                Block::IfBlock(if_)
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
                TagEnd::CodeBlock => {
                    let (lang, exec, auto_exec) = match &current_code_block_kind {
                        CodeBlockKind::Fenced(spec) => {
                            let mut spec = spec.to_string();

                            let exec = if spec.contains("exec") {
                                spec = spec.replace("exec", "");
                                true
                            } else {
                                false
                            };

                            let auto_exec = if let Some(pos) = spec.find("auto=") {
                                let auto_exec = spec[pos + 5..].trim().to_string();
                                spec = spec[..pos].to_string();
                                auto_exec.parse().ok()
                            } else {
                                None
                            };

                            let spec = spec.trim().to_string();
                            let lang = if spec.is_empty() { None } else { Some(spec) };

                            (lang, exec, auto_exec)
                        }
                        _ => (None, false, None),
                    };

                    let mut code = inlines.pop_text();
                    if code.ends_with('\n') {
                        code.pop();
                    }

                    let block = match exec {
                        true => Block::CodeChunk(CodeChunk {
                            code: Cord::from(code),
                            programming_language: lang,
                            auto_exec,
                            ..Default::default()
                        }),
                        false => match lang.as_deref() {
                            Some("asciimath") | Some("mathml") | Some("latex") | Some("tex") => {
                                mb(code, lang)
                            }
                            _ => cb(code, lang),
                        },
                    };

                    blocks.push_block(block)
                }

                // Inline nodes with inline content
                TagEnd::Emphasis => {
                    let content = inlines.pop_mark();
                    inlines.push_inline(em(content))
                }
                TagEnd::Strong => {
                    let content = inlines.pop_mark();
                    inlines.push_inline(stg(content))
                }
                TagEnd::Strikethrough => {
                    let content = inlines.pop_mark();
                    inlines.push_inline(stk(content))
                }
                TagEnd::Link => {
                    let content = inlines.pop_mark();
                    let title = {
                        if !current_title.is_empty() {
                            Some(current_title.to_string())
                        } else {
                            None
                        }
                    };
                    inlines.push_inline(Inline::Link(Link {
                        content,
                        target: current_url.to_string(),
                        title,
                        ..Default::default()
                    }))
                }
                TagEnd::Image => {
                    let caption = inlines.pop_mark();
                    let caption = if !caption.is_empty() {
                        Some(caption)
                    } else {
                        None
                    };

                    let title = if !current_title.is_empty() {
                        Some(vec![t(current_title.to_string())])
                    } else {
                        None
                    };

                    let content_url = current_url.to_string();
                    let media_object = if let Ok(format) = Format::from_url(&content_url) {
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
                TagEnd::HtmlBlock => (),         // TODO
                TagEnd::MetadataBlock(..) => (), // TODO
            },
            Event::FootnoteReference(label) => {
                let content = footnotes
                    .as_mut()
                    .and_then(|notes| notes.remove(label.as_ref()))
                    .unwrap_or_default();
                let note = Note {
                    note_type: NoteType::Footnote,
                    content,
                    ..Default::default()
                };
                inlines.push_inline(Inline::Note(note))
            }
            Event::Code(value) => {
                // Because we allow for attributes on code, we push back the
                // code in back ticks for it to be parsed again later.
                // Note that `pulldown_cmark` trims whitespace from the `value`
                // before this function is reached
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
            Event::TaskListMarker(is_checked) => lists.is_checked = Some(is_checked),
            Event::HardBreak => {
                tracing::debug!("Markdown HardBreaks are not yet handled");
            }
            Event::InlineHtml(content) | Event::Html(content) => {
                let mut content = html.handle_html(&content);
                if !content.is_empty() {
                    if inlines.active {
                        inlines.append_inlines(&mut blocks_to_inlines(content))
                    } else {
                        blocks.append_blocks(&mut content)
                    }
                }
            }
        };
    }

    if !html.tags.is_empty() {
        tracing::warn!("Unclosed HTML tags: {:?}", html.tags)
    }

    let mut blocks = blocks.pop_all();

    // Rather than discarding them, any unmatched footnotes are appended to the end
    if let Some(footnotes) = footnotes {
        blocks.append(&mut footnotes.clone().into_values().flatten().collect())
    }

    (blocks, losses)
}

/// Decode Markdown content to a vector of [`Inline`]s
pub fn decode_inlines(md: &str) -> (Vec<Inline>, Losses) {
    let (blocks, losses) = decode_blocks(md, None);
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
                    vec![t(text_)]
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

    /// Marks in the stack indicating the start of a fenced div
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
            row_type: Some(TableRowType::HeaderRow),
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
            let html = self.html.clone() + html;
            self.html.clear();

            // TODO: The following is a temporary workaround until HTML decoding
            // is implemented.

            if let Some(content) = html
                .strip_prefix("<u>")
                .and_then(|html| html.strip_suffix("</u>"))
            {
                vec![p([u(inlines_or_text(content))])]
            } else if let Some(content) = html
                .strip_prefix("<q>")
                .and_then(|html| html.strip_suffix("</q>"))
            {
                vec![p([qi(inlines_or_text(content))])]
            } else if let Some(content) = html
                .strip_prefix("<del>")
                .and_then(|html| html.strip_suffix("</del>"))
            {
                vec![p([dei(inlines_or_text(content))])]
            } else if let Some(content) = html
                .strip_prefix("<ins>")
                .and_then(|html| html.strip_suffix("</ins>"))
            {
                vec![p([isi(inlines_or_text(content))])]
            } else {
                vec![]
            }

            //codec_html::decode_fragment(&html, Some(Box::new(|text| decode_fragment(text, None))))
        } else {
            self.html.push_str(html);
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use codec::schema::shortcuts::ci;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    /// This test is just to document that pulldown_cmark trim
    /// whitespace from around inline code
    #[test]
    fn code_fragment_with_spaces() {
        assert_eq!(
            decode_blocks(r#"` some code `"#, None).0,
            vec![p([ci("some code")])]
        );
    }
}
