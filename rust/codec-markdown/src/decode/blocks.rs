use std::collections::HashMap;

use markdown::{mdast, unist::Position};
use winnow::{
    ascii::{alphanumeric1, digit1, multispace0, multispace1, space0, Caseless},
    combinator::{alt, delimited, eof, opt, preceded, separated, separated_pair, terminated},
    token::{take_till, take_until, take_while},
    IResult, Located, PResult, Parser,
};

use codec::{
    common::{indexmap::IndexMap, tracing},
    schema::{
        shortcuts, Admonition, AdmonitionType, Block, CallArgument, CallBlock, Claim, CodeBlock,
        CodeChunk, DeleteBlock, ExecutionMode, Figure, ForBlock, Heading, IfBlock, IfBlockClause,
        IncludeBlock, Inline, InsertBlock, InstructionBlock, InstructionMessage, InstructionModel,
        LabelType, List, ListItem, ListOrder, MathBlock, ModifyBlock, Node, Paragraph, QuoteBlock,
        RawBlock, ReplaceBlock, Section, StyledBlock, SuggestionBlock, Table, TableCell, TableRow,
        TableRowType, Text, ThematicBreak,
    },
};

use super::{
    decode_blocks, decode_inlines,
    inlines::{mds_to_inlines, mds_to_string},
    shared::{
        attrs, execution_mode, instruction_type, model, name, node_to_string, primitive_node,
        prompt, string_to_instruction_message, take_until_unbalanced,
    },
    Context,
};

/// Transform MDAST nodes to Stencila Schema `Block`
pub(super) fn mds_to_blocks(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut boundaries: Vec<usize> = Vec::new();

    // Get all the blocks since the last boundary
    fn pop_blocks(blocks: &mut Vec<Block>, boundaries: &mut Vec<usize>) -> Vec<Block> {
        if let Some(boundary) = boundaries.pop() {
            if boundary > blocks.len() {
                tracing::error!("Boundary index above length of blocks");
                Vec::new()
            } else {
                blocks.drain(boundary..).collect()
            }
        } else {
            Vec::new()
        }
    }

    // Used to avoid running `fold_block` if there are no instructions or suggestions
    let mut should_fold = false;
    for md in mds.into_iter() {
        let mut is_handled = false;

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
                                        Block::ReplaceBlock(ReplaceBlock { content, .. })
                                        | Block::ModifyBlock(ModifyBlock { content, .. }) => {
                                            *content = children;
                                        }

                                        _ => tracing::warn!("Found a `::: with` without a preceding `::: replace` or `::: modify`")
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

                                // Finalize the last block
                                if let Some(last_block) = blocks.last_mut() {
                                    finalize(last_block, children, context);

                                    // If the last block is a `IfBlock` then also need end the mapping for that
                                    if matches!(last_block, Block::IfBlock(..)) {
                                        if let Some(position) = position {
                                            context.map_end(position.end.offset);
                                        }
                                    }
                                }
                            }
                        }

                        is_handled = true;
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
                            } else {
                                // There was no parent `IfBlock` so issue a warning and do not `continue`
                                // (so that the paragraph will be added as is). Also add the children
                                // back to blocks so they are not lost
                                tracing::warn!("Found an `::: elif` without a preceding `::: if`");
                                blocks.append(&mut children);
                            }
                        }

                        is_handled = true;
                    } else if let Ok(block) = block(&mut Located::new(value.as_str())) {
                        // This is the start of a "fenced div" block so push it on to
                        // blocks and add a boundary marker for its children.
                        // This clause must come after `::: else` and others above to avoid `section`
                        // prematurely matching.

                        if !should_fold
                            & matches!(
                                block,
                                Block::InstructionBlock(..) | Block::SuggestionBlock(..)
                            )
                        {
                            should_fold = true;
                        };

                        // Only add a boundary for blocks that are expected to have a closing fence
                        let add_boundary =
                            if let Block::InstructionBlock(InstructionBlock { content, .. }) =
                                &block
                            {
                                match content {
                                    Some(content) => content.capacity() != 1,
                                    None => false,
                                }
                            } else if let Block::SuggestionBlock(SuggestionBlock {
                                content, ..
                            }) = &block
                            {
                                content.capacity() != 1
                            } else {
                                !matches!(block, Block::IncludeBlock(..) | Block::CallBlock(..))
                            };

                        // Add boundary and map position
                        if add_boundary {
                            boundaries.push(blocks.len() + 1);
                            if let (Some(position), Some(node_id)) = (position, block.node_id()) {
                                context.map_start(
                                    position.start.offset,
                                    block.node_type(),
                                    node_id,
                                );
                            }
                        } else {
                            context.map_position(position, block.node_type(), block.node_id());
                        }

                        blocks.push(block);
                        is_handled = true;
                    }
                }
            }
        }

        if is_handled {
            continue;
        }

        // MDAST nodes that can be directly translated into blocks
        if let Some((block, position)) = md_to_block(md, context) {
            if !should_fold
                & matches!(
                    block,
                    Block::InstructionBlock(..) | Block::SuggestionBlock(..)
                )
            {
                should_fold = true;
            };

            context.map_position(&position, block.node_type(), block.node_id());

            blocks.push(block);
        }
    }

    // Fold blocks into previous blocks if necessary
    if should_fold {
        fold_blocks(&mut blocks, context);
    }
    fn fold_blocks(blocks: &mut Vec<Block>, context: &mut Context) {
        let mut index = 0;
        while index < blocks.len() {
            // Used to avoid incrementing index when the next block has been removed
            let mut step = true;

            // If the current block is an instruction or suggestion with content capacity == 1
            // but length == 0, fold the next block into its content (unless it is a suggestion)
            if let (
                Some(
                    Block::SuggestionBlock(SuggestionBlock { content, .. })
                    | Block::InstructionBlock(InstructionBlock {
                        content: Some(content),
                        ..
                    }),
                ),
                Some(next),
            ) = (blocks.get(index), blocks.get(index + 1))
            {
                if !matches!(next, Block::SuggestionBlock(..))
                    && content.capacity() == 1
                    && content.is_empty()
                {
                    let next = blocks.remove(index + 1);

                    if let (Some(current_id), Some(next_id)) = (
                        blocks.get(index).and_then(|block| block.node_id()),
                        next.node_id(),
                    ) {
                        context.map_extend(current_id, next_id);
                    }

                    if let Some(
                        Block::InstructionBlock(InstructionBlock {
                            content: Some(content),
                            ..
                        })
                        | Block::SuggestionBlock(SuggestionBlock { content, .. }),
                    ) = blocks.get_mut(index)
                    {
                        content.push(next);
                    }

                    step = false;
                }
            }

            // If the current block is an instruction and the next is a suggestion, fold
            // the suggestion into the instruction's suggestion
            if let (Some(Block::InstructionBlock(..)), Some(Block::SuggestionBlock(..))) =
                (blocks.get(index), blocks.get(index + 1))
            {
                if let (Some(current_id), Some(next_id)) = (
                    blocks.get(index).and_then(|block| block.node_id()),
                    blocks.get(index + 1).and_then(|block| block.node_id()),
                ) {
                    context.map_extend(current_id, next_id);
                }

                if let (
                    Block::SuggestionBlock(suggestion),
                    Some(Block::InstructionBlock(InstructionBlock { suggestions, .. })),
                ) = (blocks.remove(index + 1), blocks.get_mut(index))
                {
                    match suggestions {
                        Some(suggestions) => {
                            suggestions.push(suggestion);
                        }
                        None => {
                            suggestions.replace(vec![suggestion]);
                        }
                    }

                    step = false;
                }
            }

            // If the current block is an instruction and its last suggestion has content capacity == 1
            // but length == 0, fold the next block into the suggestion's content (unless it is a suggestion)
            if let (
                Some(Block::InstructionBlock(InstructionBlock {
                    suggestions: Some(suggestions),
                    ..
                })),
                Some(next),
            ) = (blocks.get(index), blocks.get(index + 1))
            {
                if !matches!(next, Block::SuggestionBlock(..)) {
                    if let Some(SuggestionBlock { content, .. }) = suggestions.last() {
                        if content.capacity() == 1 && content.is_empty() {
                            if let (Some(current_id), Some(next_id)) = (
                                blocks.get(index).and_then(|block| block.node_id()),
                                blocks.get(index + 1).and_then(|block| block.node_id()),
                            ) {
                                if let Some(suggestion) = suggestions.last() {
                                    let suggestion_id = suggestion.node_id();
                                    context.map_extend(suggestion_id.clone(), next_id.clone());
                                    context.map_extend(current_id, suggestion_id);
                                }
                            }

                            let next = blocks.remove(index + 1);

                            if let Some(Block::InstructionBlock(InstructionBlock {
                                suggestions: Some(suggestions),
                                ..
                            })) = blocks.get_mut(index)
                            {
                                if let Some(SuggestionBlock { content, .. }) =
                                    suggestions.last_mut()
                                {
                                    content.push(next);
                                }
                            }

                            step = false;
                        }
                    }
                }
            }

            // Recurse into blocks the have block content so any instructions
            // or suggestions in them are also folded if needed
            if let Some(block) = blocks.get_mut(index) {
                if let Block::Admonition(Admonition {
                    content: blocks, ..
                })
                | Block::Section(Section {
                    content: blocks, ..
                })
                | Block::Table(Table {
                    caption: Some(blocks),
                    ..
                })
                | Block::ForBlock(ForBlock {
                    content: blocks, ..
                }) = block
                {
                    fold_blocks(blocks, context);
                } else if let Block::Figure(Figure {
                    caption, content, ..
                }) = block
                {
                    if let Some(caption) = caption {
                        fold_blocks(caption, context);
                    }
                    fold_blocks(content, context);
                } else if let Block::IfBlock(IfBlock { clauses, .. }) = block {
                    for clause in clauses {
                        fold_blocks(&mut clause.content, context);
                    }
                } else if let Block::InstructionBlock(InstructionBlock {
                    suggestions: Some(suggestions),
                    ..
                }) = block
                {
                    for suggestion in suggestions {
                        fold_blocks(&mut suggestion.content, context);
                    }
                }
            }

            if step {
                index += 1;
            }
        }
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
            suggestion_block,
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
        let mut options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

        Block::CallBlock(CallBlock {
            source: source.trim().to_string(),
            arguments: args.unwrap_or_default(),
            media_type: options.swap_remove("format").flatten().map(node_to_string),
            select: options.swap_remove("select").flatten().map(node_to_string),
            execution_mode: execution_mode_from_options(options),
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
        let mut options: IndexMap<&str, _> = attrs.unwrap_or_default().into_iter().collect();

        Block::IncludeBlock(IncludeBlock {
            source: source.trim().to_string(),
            media_type: options.swap_remove("format").flatten().map(node_to_string),
            select: options.swap_remove("select").flatten().map(node_to_string),
            execution_mode: execution_mode_from_options(options),
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
            label_automatically: label.is_some().then_some(false),
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
            label_automatically: label.is_some().then_some(false),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`ForBlock`] node
fn for_block(input: &mut Located<&str>) -> PResult<Block> {
    alt((
        // Stencila Markdown
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
        ),
        // MyST
        preceded(
            ("{for}", multispace0),
            (
                separated_pair(
                    name,
                    (multispace1, "in", multispace1),
                    take_while(1.., |c| c != '{'),
                ),
                "".value(None),
            ),
        ),
    ))
    .map(|((variable, expr), options)| {
        let options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

        Block::ForBlock(ForBlock {
            variable: variable.into(),
            code: expr.trim().into(),
            programming_language: options.first().map(|(name, _)| name.to_string()),
            execution_mode: execution_mode_from_options(options),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse an `if` or `elif` fenced div into an [`IfBlockClause`]
fn if_elif(input: &mut Located<&str>) -> PResult<(bool, IfBlockClause)> {
    alt((
        // Stencila Markdown
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
        ),
        // MyST
        (
            delimited(
                (take_while(3.., ':'), '{'),
                alt(("if", "elif")),
                ('}', multispace0),
            ),
            take_while(1.., |_| true),
            "".value(None),
        ),
    ))
    .map(|(tag, expr, options)| {
        let options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

        (
            tag == "if",
            IfBlockClause {
                code: expr.trim().into(),
                programming_language: options.first().map(|(name, _)| name.to_string()),
                execution_mode: execution_mode_from_options(options),
                ..Default::default()
            },
        )
    })
    .parse_next(input)
}

/// Start an [`InstructionBlock`]
fn instruction_block(input: &mut Located<&str>) -> PResult<Block> {
    (
        terminated(instruction_type, multispace0),
        opt(delimited('@', prompt, multispace0)),
        opt(delimited(
            ('[', multispace0),
            model,
            (multispace0, ']', multispace0),
        )),
        separated(
            0..,
            (alt(('x', 'y', 't', 'q', 's', 'c')), digit1),
            multispace1,
        ),
        opt(take_while(0.., |_| true)),
    )
        .map(
            |(instruction_type, prompt, id_pattern, options, message): (
                _,
                _,
                _,
                Vec<(char, &str)>,
                _,
            )| {
                let (message, capacity) = match message {
                    Some(message) => {
                        let message = message.trim();
                        let (message, capacity) = if let Some(message) = message.strip_suffix('<') {
                            (message.trim_end(), None)
                        } else if let Some(message) = message.strip_suffix('>') {
                            (message.trim_end(), Some(1))
                        } else {
                            (message, Some(2))
                        };

                        ((!message.is_empty()).then_some(message), capacity)
                    }
                    None => (None, Some(2)),
                };

                let message = message.map(string_to_instruction_message);

                let content = capacity.map(Vec::with_capacity);

                let mut replicates: Option<u64> = None;
                let mut minimum_score: Option<u64> = None;
                let mut temperature: Option<u64> = None;
                let mut quality_weight: Option<u64> = None;
                let mut speed_weight: Option<u64> = None;
                let mut cost_weight: Option<u64> = None;
                for (tag, value) in options {
                    let value = value.parse().ok();
                    match tag {
                        'x' => replicates = value,
                        'y' => minimum_score = value,
                        't' => temperature = value,
                        'q' => quality_weight = value,
                        's' => speed_weight = value,
                        'c' => cost_weight = value,
                        _ => {}
                    }
                }

                let model = if id_pattern.is_some()
                    || minimum_score.is_some()
                    || temperature.is_some()
                    || quality_weight.is_some()
                    || speed_weight.is_some()
                    || cost_weight.is_some()
                {
                    Some(Box::new(InstructionModel {
                        id_pattern: id_pattern.map(String::from),
                        minimum_score,
                        temperature,
                        quality_weight,
                        speed_weight,
                        cost_weight,
                        ..Default::default()
                    }))
                } else {
                    None
                };

                Block::InstructionBlock(InstructionBlock {
                    instruction_type,
                    message,
                    content,
                    prompt: prompt.map(String::from),
                    replicates,
                    model,
                    ..Default::default()
                })
            },
        )
        .parse_next(input)
}

/// Parse a [`SuggestionBlock`] node
fn suggestion_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (alt(("suggestion", "suggest")), multispace0),
        opt(take_while(1.., |_| true)),
    )
    .map(|feedback: Option<&str>| {
        let (feedback, capacity) = match feedback {
            Some(feedback) => {
                let feedback = feedback.trim();
                let (feedback, capacity) = if let Some(feedback) = feedback.strip_suffix('<') {
                    (feedback.trim_end(), 0)
                } else if let Some(feedback) = feedback.strip_suffix('>') {
                    (feedback.trim_end(), 1)
                } else {
                    (feedback, 2)
                };

                ((!feedback.is_empty()).then_some(feedback), capacity)
            }
            None => (None, 2),
        };

        let content = Vec::with_capacity(capacity);

        Block::SuggestionBlock(SuggestionBlock {
            feedback: feedback.map(String::from),
            content,
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`InsertBlock`] node
fn insert_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (alt(("insert", "ins")), multispace0),
        opt(take_while(1.., |_| true)),
    )
    .map(|feedback| {
        Block::InsertBlock(InsertBlock {
            feedback: feedback.map(String::from),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`DeleteBlock`] node
fn delete_block(input: &mut Located<&str>) -> PResult<Block> {
    preceded(
        (alt(("delete", "del")), multispace0),
        opt(take_while(1.., |_| true)),
    )
    .map(|feedback| {
        Block::DeleteBlock(DeleteBlock {
            feedback: feedback.map(String::from),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`ReplaceBlock`] node
fn replace_block(input: &mut Located<&str>) -> PResult<Block> {
    delimited(
        (alt(("replace", "rep")), multispace0),
        opt(take_while(1.., |_| true)),
        opt(delimited(multispace0, "::: with", multispace0)),
    )
    .map(|feedback| {
        Block::ReplaceBlock(ReplaceBlock {
            feedback: feedback.map(String::from),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`ModifyBlock`] node
fn modify_block(input: &mut Located<&str>) -> PResult<Block> {
    delimited(
        (alt(("modify", "mod")), multispace0),
        opt(take_while(1.., |_| true)),
        opt(delimited(multispace0, "::: with", multispace0)),
    )
    .map(|feedback| {
        Block::ModifyBlock(ModifyBlock {
            feedback: feedback.map(String::from),
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
            label_automatically: label.is_some().then_some(false),
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
            alt(("else", "{else}")).map(|_| Divider::Else),
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
    if let Block::SuggestionBlock(SuggestionBlock { content, .. })
    | Block::DeleteBlock(DeleteBlock { content, .. })
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
                chunk.execution_mode = inner.execution_mode;
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
            chunk.label_automatically = figure.label_automatically;
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
        if let Some(blocks) = content {
            if blocks.is_empty() && children.is_empty() {
                *content = None;
            } else {
                blocks.append(&mut children);
            }
        } else {
            *content = (!children.is_empty()).then_some(children);
        }
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
            chunk.label_automatically = table.label_automatically;
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

/// Get the execution mode from block options
fn execution_mode_from_options(options: IndexMap<&str, Option<Node>>) -> Option<ExecutionMode> {
    for (name, value) in options {
        if matches!(name, "always" | "auto" | "locked" | "lock") && value.is_none() {
            return name.parse().ok();
        }
    }
    None
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

        mdast::Node::Code(code) => {
            let position = code.position.clone();
            let block = myst_to_block(&code).unwrap_or_else(|| code_to_block(code));

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

/// Transform a [`mdast::Code`] node to a block if it is a recognized MyST directive
///
/// Note that `if`, `elif`, `else`, and `for` directives are are handled elsewhere
/// because they do not always have closing semicolons (e.g. if followed by a elif)
fn myst_to_block(code: &mdast::Code) -> Option<Block> {
    // If no `lang` after backticks then not a MyST directive
    let lang = code.lang.as_deref()?;

    // Attempt to get name of the directive from `lang`
    let name = if lang.starts_with('{') && lang.ends_with('}') {
        &lang[1..(lang.len() - 1)]
    } else {
        return None;
    };

    let args = code.meta.as_deref();

    // Extract directive options and separate them from the value of the directive
    let mut options: HashMap<&str, &str> = HashMap::new();
    let mut value = String::new();
    for line in code.value.lines() {
        if line.starts_with(':') && line.chars().filter(|&c| c == ':').count() > 1 {
            let (key, value) = line[1..].split_once(':').unwrap_or_default();
            options.insert(key.trim(), value.trim());
        } else {
            value.push_str(line);
            if !value.is_empty() {
                value.push('\n');
            }
        }
    }
    if value.ends_with('\n') {
        value.pop();
    }

    // Transform into a Stencila block based on the name

    if let Some(claim_type) = name.strip_prefix("prf:") {
        return Some(Block::Claim(Claim {
            claim_type: claim_type.parse().unwrap_or_default(),
            label: options.get("label").map(|label| label.to_string()),
            content: decode_blocks(&value),
            ..Default::default()
        }));
    }

    Some(match name {
        "admonition" | "attention" | "caution" | "danger" | "error" | "failure" | "hint"
        | "important" | "info" | "note" | "seealso" | "success" | "tip" | "warning" => {
            Block::Admonition(Admonition {
                // Handle MyST admonition types for which there is not a 1:1 mapping,
                // parse the rest
                admonition_type: match name {
                    "attention" => AdmonitionType::Important,
                    "caution" => AdmonitionType::Warning,
                    "hint" => AdmonitionType::Tip,
                    "seealso" => AdmonitionType::Note,
                    type_ => type_.parse().unwrap_or_default(),
                },
                content: decode_blocks(&value),
                title: args.map(decode_inlines),
                is_folded: options.get("class").map(|&class| class == "dropdown"),
                ..Default::default()
            })
        }
        "code-cell" | "mermaid" => {
            let programming_language = match name {
                "mermaid" => Some(name.to_string()),
                _ => args.map(String::from),
            };

            Block::CodeChunk(CodeChunk {
                code: value.into(),
                programming_language,
                execution_mode: options.get("mode").and_then(|mode| mode.parse().ok()),
                label_type: options.get("type").and_then(|&type_| match type_ {
                    "figure" => Some(LabelType::FigureLabel),
                    "table" => Some(LabelType::TableLabel),
                    _ => None,
                }),
                label: options.get("label").map(|label| label.to_string()),
                label_automatically: options.contains_key("label").then_some(false),
                caption: options
                    .get("caption")
                    .map(|&caption| decode_blocks(caption)),
                ..Default::default()
            })
        }
        "figure" => {
            use shortcuts::{img, p};
            let content = code
                .meta
                .as_ref()
                .map(|url| vec![p([img(url)])])
                .unwrap_or_default();
            let caption = decode_blocks(&value);

            Block::Figure(Figure {
                label: options.get("label").map(|label| label.to_string()),
                label_automatically: options.contains_key("label").then_some(false),
                caption: (!caption.is_empty()).then_some(caption),
                content,
                ..Default::default()
            })
        }
        "table" => {
            let rows = if let Some(Block::Table(Table { rows, .. })) = decode_blocks(&value).first()
            {
                rows.clone()
            } else {
                Vec::new()
            };

            Block::Table(Table {
                label: options.get("label").map(|label| label.to_string()),
                label_automatically: options.contains_key("label").then_some(false),
                caption: args.map(decode_blocks),
                rows,
                ..Default::default()
            })
        }
        "include" => Block::IncludeBlock(IncludeBlock {
            source: args.unwrap_or_default().to_string(),
            execution_mode: options.get("mode").and_then(|mode| mode.parse().ok()),
            media_type: options.get("format").map(|format| format.to_string()),
            select: options.get("select").map(|select| select.to_string()),
            ..Default::default()
        }),
        "new" | "edit" | "fix" | "describe" => Block::InstructionBlock(InstructionBlock {
            instruction_type: name.parse().unwrap_or_default(),
            message: args.map(InstructionMessage::from),
            prompt: options.get("prompt").map(|value| value.to_string()),
            replicates: options.get("reps").and_then(|value| value.parse().ok()),
            content: if !value.trim().is_empty() {
                Some(decode_blocks(&value))
            } else {
                None
            },
            ..Default::default()
        }),
        "suggest" => Block::SuggestionBlock(SuggestionBlock {
            feedback: args.map(|value| value.to_string()),
            content: decode_blocks(&value),
            ..Default::default()
        }),
        "style" => Block::StyledBlock(StyledBlock {
            code: args.unwrap_or_default().into(),
            content: decode_blocks(&value),
            ..Default::default()
        }),
        _ => {
            // Fallback to code block that will preserve
            let mut lang = lang.to_string();
            if let Some(rest) = args {
                lang.push(' ');
                lang.push_str(rest);
            }

            Block::CodeBlock(CodeBlock {
                programming_language: Some(lang),
                code: value.into(),
                ..Default::default()
            })
        }
    })
}

/// Transform a [`mdast::Code`] node to a Stencila [`Block`]
fn code_to_block(code: mdast::Code) -> Block {
    let mdast::Code {
        lang, meta, value, ..
    } = code;

    let meta = meta.unwrap_or_default();
    let is_exec = meta.starts_with("exec") || lang.as_deref() == Some("exec");
    let is_raw = meta.starts_with("raw") || lang.as_deref() == Some("raw");

    if is_exec {
        let mut meta = meta.strip_prefix("exec").unwrap_or_default().trim();
        let is_invisible = value.contains("@invisible").then_some(true);

        Block::CodeChunk(CodeChunk {
            code: value.into(),
            programming_language: if lang.as_deref() == Some("exec") {
                None
            } else {
                lang
            },
            execution_mode: execution_mode(&mut meta).ok(),
            is_invisible,
            ..Default::default()
        })
    } else if is_raw {
        let format = lang
            .and_then(|lang| (lang != "raw").then_some(lang))
            .unwrap_or("markdown".to_string());

        Block::RawBlock(RawBlock {
            content: value.into(),
            format,
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
    }
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
    use codec::schema::{ClaimType, ExecutionMode, Node};
    use common_dev::pretty_assertions::assert_eq;

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
                "for row in select * from table { sql, auto }"
            ))
            .unwrap(),
            Block::ForBlock(ForBlock {
                variable: "row".to_string(),
                code: "select * from table".into(),
                programming_language: Some("sql".to_string()),
                execution_mode: Some(ExecutionMode::Auto),
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
