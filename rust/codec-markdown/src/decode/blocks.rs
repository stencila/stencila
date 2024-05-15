use std::collections::HashMap;

use markdown::{mdast, unist::Position};
use winnow::{
    ascii::{alphanumeric1, multispace0, multispace1, space0, Caseless},
    combinator::{alt, delimited, eof, opt, preceded, separated, separated_pair, terminated},
    token::{take_till, take_until, take_while},
    IResult, Located, PResult, Parser,
};

use codec::{
    common::{indexmap::IndexMap, tracing},
    schema::{
        Admonition, AdmonitionType, AutomaticExecution, Block, CallArgument, CallBlock, Claim,
        CodeBlock, CodeChunk, DeleteBlock, Figure, ForBlock, Heading, IfBlock, IfBlockClause,
        IncludeBlock, Inline, InsertBlock, InstructionBlock, InstructionMessage, LabelType, List,
        ListItem, ListOrder, MathBlock, ModifyBlock, Paragraph, QuoteBlock, ReplaceBlock, Section,
        StyledBlock, SuggestionBlockType, SuggestionStatus, Table, TableCell, TableRow,
        TableRowType, Text, ThematicBreak,
    },
};

use super::{
    inlines::{mds_to_inlines, mds_to_string},
    shared::{
        assignee, attrs, name, node_to_from_str, node_to_string, primitive_node,
        take_until_unbalanced,
    },
    Context,
};

/// Transform MDAST nodes to Stencila Schema `Block`
pub(super) fn mds_to_blocks(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut boundaries: Vec<usize> = Vec::new();

    // Get all the blocks since the last boundary
    fn pop_blocks(blocks: &mut Vec<Block>, boundary: &mut Vec<usize>) -> Vec<Block> {
        if let Some(div) = boundary.pop() {
            blocks.drain(div..).collect()
        } else {
            Vec::new()
        }
    }

    'mds: for md in mds {
        // Parse "fenced div" paragraphs (starting with `:::`) and handle them specially...
        'fenced: {
            if let mdast::Node::Paragraph(mdast::Paragraph { children, position }) = &md {
                if let Some(mdast::Node::Text(mdast::Text { value, .. })) = children.first() {
                    if !value.starts_with(":::") {
                        // Not a "fenced div" so ignore rest of this block
                        break 'fenced;
                    };

                    // Serialize MDAST nodes to text so that fragments such as
                    // URLS (for `::: include`), and `$`s for styles with interpolated variables
                    // which are otherwise consumed into non text nodes are included in parse value
                    let value = mds_to_string(children);

                    if let Ok(divider) = divider(&mut value.as_str()) {
                        let children = pop_blocks(&mut blocks, &mut boundaries);

                        match divider {
                            Divider::With => {
                                if let Some(block) = blocks.last_mut() {
                                    match block {
                                    Block::InstructionBlock(InstructionBlock {
                                        content, ..
                                    }) => {
                                        // This allows for when the `::: with` of an instruction block is a
                                        // separate paragraph (i.e. blank line between `::: do` and `::: with`)
                                        *content = Some(vec![])
                                    }

                                    Block::ReplaceBlock(ReplaceBlock { content, .. })
                                    | Block::ModifyBlock(ModifyBlock { content, .. }) => {
                                        *content = children;
                                    }

                                    _ => tracing::warn!("Found a `::: with` without a preceding `::: do`, `::: replace` or `::: modify`")
                                }
                                }
                                boundaries.push(blocks.len());
                            }
                            Divider::Else => {
                                if let Some(block) = blocks.last_mut() {
                                    match block {
                                    // Parent is a `ForBlock` so assign children to its `content` and
                                    // create a placeholder `otherwise` to indicate that when the else finishes
                                    // the tail of blocks should be popped to the `otherwise` of the current `ForBlock`
                                    Block::ForBlock(for_block) => {
                                        for_block.content = children;
                                        for_block.otherwise = Some(Vec::new());
                                    }

                                    // Parent is an `IfBlock` so assign children to the `content` of
                                    // the last clause and add a final clause with no code or language
                                    Block::IfBlock(if_block) => {
                                        if let Some(last) = if_block.clauses.last_mut() {
                                            last.content = children;
                                        } else {
                                            tracing::error!("Expected there to be at least one if clause already")
                                        }

                                        let if_block_clause = IfBlockClause::default();

                                        // End the mapping for the previous `IfBlockClause` and start a new one
                                        if let Some(position) = position {
                                            context.map_end(position.start.offset.saturating_sub(1));
                                            context.map_start(
                                                position.start.offset,
                                                if_block_clause.node_type(),
                                                if_block_clause.node_id(),
                                            );
                                        }

                                        if_block.clauses.push(if_block_clause);
                                    }

                                    _ => tracing::warn!("Found an `::: else` without a preceding `::: if` or `::: for`"),
                                }
                                }

                                boundaries.push(blocks.len());
                            }
                            Divider::End => {
                                // End the mapping for the previous block
                                if let Some(position) = position {
                                    context.map_end(position.end.offset);
                                }

                                // Finalize the last block and determine if it is a suggestion
                                let is_suggestion = if let Some(last_block) = blocks.last_mut() {
                                    finalize(last_block, children, context);

                                    // If the last block is a `IfBlock` then also need end the mapping for that
                                    if matches!(last_block, Block::IfBlock(..)) {
                                        if let Some(position) = position {
                                            context.map_end(position.end.offset);
                                        }
                                    }

                                    matches!(
                                        last_block,
                                        Block::InsertBlock(..)
                                            | Block::DeleteBlock(..)
                                            | Block::ReplaceBlock(..)
                                            | Block::ModifyBlock(..)
                                    )
                                } else {
                                    false
                                };

                                // If the the block before this one was an instruction and this is a suggestion
                                // then associate the two. Also extend the range of the mapping for the
                                // instruction to the end of the suggestion.
                                if is_suggestion
                                    && matches!(
                                        blocks.iter().rev().nth(1),
                                        Some(Block::InstructionBlock(..))
                                    )
                                {
                                    let (node_id, suggestion) = match blocks.pop() {
                                        Some(Block::InsertBlock(block)) => (
                                            block.node_id(),
                                            SuggestionBlockType::InsertBlock(block),
                                        ),
                                        Some(Block::DeleteBlock(block)) => (
                                            block.node_id(),
                                            SuggestionBlockType::DeleteBlock(block),
                                        ),
                                        Some(Block::ReplaceBlock(block)) => (
                                            block.node_id(),
                                            SuggestionBlockType::ReplaceBlock(block),
                                        ),
                                        Some(Block::ModifyBlock(block)) => (
                                            block.node_id(),
                                            SuggestionBlockType::ModifyBlock(block),
                                        ),
                                        _ => unreachable!(),
                                    };
                                    if let Some(Block::InstructionBlock(instruct)) =
                                        blocks.last_mut()
                                    {
                                        // Associate the suggestion with the instruction
                                        instruct.suggestion = Some(suggestion);

                                        // Extend the instruction to the end of the suggestion
                                        context.map_extend(instruct.node_id(), node_id);
                                    }
                                }
                            }
                        }
                        continue 'mds;
                    } else if let Ok((is_if, if_clause)) =
                        if_elif(&mut Located::new(value.as_str()))
                    {
                        if is_if {
                            let ifc_nt = if_clause.node_type();
                            let ifc_ni = if_clause.node_id();

                            // This is an `::: if` so start a new `IfBlock`
                            let if_block = IfBlock {
                                clauses: vec![if_clause],
                                ..Default::default()
                            };

                            // Start mapping entries for both the `IfBlock` and `IfBlockClause`
                            if let Some(position) = position {
                                context.map_start(
                                    position.start.offset,
                                    if_block.node_type(),
                                    if_block.node_id(),
                                );
                                context.map_start(position.start.offset, ifc_nt, ifc_ni);
                            }

                            blocks.push(Block::IfBlock(if_block));
                            boundaries.push(blocks.len());

                            continue 'mds;
                        } else {
                            // This is an `::: elif` so end the mapping for the previous `IfBlockClause`
                            // and start a new one
                            if let Some(position) = position {
                                context.map_end(position.start.offset.saturating_sub(1));
                                context.map_start(
                                    position.start.offset,
                                    if_clause.node_type(),
                                    if_clause.node_id(),
                                );
                            }

                            let mut children = pop_blocks(&mut blocks, &mut boundaries);

                            if let Some(Block::IfBlock(if_block)) = blocks.last_mut() {
                                // Assign children to the  `content` of the last clause and add a clause
                                if let Some(last) = if_block.clauses.last_mut() {
                                    last.content = children;
                                } else {
                                    tracing::error!(
                                        "Expected there to be at least one if clause already"
                                    )
                                }
                                if_block.clauses.push(if_clause);

                                boundaries.push(blocks.len());
                                continue 'mds;
                            } else {
                                // There was no parent `IfBlock` so issue a warning and do not `continue`
                                // (so that the paragraph will be added as is). Also add the children
                                // back to blocks so they are not lost
                                tracing::warn!("Found an `::: elif` without a preceding `::: if`");
                                blocks.append(&mut children);
                            }
                        }
                    } else if let Ok(block) = block(&mut Located::new(value.as_str())) {
                        // If this is the start of a "fenced div" block then push it on to
                        // blocks and add a boundary marker for its children.
                        // This clause must come after `::: else` and others above to avoid `div_section`
                        // prematurely matching.

                        // Only add a boundary for blocks that will have children to collect
                        if matches!(
                            block,
                            Block::IncludeBlock(..)
                                | Block::CallBlock(..)
                                | Block::InstructionBlock(InstructionBlock { content: None, .. })
                        ) {
                            context.map_position(position, block.node_type(), block.node_id());
                        } else {
                            boundaries.push(blocks.len() + 1);
                            if let (Some(position), Some(node_id)) = (position, block.node_id()) {
                                context.map_start(
                                    position.start.offset,
                                    block.node_type(),
                                    node_id,
                                );
                            }
                        }

                        blocks.push(block);

                        continue 'mds;
                    }
                }
            }
        }

        // MDAST nodes that can be directly translated into blocks
        if let Some((block, position)) = md_to_block(md, context) {
            context.map_position(&position, block.node_type(), block.node_id());
            blocks.push(block);
        };
    }

    blocks
}

/// Parse a "div": a paragraph starting with at least three semicolons
fn block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (take_while(3.., ':'), space0),
        alt((
            call_block,
            include_block,
            code_chunk,
            figure,
            table,
            for_block,
            instruction_block,
            delete_block,
            insert_block,
            replace_block,
            modify_block,
            claim,
            styled_block,
            // Section parser is permissive of label so needs to
            // come last to avoid prematurely matching others above
            div_section,
        )),
    )
    .parse_next(input)
}

/// Parse an argument to a `CallBlock`.
///
/// Arguments must be key-value or key-symbol pairs separated by `=`.
fn call_arg(input: &mut Located<&str>) -> PResult<CallArgument> {
    // TODO allow for programming language to be specified
    (
        terminated(name, delimited(multispace0, "=", multispace0)),
        alt((
            delimited('`', take_until(0.., "`"), '`').map(|code| (code, None)),
            primitive_node.map(|node| ("", Some(node))),
        )),
    )
        .map(|(name, (code, value))| CallArgument {
            name: name.into(),
            code: code.into(),
            value: value.map(Box::new),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse a [`CallBlock`] node
fn call_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        ("call", multispace1),
        (
            take_till(1.., '('),
            opt(delimited(
                ('(', multispace0),
                separated(0.., call_arg, delimited(multispace0, ",", multispace0)),
                (multispace0, ')'),
            )),
            opt(attrs),
        ),
    )
    .map(|(source, args, options)| {
        let mut options: HashMap<&str, _> = options.unwrap_or_default().into_iter().collect();

        Block::CallBlock(CallBlock {
            source: source.trim().to_string(),
            arguments: args.unwrap_or_default(),
            media_type: options.remove("format").flatten().map(node_to_string),
            select: options.remove("select").flatten().map(node_to_string),
            auto_exec: options.remove("auto").flatten().and_then(node_to_from_str),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse an [`IncludeBlock`] node
fn include_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (alt(("include", "inc")), multispace1),
        (take_while(1.., |c| c != '{'), opt(attrs)),
    )
    .map(|(source, attrs)| {
        let mut options: HashMap<&str, _> = attrs.unwrap_or_default().into_iter().collect();

        Block::IncludeBlock(IncludeBlock {
            source: source.trim().to_string(),
            media_type: options.remove("format").flatten().map(node_to_string),
            select: options.remove("select").flatten().map(node_to_string),
            auto_exec: options.remove("auto").flatten().and_then(node_to_from_str),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`Claim`] node
fn claim(input: &mut Located<&str>) -> PResult<Block> {
    (
        terminated(
            alt((
                Caseless("corollary"),
                Caseless("hypothesis"),
                Caseless("lemma"),
                Caseless("postulate"),
                Caseless("proof"),
                Caseless("proposition"),
                Caseless("statement"),
                Caseless("theorem"),
            )),
            multispace0,
        ),
        opt(take_while(1.., |_| true)),
    )
        .map(|(claim_type, label): (&str, Option<&str>)| {
            Block::Claim(Claim {
                claim_type: claim_type.parse().unwrap_or_default(),
                label: label.map(String::from),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a [`CodeChunk`] node with a label and/or caption
fn code_chunk(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        ("chunk", multispace0),
        (
            opt(terminated(
                alt((
                    Caseless("figure"),
                    Caseless("fig"),
                    Caseless("fig."),
                    Caseless("table"),
                )),
                multispace0,
            )),
            opt(take_while(1.., |_| true)),
        ),
    )
    .map(|(label_type, label): (Option<&str>, Option<&str>)| {
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
    })
    .parse_next(input)
}

/// Parse a [`Figure`] node with a label and/or caption
fn figure(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (
            alt((Caseless("figure"), Caseless("fig"), Caseless("fig."))),
            multispace0,
        ),
        opt(take_while(1.., |_| true)),
    )
    .map(|label: Option<&str>| {
        Block::Figure(Figure {
            label: label.and_then(|label| (!label.is_empty()).then_some(label.to_string())),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`ForBlock`] node
fn for_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        ("for", multispace1),
        (
            separated_pair(
                name,
                (multispace1, "in", multispace1),
                alt((
                    delimited('`', take_until(0.., '`'), '`'),
                    take_while(1.., |c| c != '{'),
                )),
            ),
            opt(preceded(multispace0, attrs)),
        ),
    )
    .map(|((variable, expr), options)| {
        let mut options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

        Block::ForBlock(ForBlock {
            variable: variable.into(),
            code: expr.trim().into(),
            programming_language: options.first().map(|(name, _)| name.to_string()),
            auto_exec: options
                .swap_remove("auto")
                .flatten()
                .and_then(node_to_from_str),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse an `if` or `elif` fenced div into an [`IfBlockClause`]
fn if_elif(input: &mut Located<&str>) -> PResult<(bool, IfBlockClause)> {
    (
        delimited(
            (take_while(3.., ':'), space0),
            alt(("if", "elif")),
            multispace1,
        ),
        alt((
            delimited('`', take_until(0.., '`'), '`'),
            take_while(1.., |c| c != '{'),
        )),
        opt(preceded(multispace0, attrs)),
    )
        .map(|(tag, expr, options)| {
            let mut options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

            (
                tag == "if",
                IfBlockClause {
                    code: expr.trim().into(),
                    programming_language: options.first().map(|(name, _)| name.to_string()),
                    auto_exec: options
                        .swap_remove("auto")
                        .flatten()
                        .and_then(node_to_from_str),
                    ..Default::default()
                },
            )
        })
        .parse_next(input)
}

/// Start an [`InstructionBlock`]
fn instruction_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        ("do", multispace0),
        (
            opt(delimited('@', assignee, multispace0)),
            opt(alt((
                (
                    take_until(0.., ':'),
                    (take_while(3.., ':'), multispace0, opt("with")).value(true),
                ),
                (take_while(0.., |_| true), "".value(false)),
            ))),
        ),
    )
    .map(
        |(assignee, message): (Option<&str>, Option<(&str, bool)>)| {
            let (text, content) = message.unwrap_or_default();
            Block::InstructionBlock(InstructionBlock {
                messages: vec![InstructionMessage::from(text.trim())],
                content: if content { Some(Vec::new()) } else { None },
                assignee: assignee.map(String::from),
                ..Default::default()
            })
        },
    )
    .parse_next(input)
}

/// Parse a suggestion status
fn suggestion_status(input: &mut Located<&str>) -> PResult<SuggestionStatus> {
    alt((
        alt(("accepted", "accept")).map(|_| SuggestionStatus::Accepted),
        alt(("rejected", "reject")).map(|_| SuggestionStatus::Rejected),
        alt(("proposed", "propose")).map(|_| SuggestionStatus::Proposed),
    ))
    .parse_next(input)
}

/// Parse a [`InsertBlock`] node
fn insert_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (alt(("insert", "ins")), multispace0),
        opt(suggestion_status),
    )
    .map(|suggestion_status| {
        Block::InsertBlock(InsertBlock {
            suggestion_status,
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`DeleteBlock`] node
fn delete_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (alt(("delete", "del")), multispace0),
        opt(suggestion_status),
    )
    .map(|suggestion_status| {
        Block::DeleteBlock(DeleteBlock {
            suggestion_status,
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`ReplaceBlock`] node
fn replace_block(input: &mut Located<&str>) -> PResult<Block> {
    delimited(
        (alt(("replace", "rep")), multispace0),
        opt(suggestion_status),
        opt(delimited(multispace0, "::: with", multispace0)),
    )
    .map(|suggestion_status| {
        Block::ReplaceBlock(ReplaceBlock {
            suggestion_status,
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`ModifyBlock`] node
fn modify_block(input: &mut Located<&str>) -> PResult<Block> {
    delimited(
        (alt(("modify", "mod")), multispace0),
        opt(suggestion_status),
        opt(delimited(multispace0, "::: with", multispace0)),
    )
    .map(|suggestion_status| {
        Block::ModifyBlock(ModifyBlock {
            suggestion_status,
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`Section`] node
fn div_section(input: &mut Located<&str>) -> PResult<Block> {
    alphanumeric1
        .map(|section_type: &str| {
            Block::Section(Section {
                section_type: section_type.parse().ok(),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a [`StyledBlock`] node
fn styled_block(input: &mut Located<&str>) -> PResult<Block> {
    delimited('{', take_until_unbalanced('{', '}'), '}')
        .map(|code: &str| {
            Block::StyledBlock(StyledBlock {
                code: code.trim().into(),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a [`Table`] with a label and/or caption
fn table(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (Caseless("table"), multispace0),
        opt(take_while(1.., |_| true)),
    )
    .map(|label: Option<&str>| {
        Block::Table(Table {
            label: label.map(|label| label.to_string()),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a divider between sections of content
fn divider(input: &mut &str) -> PResult<Divider> {
    delimited(
        (take_while(3.., ':'), space0),
        alt((
            "else".map(|_| Divider::Else),
            "with".map(|_| Divider::With),
            "".map(|_| Divider::End),
        )),
        (space0, eof),
    )
    .parse_next(input)
}

#[derive(Debug, PartialEq)]
enum Divider {
    With,
    Else,
    End,
}

/// Finalize a block by assigning children etc
fn finalize(parent: &mut Block, mut children: Vec<Block>, context: &mut Context) {
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
                let node_id = inner.node_id();
                chunk.programming_language = inner.programming_language;
                chunk.auto_exec = inner.auto_exec;
                chunk.code = inner.code;

                // Remove the inner code chunk from the mapping
                context.map_remove(node_id)
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
            let Block::CodeChunk(mut chunk) = children.remove(chunk) else {
                unreachable!("checked above")
            };

            chunk.label_type = Some(LabelType::FigureLabel);
            chunk.label = figure.label.clone();
            chunk.caption = (!children.is_empty()).then_some(children);

            // Replace the mapping entry for figure, with one for chunk
            context.map_remove(chunk.node_id());
            context.map_replace(figure.node_id(), chunk.node_type(), chunk.node_id());

            *parent = Block::CodeChunk(chunk);
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
            let Block::CodeChunk(mut chunk) = children.remove(chunk) else {
                unreachable!("checked above")
            };

            chunk.label_type = Some(LabelType::TableLabel);
            chunk.label = table.label.clone();
            chunk.caption = (!children.is_empty()).then_some(children);

            // Replace the mapping entry for figure, with one for chunk
            context.map_remove(chunk.node_id());
            context.map_replace(table.node_id(), chunk.node_type(), chunk.node_id());

            *parent = Block::CodeChunk(chunk);
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

/// Parse a suggestion status
fn parse_auto_exec(input: &mut &str) -> PResult<AutomaticExecution> {
    preceded(
        ("auto", multispace0, '=', multispace0),
        alt((
            "always".map(|_| AutomaticExecution::Always),
            "needed".map(|_| AutomaticExecution::Needed),
            "never".map(|_| AutomaticExecution::Never),
        )),
    )
    .parse_next(input)
}

/// Transform an MDAST node to a Stencila `Block`
fn md_to_block(md: mdast::Node, context: &mut Context) -> Option<(Block, Option<Position>)> {
    Some(match md {
        mdast::Node::Yaml(mdast::Yaml { value, .. }) => {
            context.yaml = Some(value);
            return None;
        }

        mdast::Node::BlockQuote(mdast::BlockQuote { children, position }) => (
            mds_to_quote_block_or_admonition(children, context),
            position,
        ),

        mdast::Node::Code(mdast::Code {
            lang,
            meta,
            value,
            position,
        }) => {
            let meta = meta.unwrap_or_default();
            let is_exec = meta.starts_with("exec") || lang.as_deref() == Some("exec");
            let mut meta = meta.strip_prefix("exec").unwrap_or_default().trim();

            let block = if is_exec {
                let is_invisible = value.contains("@invisible").then_some(true);

                Block::CodeChunk(CodeChunk {
                    code: value.into(),
                    programming_language: if lang.as_deref() == Some("exec") {
                        None
                    } else {
                        lang
                    },
                    auto_exec: parse_auto_exec(&mut meta).ok(),
                    is_invisible,
                    ..Default::default()
                })
            } else if matches!(
                lang.as_deref(),
                Some("asciimath") | Some("math") | Some("mathml") | Some("latex") | Some("tex")
            ) {
                Block::MathBlock(MathBlock {
                    code: value.into(),
                    math_language: lang,
                    ..Default::default()
                })
            } else {
                Block::CodeBlock(CodeBlock {
                    code: value.into(),
                    programming_language: lang,
                    ..Default::default()
                })
            };

            (block, position)
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
        }) => (
            Block::Heading(Heading::new(
                depth as i64,
                mds_to_inlines(children, context),
            )),
            position,
        ),

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
            (
                Block::List(List::new(mds_to_list_items(children, context), order)),
                position,
            )
        }

        mdast::Node::Math(mdast::Math {
            meta,
            value,
            position,
        }) => (
            Block::MathBlock(MathBlock {
                code: value.into(),
                math_language: meta
                    .and_then(|string| string.split_whitespace().next().map(String::from)),
                ..Default::default()
            }),
            position,
        ),

        mdast::Node::Paragraph(mdast::Paragraph { children, position }) => (
            Block::Paragraph(Paragraph::new(mds_to_inlines(children, context))),
            position,
        ),

        mdast::Node::Table(mdast::Table {
            children,
            align: _,
            position,
        }) => {
            // TODO: use table alignment
            (
                Block::Table(Table::new(mds_to_table_rows(children, context))),
                position,
            )
        }

        mdast::Node::ThematicBreak(mdast::ThematicBreak { position }) => {
            (Block::ThematicBreak(ThematicBreak::new()), position)
        }

        _ => {
            // TODO: Any unexpected inlines should be aggregated into a block
            context.lost("Block");
            return None;
        }
    })
}

fn mds_to_quote_block_or_admonition(mds: Vec<mdast::Node>, context: &mut Context) -> Block {
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
    let parsed: IResult<&str, (&str, Option<&str>, Option<&str>, Option<char>)> = (
        delimited("[!", take_until(1.., "]"), "]"),
        opt(preceded(space0, alt(("+", "-")))),
        opt(preceded(space0, take_while(1.., |c| c != '\n'))),
        opt('\n'),
    )
        .parse_peek(first_string.as_str());

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

            return Block::Admonition(Admonition {
                admonition_type,
                is_folded,
                title,
                content,
                ..Default::default()
            });
        }
    }

    Block::QuoteBlock(QuoteBlock::new(content))
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
                context.map_position(&position, node.node_type(), Some(node.node_id()));
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
                context.map_position(&position, node.node_type(), Some(node.node_id()));

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
                context.map_position(&position, node.node_type(), Some(node.node_id()));
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
    use codec::schema::{ClaimType, Node};

    use super::*;

    #[test]
    fn test_call_arg() {
        call_arg(&mut Located::new("arg=1")).unwrap();
        call_arg(&mut Located::new("arg = 1")).unwrap();
        call_arg(&mut Located::new("arg=`1*1`")).unwrap();
    }

    #[test]
    fn test_call_block() {
        assert_eq!(
            call_block(&mut Located::new("call file.md ()")).unwrap(),
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                ..Default::default()
            })
        );
        assert_eq!(
            call_block(&mut Located::new("call file.md (a=1)")).unwrap(),
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
            call_block(&mut Located::new(r#"call file.md (parAm_eter_1="string")"#)).unwrap(),
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
            call_block(&mut Located::new(
                "call file.md (a=1.23, b=`var`, c='string')"
            ))
            .unwrap(),
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
            call_block(&mut Located::new("call file.md (a=1,b = 2  , c=3, d =4)")).unwrap(),
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
    }

    #[test]
    fn test_claim() {
        assert_eq!(
            claim(&mut Located::new("hypothesis")).unwrap(),
            Block::Claim(Claim {
                claim_type: ClaimType::Hypothesis,
                ..Default::default()
            })
        );

        assert_eq!(
            claim(&mut Located::new("lemma Lemma 1")).unwrap(),
            Block::Claim(Claim {
                claim_type: ClaimType::Lemma,
                label: Some(String::from("Lemma 1")),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_for_block() {
        // Simple
        assert_eq!(
            for_block(&mut Located::new("for item in expr")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            })
        );

        // With less/extra spacing
        assert_eq!(
            for_block(&mut Located::new("for item  in    expr")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            })
        );

        // With language specified
        assert_eq!(
            for_block(&mut Located::new("for item in expr {python}")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "expr".into(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            })
        );

        // With more complex expression
        assert_eq!(
            for_block(&mut Located::new("for i in 1:10")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "i".to_string(),
                code: "1:10".into(),
                ..Default::default()
            })
        );

        // With more complex expression using backticks and language
        assert_eq!(
            for_block(&mut Located::new("for iTem_ in `[{},{}]` {js}")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "iTem_".to_string(),
                code: "[{},{}]".into(),
                programming_language: Some("js".to_string()),
                ..Default::default()
            })
        );

        // With more complex expression and language and auto exec
        assert_eq!(
            for_block(&mut Located::new(
                "for row in select * from table { sql, auto=never }"
            ))
            .unwrap(),
            Block::ForBlock(ForBlock {
                variable: "row".to_string(),
                code: "select * from table".into(),
                programming_language: Some("sql".to_string()),
                auto_exec: Some(AutomaticExecution::Never),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_if_elif() {
        // Simple
        assert_eq!(
            if_elif(&mut Located::new("::: if expr")).unwrap(),
            (
                true,
                IfBlockClause {
                    code: "expr".into(),
                    ..Default::default()
                }
            )
        );

        // With less/extra spacing
        assert_eq!(
            if_elif(&mut Located::new("::: if    expr")).unwrap(),
            (
                true,
                IfBlockClause {
                    code: "expr".into(),
                    ..Default::default()
                }
            )
        );

        // With language specified
        assert_eq!(
            if_elif(&mut Located::new("::: if expr {python}")).unwrap(),
            (
                true,
                IfBlockClause {
                    code: "expr".into(),
                    programming_language: Some("python".to_string()),
                    ..Default::default()
                }
            )
        );

        // With more complex expression
        assert_eq!(
            if_elif(&mut Located::new("::: elif a > 1 and b[8] < 1.23")).unwrap(),
            (
                false,
                IfBlockClause {
                    code: "a > 1 and b[8] < 1.23".into(),
                    ..Default::default()
                }
            )
        );

        // With more complex expression and language
        assert_eq!(
            if_elif(&mut Located::new(
                "::: elif `a if true else b - c[5:-1] + {}['d']` {python}"
            ))
            .unwrap(),
            (
                false,
                IfBlockClause {
                    code: "a if true else b - c[5:-1] + {}['d']".into(),
                    programming_language: Some("python".to_string()),
                    ..Default::default()
                }
            )
        );
    }

    #[test]
    fn test_styled_block() {
        assert_eq!(
            styled_block(&mut Located::new("{}")).unwrap(),
            Block::StyledBlock(StyledBlock {
                ..Default::default()
            })
        );

        assert_eq!(
            styled_block(&mut Located::new("{ color: red }")).unwrap(),
            Block::StyledBlock(StyledBlock {
                code: "color: red".into(),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_divider() {
        assert_eq!(divider(&mut "::: with").unwrap(), Divider::With);
        assert_eq!(divider(&mut "::::: with  ").unwrap(), Divider::With);

        assert_eq!(divider(&mut "::: else").unwrap(), Divider::Else);
        assert_eq!(divider(&mut "::::: else  ").unwrap(), Divider::Else);

        assert_eq!(divider(&mut ":::").unwrap(), Divider::End);
        assert_eq!(divider(&mut "::::").unwrap(), Divider::End);
        assert_eq!(divider(&mut "::::::").unwrap(), Divider::End);

        assert!(divider(&mut "::: some chars").is_err());
        assert!(divider(&mut "::: with :::").is_err());
        assert!(divider(&mut "::").is_err());
        assert!(divider(&mut ":").is_err());
    }
}
