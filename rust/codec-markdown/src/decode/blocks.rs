use std::{collections::HashMap, str::FromStr};

use markdown::{mdast, unist::Position};
use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_until},
    character::complete::{
        alpha1, char, multispace0, multispace1, newline, none_of, not_line_ending,
    },
    combinator::{all_consuming, map, opt, recognize},
    multi::{many0, many_m_n, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use codec::{
    common::tracing,
    schema::{
        Admonition, AdmonitionType, AutomaticExecution, Block, CallArgument, CallBlock, Claim,
        CodeBlock, CodeChunk, DeleteBlock, Figure, ForBlock, Heading, IfBlock, IfBlockClause,
        IncludeBlock, Inline, InsertBlock, InstructionBlock, InstructionBlockOptions,
        InstructionMessage, LabelType, List, ListItem, ListOrder, MathBlock, ModifyBlock,
        Paragraph, QuoteBlock, ReplaceBlock, Section, StyledBlock, SuggestionBlockType,
        SuggestionStatus, Table, TableCell, TableRow, TableRowType, Text, ThematicBreak,
    },
};

use super::{
    inlines::mds_to_inlines,
    shared::{
        assignee, attrs, name, node_to_from_str, node_to_string, primitive_node,
        take_until_unbalanced,
    },
    Context,
};

/// Transform MDAST nodes to Stencila Schema `Block`
pub(super) fn mds_to_blocks(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut divs = Vec::new();

    // Get all the blocks since the last fenced div
    fn pop_div(blocks: &mut Vec<Block>, divs: &mut Vec<usize>) -> Vec<Block> {
        if let Some(div) = divs.pop() {
            blocks.drain(div..).collect()
        } else {
            Vec::new()
        }
    }

    for md in mds {
        // Detect fenced div paragraphs (starting with `:::`) and handle them specially...
        if let mdast::Node::Paragraph(mdast::Paragraph {
            children,
            position: _,
        }) = &md
        {
            if let Some(mdast::Node::Text(mdast::Text { value, .. })) = children.first() {
                let line = value.trim_end();

                if div_else(line) {
                    // If this is an `::: else` then handle it depending on the parents block

                    let children = if let Some(div) = divs.pop() {
                        blocks.drain(div..).collect()
                    } else {
                        Vec::new()
                    };

                    if let Some(block) = blocks.last_mut() {
                        match block {
                            // Parent is a `ForBlock` so assign children to its `content` and
                            // create a placeholder `otherwise` to indicate that when the else finishes
                            // the tail of blocks should be popped to the `otherwise` of the current `ForBlock`
                            Block::ForBlock(for_block) => {
                                for_block.content = children;
                                for_block.otherwise = Some(Vec::new());
                            }

                            // Parent is an `IfBlock` so assign children to the  `content` of
                            // the last clause and add a final clause with no code or language
                            Block::IfBlock(if_block) => {
                                if let Some(last) = if_block.clauses.last_mut() {
                                    last.content = children;
                                } else {
                                    tracing::error!(
                                        "Expected there to be at least one if clause already"
                                    )
                                }
                                if_block.clauses.push(IfBlockClause::default());
                            }

                            _ => {
                                tracing::warn!(
                                    "Found an `::: else` without a preceding `::: if` or `::: for`"
                                );
                            }
                        }
                    }

                    divs.push(blocks.len());

                    continue;
                } else if div_with(line) {
                    // If this is an `::: with` then handle it depending on the parents block

                    let children = if let Some(div) = divs.pop() {
                        blocks.drain(div..).collect()
                    } else {
                        Vec::new()
                    };

                    if let Some(block) = blocks.last_mut() {
                        match block {
                            Block::ReplaceBlock(ReplaceBlock { content, .. })
                            | Block::ModifyBlock(ModifyBlock { content, .. }) => {
                                *content = children;
                            }

                            _ => {
                                tracing::warn!(
                                    "Found a `::: with` without a preceding `::: replace` or `::: modify`"
                                );
                            }
                        }
                    }

                    divs.push(blocks.len());

                    continue;
                } else if let Ok((.., (is_if, if_clause))) = div_if_elif(line) {
                    if is_if {
                        // This is a `::: if` so start a new `IfBlock`
                        blocks.push(Block::IfBlock(IfBlock {
                            clauses: vec![if_clause],
                            ..Default::default()
                        }));

                        divs.push(blocks.len());

                        continue;
                    } else {
                        let mut children = pop_div(&mut blocks, &mut divs);

                        if let Some(Block::IfBlock(if_block)) = blocks.last_mut() {
                            // This is a `::: elif` so assign children to the  `content` of
                            // the last clause and add a clause
                            if let Some(last) = if_block.clauses.last_mut() {
                                last.content = children;
                            } else {
                                tracing::error!(
                                    "Expected there to be at least one if clause already"
                                )
                            }
                            if_block.clauses.push(if_clause);

                            divs.push(blocks.len());
                            continue;
                        } else {
                            // There was no parent `IfBlock` so issue a warning and do not `continue`
                            // (so that the paragraph will be added as is). Also add the children
                            // back to blocks so they are not lost
                            tracing::warn!("Found an `::: elif` without a preceding `::: if`");
                            blocks.append(&mut children);
                        }
                    }
                } else if let Ok((.., block)) = fenced_para(line) {
                    blocks.push(block);

                    continue;
                } else if let Ok((.., (has_content, block))) = div_instruction_block(line) {
                    blocks.push(block);

                    if has_content {
                        divs.push(blocks.len());
                    }

                    continue;
                } else if let Ok((.., block)) = div_start(line) {
                    // If this is the start of a fenced div block then push it on to
                    // blocks and add a division marker to store its children.
                    // This clause must come after `::: else` and others above to avoid `div_section`
                    // prematurely matching.

                    blocks.push(block);
                    divs.push(blocks.len());

                    continue;
                } else if div_end(line) {
                    // If this the end of a fenced div, i.e. `:::`, then get children and finalize
                    // the last parent block

                    let children = pop_div(&mut blocks, &mut divs);
                    let is_suggestion = if let Some(parent) = blocks.last_mut() {
                        div_finalize(parent, children);

                        matches!(
                            parent,
                            Block::InsertBlock(..)
                                | Block::DeleteBlock(..)
                                | Block::ReplaceBlock(..)
                                | Block::ModifyBlock(..)
                        )
                    } else {
                        false
                    };

                    // If the the block before this one was an instruction and this is a suggestion
                    // then associate the two
                    if is_suggestion
                        && matches!(
                            blocks.iter().rev().nth(1),
                            Some(Block::InstructionBlock(..))
                        )
                    {
                        let suggestion = match blocks.pop() {
                            Some(Block::InsertBlock(block)) => {
                                SuggestionBlockType::InsertBlock(block)
                            }
                            Some(Block::DeleteBlock(block)) => {
                                SuggestionBlockType::DeleteBlock(block)
                            }
                            Some(Block::ReplaceBlock(block)) => {
                                SuggestionBlockType::ReplaceBlock(block)
                            }
                            Some(Block::ModifyBlock(block)) => {
                                SuggestionBlockType::ModifyBlock(block)
                            }
                            _ => unreachable!(),
                        };
                        if let Some(Block::InstructionBlock(instruct)) = blocks.last_mut() {
                            instruct.options.suggestion = Some(suggestion);
                        }
                    }

                    continue;
                }
            }
        }

        if let Some(block) = md_to_block(md, context) {
            blocks.push(block);
        };
    }

    blocks
}

/// Parse the start of a fenced paragraph: not a div with children
/// but starts with three or more semicolons
fn fenced_para(line: &str) -> IResult<&str, Block> {
    alt((include_block, call_block))(line)
}

/// Parse the start of a fenced div
fn div_start(line: &str) -> IResult<&str, Block> {
    alt((
        div_code_chunk,
        div_figure,
        div_table,
        div_for_block,
        div_delete_block,
        div_insert_block,
        div_replace_block,
        div_modify_block,
        div_claim,
        div_styled_block,
        // Section parser is permissive of label so needs to
        // come last to avoid prematurely matching others above
        div_section,
    ))(line)
}

/// Detect at least three semicolons
fn semis(line: &str) -> IResult<&str, &str> {
    recognize(many_m_n(3, 100, char(':')))(line)
}

/// Parse an [`IncludeBlock`] node
pub fn include_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("include"), multispace1)),
            tuple((is_not("{"), opt(attrs))),
        )),
        |(source, attrs)| {
            let mut options: HashMap<String, _> = attrs.unwrap_or_default().into_iter().collect();

            Block::IncludeBlock(IncludeBlock {
                source: source.trim().to_string(),
                media_type: options.remove("format").flatten().map(node_to_string),
                select: options.remove("select").flatten().map(node_to_string),
                auto_exec: options.remove("auto").flatten().and_then(node_to_from_str),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`CallBlock`] node
pub fn call_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("call"), multispace1)),
            tuple((
                is_not("("),
                delimited(
                    pair(char('('), multispace0),
                    separated_list0(delimited(multispace0, tag(","), multispace0), call_arg),
                    pair(multispace0, char(')')),
                ),
                opt(attrs),
            )),
        )),
        |(source, args, attrs)| {
            let mut options: HashMap<String, _> = attrs.unwrap_or_default().into_iter().collect();

            Block::CallBlock(CallBlock {
                source: source.trim().to_string(),
                arguments: args,
                media_type: options.remove("format").flatten().map(node_to_string),
                select: options.remove("select").flatten().map(node_to_string),
                auto_exec: options.remove("auto").flatten().and_then(node_to_from_str),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse an argument to a `CallBlock`.
///
/// Arguments must be key-value or key-symbol pairs separated by `=`.
fn call_arg(input: &str) -> IResult<&str, CallArgument> {
    map(
        // TODO allow for programming language to be specified
        pair(
            terminated(name, delimited(multispace0, tag("="), multispace0)),
            alt((
                map(delimited(char('`'), take_until("`"), char('`')), |code| {
                    (code, None)
                }),
                map(primitive_node, |node| ("", Some(node))),
            )),
        ),
        |(name, (code, value))| CallArgument {
            name: name.into(),
            code: code.into(),
            value: value.map(Box::new),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`Claim`] node
pub fn div_claim(line: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0)),
            tuple((
                alt((
                    tag("corollary"),
                    tag("hypothesis"),
                    tag("lemma"),
                    tag("postulate"),
                    tag("proof"),
                    tag("proposition"),
                    tag("statement"),
                    tag("theorem"),
                )),
                opt(preceded(multispace1, not_line_ending)),
            )),
        )),
        |(claim_type, label)| {
            Block::Claim(Claim {
                claim_type: claim_type.parse().unwrap_or_default(),
                label: label.map(String::from),
                ..Default::default()
            })
        },
    )(line)
}

/// Parse a [`CodeChunk`] node with a label and/or caption
pub fn div_code_chunk(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("chunk"), multispace0)),
            pair(
                opt(terminated(
                    alt((tag("figure"), tag("fig"), tag("fig."), tag("table"))),
                    multispace0,
                )),
                opt(not_line_ending),
            ),
        )),
        |(label_type, label)| {
            Block::CodeChunk(CodeChunk {
                label_type: label_type.and_then(|label_type| {
                    match label_type.to_lowercase().as_str() {
                        "figure" | "fig" | "fig." => Some(LabelType::FigureLabel),
                        "table" => Some(LabelType::TableLabel),
                        _ => None,
                    }
                }),
                label: label.map(|label| label.to_string()),
                ..Default::default()
            })
        },
    )(input)
}

/// Start an [`InstructionBlock`]
pub fn div_instruction_block(input: &str) -> IResult<&str, (bool, Block)> {
    let (input, has_content) = if let Some(stripped) = input.strip_suffix(":::") {
        (stripped, true)
    } else {
        (input, false)
    };

    let (remains, (assignee, text)) = preceded(
        tuple((semis, multispace0, tag("do"), multispace0)),
        pair(
            opt(delimited(char('@'), assignee, multispace1)),
            is_not("\n"),
        ),
    )(input)?;

    let block = Block::InstructionBlock(InstructionBlock {
        messages: vec![InstructionMessage::from(text)],
        options: Box::new(InstructionBlockOptions {
            assignee: assignee.map(String::from),
            ..Default::default()
        }),
        ..Default::default()
    });

    Ok((remains, (has_content, block)))
}

/// Parse a [`DeleteBlock`] node
pub fn div_delete_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("delete"), multispace0)),
            opt(not_line_ending),
        )),
        |status| {
            Block::DeleteBlock(DeleteBlock {
                suggestion_status: status
                    .and_then(|status| SuggestionStatus::from_str(status).ok()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`Figure`] node with a label and/or caption
pub fn div_figure(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((
                semis,
                multispace0,
                alt((tag("figure"), tag("fig"), tag("fig."))),
                multispace0,
            )),
            opt(not_line_ending),
        )),
        |label| {
            Block::Figure(Figure {
                label: label.and_then(|label| (!label.is_empty()).then_some(label.to_string())),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`ForBlock`] node
pub fn div_for_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("for"), multispace1)),
            tuple((
                separated_pair(
                    name,
                    tuple((multispace1, tag("in"), multispace1)),
                    is_not("{"),
                ),
                opt(preceded(
                    multispace0,
                    delimited(char('{'), take_until("}"), char('}')),
                )),
            )),
        )),
        |((variable, expr), lang)| {
            Block::ForBlock(ForBlock {
                variable: variable.into(),
                code: expr.trim().into(),
                programming_language: lang.map(|lang| lang.trim().to_string()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse an `if` or `elif` fenced div into an [`IfBlockClause`]
pub fn div_if_elif(input: &str) -> IResult<&str, (bool, IfBlockClause)> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0)),
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
                opt(attrs),
            )),
        )),
        |(tag, expr, options)| {
            let lang = options
                .iter()
                .flatten()
                .next()
                .map(|tuple| tuple.0.trim().to_string());
            (
                tag == "if",
                IfBlockClause {
                    code: expr.trim().into(),
                    programming_language: lang,
                    ..Default::default()
                },
            )
        },
    )(input)
}

/// Parse a [`InsertBlock`] node
pub fn div_insert_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("insert"), multispace0)),
            opt(not_line_ending),
        )),
        |status| {
            Block::InsertBlock(InsertBlock {
                suggestion_status: status
                    .and_then(|status| SuggestionStatus::from_str(status).ok()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`ReplaceBlock`] node
pub fn div_replace_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("replace"), multispace0)),
            opt(not_line_ending),
        )),
        |status| {
            Block::ReplaceBlock(ReplaceBlock {
                suggestion_status: status
                    .and_then(|status| SuggestionStatus::from_str(status).ok()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`ModifyBlock`] node
pub fn div_modify_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("modify"), multispace0)),
            opt(not_line_ending),
        )),
        |status| {
            Block::ModifyBlock(ModifyBlock {
                suggestion_status: status
                    .and_then(|status| SuggestionStatus::from_str(status).ok()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`Section`] node
pub fn div_section(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(tuple((semis, multispace0)), alpha1)),
        |typ| {
            Block::Section(Section {
                section_type: typ.parse().ok(),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`StyledBlock`] node
pub fn div_styled_block(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0)),
            delimited(char('{'), take_until_unbalanced('{', '}'), char('}')),
        )),
        |code| {
            Block::StyledBlock(StyledBlock {
                code: code.into(),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`Table`] with a label and/or caption
pub fn div_table(input: &str) -> IResult<&str, Block> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("table"), multispace0)),
            opt(not_line_ending),
        )),
        |label| {
            Block::Table(Table {
                label: label.map(|label| label.to_string()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a `with` fenced div
fn div_with(input: &str) -> bool {
    all_consuming(recognize(tuple((
        semis,
        multispace0,
        tag("with"),
        // Allow for, but ignore, trailing content
        opt(pair(multispace1, is_not(""))),
    ))))(input)
    .is_ok()
}

/// Parse an `else` fenced div
fn div_else(input: &str) -> bool {
    all_consuming(recognize(tuple((
        semis,
        multispace0,
        tag("else"),
        // Allow for, but ignore, trailing content
        opt(pair(multispace1, is_not(""))),
    ))))(input)
    .is_ok()
}

/// Parse the end of a fenced div
fn div_end(line: &str) -> bool {
    line.starts_with(":::") && line.ends_with(":::")
}

/// Finalize a parent div by assigning children etc
fn div_finalize(parent: &mut Block, mut children: Vec<Block>) {
    if let Block::DeleteBlock(DeleteBlock { content, .. })
    | Block::InsertBlock(InsertBlock { content, .. })
    | Block::Claim(Claim { content, .. })
    | Block::Section(Section { content, .. })
    | Block::StyledBlock(StyledBlock { content, .. }) = parent
    {
        // Parent div is a node type where we just have to assign children
        // to content.
        *content = children;
    } else if let Block::CodeChunk(chunk) = parent {
        // Parent div code chunk with label and caption etc
        for child in children {
            if let Block::CodeChunk(inner) = child {
                chunk.programming_language = inner.programming_language;
                chunk.auto_exec = inner.auto_exec;
                chunk.code = inner.code;
            } else {
                match &mut chunk.caption {
                    Some(caption) => {
                        caption.push(child);
                    }
                    None => {
                        chunk.caption = Some(vec![child]);
                    }
                }
            }
        }
    } else if let Block::Figure(figure) = parent {
        if children
            .iter()
            .filter(|block| matches!(block, Block::CodeChunk(..)))
            .count()
            == 1
        {
            // The figure has a single code chunk so return the code chunk with label type, label,
            // and caption set
            let chunk = children
                .iter()
                .position(|block| matches!(block, Block::CodeChunk(..)))
                .expect("checked above");
            let Block::CodeChunk(chunk) = children.remove(chunk) else {
                unreachable!("checked above")
            };

            *parent = Block::CodeChunk(CodeChunk {
                label_type: Some(LabelType::FigureLabel),
                label: figure.label.clone(),
                caption: (!children.is_empty()).then_some(children),
                ..chunk
            });
        } else {
            // Put all paragraphs into the caption (unless they have just a single image) and
            // everything else in the content
            let mut caption = vec![];
            let mut content = vec![];
            for child in children {
                if let Block::Paragraph(Paragraph {
                    content: inlines, ..
                }) = &child
                {
                    if let (1, Some(Inline::ImageObject(..))) = (inlines.len(), inlines.first()) {
                        content.push(child)
                    } else {
                        caption.push(child)
                    }
                } else {
                    content.push(child)
                }
            }
            figure.caption = (!caption.is_empty()).then_some(caption);
            figure.content = content;
        }
    } else if let Block::ForBlock(for_block) = parent {
        // At the end of a for block, if there is an `otherwise` placeholder
        // add children to that. If not then add them to content.
        if for_block.otherwise.is_some() {
            for_block.otherwise = Some(children);
        } else {
            for_block.content = children;
        }
    } else if let Block::IfBlock(if_block) = parent {
        // At the end of an if block assign children to the last clause.
        if let Some(last_clause) = if_block.clauses.last_mut() {
            last_clause.content = children;
        } else {
            tracing::error!("Expected if block to have at least one clause but there was none");
        }
    } else if let Block::InstructionBlock(InstructionBlock { content, .. }) = parent {
        *content = Some(children);
    } else if let Block::ReplaceBlock(replace_block) = parent {
        // At the end of replace block `::with` so set replacement
        replace_block.replacement = children;
    } else if let Block::Table(table) = parent {
        if children
            .iter()
            .filter(|block| matches!(block, Block::CodeChunk(..)))
            .count()
            == 1
        {
            // The table has a single code chunk so return the code chunk with the table label type,
            // and label and caption set to all other nodes
            let chunk = children
                .iter()
                .position(|block| matches!(block, Block::CodeChunk(..)))
                .expect("checked above");
            let Block::CodeChunk(chunk) = children.remove(chunk) else {
                unreachable!("checked above")
            };

            *parent = Block::CodeChunk(CodeChunk {
                label_type: Some(LabelType::TableLabel),
                label: table.label.clone(),
                caption: (!children.is_empty()).then_some(children),
                ..chunk
            });
        } else {
            // Put all children before the table into caption, and after into notes.
            let mut before = true;
            for child in children {
                if let Block::Table(Table { rows, .. }) = child {
                    table.rows = rows;
                    before = false;
                } else if before {
                    match &mut table.caption {
                        Some(caption) => {
                            caption.push(child);
                        }
                        None => {
                            table.caption = Some(vec![child]);
                        }
                    }
                } else {
                    match &mut table.notes {
                        Some(notes) => {
                            notes.push(child);
                        }
                        None => {
                            table.notes = Some(vec![child]);
                        }
                    }
                }
            }
        }
    }
}

/// Parse an `auto_exec` property
pub fn parse_auto_exec(input: &str) -> Option<AutomaticExecution> {
    let result: IResult<&str, &str> = preceded(
        tuple((tag("auto"), delimited(multispace0, char('='), multispace0))),
        alt((tag("always"), tag("needed"), tag("never"))),
    )(input);

    match result {
        Ok((.., value)) => match value {
            "always" => Some(AutomaticExecution::Always),
            "needed" => Some(AutomaticExecution::Needed),
            "never" => Some(AutomaticExecution::Never),
            _ => None,
        },
        Err(..) => None,
    }
}

/// Transform an MDAST node to a Stencila `Block`
fn md_to_block(md: mdast::Node, context: &mut Context) -> Option<Block> {
    Some(match md {
        mdast::Node::Yaml(mdast::Yaml { value, .. }) => {
            context.yaml = Some(value);
            return None;
        }

        mdast::Node::BlockQuote(mdast::BlockQuote { children, position }) => {
            mds_to_quote_block_or_admonition(children, position, context)
        }

        mdast::Node::Code(mdast::Code {
            lang,
            meta,
            value,
            position,
        }) => {
            let meta = meta.unwrap_or_default();
            let is_exec = meta.starts_with("exec") || lang.as_deref() == Some("exec");
            let meta = meta.strip_prefix("exec").unwrap_or_default().trim();
            if is_exec {
                let node = CodeChunk {
                    code: value.into(),
                    programming_language: if lang.as_deref() == Some("exec") {
                        None
                    } else {
                        lang
                    },
                    auto_exec: parse_auto_exec(meta),
                    ..Default::default()
                };
                context.map(position, node.node_type(), node.node_id());
                Block::CodeChunk(node)
            } else if matches!(
                lang.as_deref(),
                Some("asciimath") | Some("mathml") | Some("latex") | Some("tex")
            ) {
                let node = MathBlock {
                    code: value.into(),
                    math_language: lang,
                    ..Default::default()
                };
                context.map(position, node.node_type(), node.node_id());
                Block::MathBlock(node)
            } else {
                let node = CodeBlock {
                    code: value.into(),
                    programming_language: lang,
                    ..Default::default()
                };
                context.map(position, node.node_type(), node.node_id());
                Block::CodeBlock(node)
            }
        }

        mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
            identifier,
            children,
            label,
            ..
        }) => {
            if label.as_deref() != Some(&identifier) {
                context.lost("FootnoteDefinition.label")
            }

            let blocks = mds_to_blocks(children, context);
            context.footnote(identifier, blocks);

            return None;
        }

        mdast::Node::Heading(mdast::Heading {
            depth,
            children,
            position,
        }) => {
            let node = Heading::new(depth as i64, mds_to_inlines(children, context));
            context.map(position, node.node_type(), node.node_id());
            Block::Heading(node)
        }

        mdast::Node::List(mdast::List {
            ordered,
            children,
            position,
            ..
        }) => {
            let order = if ordered {
                ListOrder::Ascending
            } else {
                ListOrder::Unordered
            };
            let node = List::new(mds_to_list_items(children, context), order);
            context.map(position, node.node_type(), node.node_id());
            Block::List(node)
        }

        mdast::Node::Math(mdast::Math {
            meta,
            value,
            position,
        }) => {
            let node = MathBlock {
                code: value.into(),
                math_language: meta.or_else(|| Some("tex".to_string())),
                ..Default::default()
            };
            context.map(position, node.node_type(), node.node_id());
            Block::MathBlock(node)
        }

        mdast::Node::Paragraph(mdast::Paragraph { children, position }) => {
            let node = Paragraph::new(mds_to_inlines(children, context));
            context.map(position, node.node_type(), node.node_id());
            Block::Paragraph(node)
        }

        mdast::Node::Table(mdast::Table {
            children,
            align: _,
            position,
        }) => {
            // TODO: use table alignment
            let node = Table::new(mds_to_table_rows(children, context));
            context.map(position, node.node_type(), node.node_id());
            Block::Table(node)
        }

        mdast::Node::ThematicBreak(mdast::ThematicBreak { position }) => {
            let node = ThematicBreak::new();
            context.map(position, node.node_type(), node.node_id());
            Block::ThematicBreak(node)
        }

        _ => {
            // TODO: Any unexpected inlines should be aggregated into a block
            context.lost("Block");
            return None;
        }
    })
}

fn mds_to_quote_block_or_admonition(
    mds: Vec<mdast::Node>,
    position: Option<Position>,
    context: &mut Context,
) -> Block {
    let mut content = mds_to_blocks(mds, context);

    let first_text = content
        .first_mut()
        .and_then(|node| {
            if let Block::Paragraph(Paragraph { content, .. }) = node {
                content.first_mut()
            } else {
                None
            }
        })
        .and_then(|node| {
            if let Inline::Text(text) = node {
                Some(text)
            } else {
                None
            }
        });

    let first_string = first_text
        .as_ref()
        .map(|Text { value, .. }| value.to_string())
        .unwrap_or_default();

    #[allow(clippy::type_complexity)]
    let parsed: IResult<&str, (&str, Option<&str>, Option<&str>, Option<char>)> =
        tuple((
            delimited(tag("[!"), is_not("]"), tag("]")),
            opt(preceded(many0(char(' ')), alt((tag("+"), tag("-"))))),
            opt(preceded(many0(char(' ')), not_line_ending)),
            opt(newline),
        ))(&first_string);

    if let Ok((rest, (admonition_type, fold, title, ..))) = parsed {
        if let Ok(admonition_type) = admonition_type.parse::<AdmonitionType>() {
            let is_folded = fold.and_then(|symbol| match symbol {
                "-" => Some(false),
                "+" => Some(true),
                _ => None,
            });

            let title = title.and_then(|title| {
                if title.is_empty() {
                    None
                } else {
                    Some(vec![Inline::Text(Text::from(title))])
                }
            });

            if rest.is_empty() {
                content.remove(0);
            } else if let Some(first_text) = first_text {
                first_text.value = rest.into();
            }

            let node = Admonition {
                admonition_type,
                is_folded,
                title,
                content,
                ..Default::default()
            };
            context.map(position, node.node_type(), node.node_id());
            return Block::Admonition(node);
        }
    }

    let node = QuoteBlock::new(content);
    context.map(position, node.node_type(), node.node_id());
    Block::QuoteBlock(node)
}

fn mds_to_list_items(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<ListItem> {
    mds.into_iter()
        .filter_map(|md| {
            if let mdast::Node::ListItem(mdast::ListItem {
                children,
                checked,
                position,
                ..
            }) = md
            {
                let node = ListItem {
                    content: mds_to_blocks(children, context),
                    is_checked: checked,
                    ..Default::default()
                };
                context.map(position, node.node_type(), node.node_id());
                Some(node)
            } else {
                context.lost("non-ListItem");
                None
            }
        })
        .collect()
}

fn mds_to_table_rows(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<TableRow> {
    let mut first = true;
    mds.into_iter()
        .filter_map(|md| {
            if let mdast::Node::TableRow(mdast::TableRow { children, position }) = md {
                let row_type = if first {
                    first = false;
                    Some(TableRowType::HeaderRow)
                } else {
                    None
                };

                let node = TableRow {
                    cells: mds_to_table_cells(children, context),
                    row_type,
                    ..Default::default()
                };
                context.map(position, node.node_type(), node.node_id());

                Some(node)
            } else {
                context.lost("non-TableRow");
                None
            }
        })
        .collect()
}

fn mds_to_table_cells(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<TableCell> {
    mds.into_iter()
        .filter_map(|md| {
            if let mdast::Node::TableCell(mdast::TableCell { children, position }) = md {
                let content = if children.is_empty() {
                    Vec::new()
                } else {
                    vec![Block::Paragraph(Paragraph::new(mds_to_inlines(
                        children, context,
                    )))]
                };
                let node = TableCell {
                    content,
                    ..Default::default()
                };
                context.map(position, node.node_type(), node.node_id());
                Some(node)
            } else {
                context.lost("non-TableCell");
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use codec::{
        common::eyre::Result,
        schema::{ClaimType, Node},
    };

    use super::*;

    #[test]
    fn test_call_arg() -> Result<()> {
        call_arg("arg=1")?;
        call_arg("arg = 1")?;
        call_arg("arg=`1*1`")?;

        Ok(())
    }

    #[test]
    fn test_call_block() -> Result<()> {
        assert_eq!(
            call_block("::: call file.md ()")?.1,
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                ..Default::default()
            })
        );
        assert_eq!(
            call_block("::: call file.md (a=1)")?.1,
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                arguments: vec![CallArgument {
                    name: "a".to_string(),
                    value: Some(Box::new(Node::Integer(1))),
                    ..Default::default()
                }],
                ..Default::default()
            })
        );
        assert_eq!(
            call_block(r#"::: call file.md (parAm_eter_1="string")"#)?.1,
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                arguments: vec![CallArgument {
                    name: "parAm_eter_1".to_string(),
                    value: Some(Box::new(Node::String("string".to_string()))),
                    ..Default::default()
                }],
                ..Default::default()
            })
        );
        assert_eq!(
            call_block("::: call file.md (a=1.23, b=`var`, c='string')")?.1,
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                arguments: vec![
                    CallArgument {
                        name: "a".to_string(),
                        value: Some(Box::new(Node::Number(1.23))),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "b".to_string(),
                        code: "var".into(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "c".to_string(),
                        value: Some(Box::new(Node::String("string".to_string()))),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            })
        );
        assert_eq!(
            call_block("::: call file.md (a=1,b = 2  , c=3, d =4)")?.1,
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                arguments: vec![
                    CallArgument {
                        name: "a".to_string(),
                        value: Some(Box::new(Node::Integer(1))),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "b".to_string(),
                        value: Some(Box::new(Node::Integer(2))),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "c".to_string(),
                        value: Some(Box::new(Node::Integer(3))),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "d".to_string(),
                        value: Some(Box::new(Node::Integer(4))),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
        );

        Ok(())
    }

    #[test]
    fn test_claim() -> Result<()> {
        assert_eq!(
            div_claim("::: hypothesis")?.1,
            Block::Claim(Claim {
                claim_type: ClaimType::Hypothesis,
                ..Default::default()
            })
        );

        assert_eq!(
            div_claim("::: lemma Lemma 1")?.1,
            Block::Claim(Claim {
                claim_type: ClaimType::Lemma,
                label: Some(String::from("Lemma 1")),
                ..Default::default()
            })
        );

        Ok(())
    }

    #[test]
    fn test_for() -> Result<()> {
        // Simple
        assert_eq!(
            div_for_block("::: for item in expr").unwrap().1,
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            })
        );

        // With less/extra spacing
        assert_eq!(
            div_for_block(":::for item  in    expr").unwrap().1,
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            })
        );

        // With language specified
        assert_eq!(
            div_for_block("::: for item in expr {python}").unwrap().1,
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "expr".into(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            })
        );

        // With more complex expression
        assert_eq!(
            div_for_block("::: for i in 1:10").unwrap().1,
            Block::ForBlock(ForBlock {
                variable: "i".to_string(),
                code: "1:10".into(),
                ..Default::default()
            })
        );
        assert_eq!(
            div_for_block("::: for row in select * from table { sql }")
                .unwrap()
                .1,
            Block::ForBlock(ForBlock {
                variable: "row".to_string(),
                code: "select * from table".into(),
                programming_language: Some("sql".to_string()),
                ..Default::default()
            })
        );

        Ok(())
    }

    #[test]
    fn test_div_if_block() -> Result<()> {
        // Simple
        assert_eq!(
            div_if_elif("::: if expr")?.1 .1,
            IfBlockClause {
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With less/extra spacing
        assert_eq!(
            div_if_elif(":::if    expr")?.1 .1,
            IfBlockClause {
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With language specified
        assert_eq!(
            div_if_elif("::: if expr {python}")?.1 .1,
            IfBlockClause {
                code: "expr".into(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            }
        );

        // With more complex expression
        assert_eq!(
            div_if_elif("::: if a > 1 and b[8] < 1.23")?.1 .1,
            IfBlockClause {
                code: "a > 1 and b[8] < 1.23".into(),
                ..Default::default()
            }
        );

        Ok(())
    }

    #[test]
    fn test_dev_end() {
        assert!(div_end(":::"));
        assert!(div_end("::::"));
        assert!(div_end("::::::"));

        assert!(!div_end(":::some chars"));
        assert!(!div_end("::"));
        assert!(!div_end(":"));
    }
}
