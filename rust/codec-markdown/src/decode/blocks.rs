use std::{collections::HashMap, str::FromStr};

use indexmap::IndexMap;
use inflector::Inflector;
use markdown::{
    mdast::{self, AlignKind},
    unist::Position,
};
use serde_json::json;
use winnow::{
    LocatingSlice as Located, ModalResult, Parser,
    ascii::{Caseless, multispace0, multispace1, space0},
    combinator::{alt, delimited, eof, opt, preceded, repeat, separated, terminated},
    stream::AsChar,
    token::{take, take_till, take_until, take_while},
};

use stencila_codec::{
    stencila_format::Format,
    stencila_schema::{
        Admonition, AdmonitionType, AppendixBreak, Author, Block, CallArgument, CallBlock, Chat,
        ChatMessage, ChatMessageGroup, ChatMessageOptions, Claim, CodeBlock, CodeChunk,
        CodeExpression, ExecutionBounds, ExecutionMode, Figure, FigureOptions, ForBlock, Heading,
        HorizontalAlignment, IfBlock, IfBlockClause, ImageObject, IncludeBlock, Inline,
        InstructionBlock, InstructionMessage, LabelType, List, ListItem, ListOrder, MathBlock,
        Node, Page, Paragraph, PromptBlock, QuoteBlock, RawBlock, Section, SoftwareApplication,
        StyledBlock, SuggestionBlock, SuggestionStatus, SuggestionType, Table, TableCell,
        TableCellOptions, TableCellType, TableRow, TableRowType, Text, ThematicBreak, Walkthrough,
        WalkthroughStep,
    },
};

use crate::decode::shared::suggestion_metadata_from_attrs;

use super::{
    Context, decode_blocks, decode_inlines,
    inlines::{inlines, mds_to_inlines, mds_to_string},
    shared::{
        Attrs, attrs, attrs_list, block_node_type, execution_bounds, execution_mode,
        instruction_type, is_executable_language, model_parameters, name, node_to_string,
        primitive_node, prompt, relative_position,
    },
};

/// Transform MDAST nodes to Stencila Schema `Block`
pub(super) fn mds_to_blocks(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut boundaries: Vec<usize> = Vec::new();

    // Get all the blocks since the last boundary
    fn pop_blocks(blocks: &mut Vec<Block>, boundaries: &mut Vec<usize>) -> Vec<Block> {
        if let Some(boundary) = boundaries.pop() {
            if boundary > blocks.len() {
                Vec::new()
            } else {
                blocks.drain(boundary..).collect()
            }
        } else {
            Vec::new()
        }
    }

    // Used to avoid running `fold_block` if there are no instructions or suggestions
    let mut has_walkthrough = false;
    let mut should_fold = false;
    for md in mds.into_iter() {
        let mut is_handled = false;

        // Get paragraph children and position for fence checks below
        let mut para = None;
        if let mdast::Node::Paragraph(mdast::Paragraph { children, position }) = &md {
            para = Some((children, position));
        }

        // Handle fence paragraphs starting with `:::`, `:++`, `:--`, `:~~`, `:~>`, or `/`
        if let Some((true, children, position)) = para.map(|(children, position)| {
            let value = mds_to_string(children);
            (
                value.starts_with(":::")
                    || value.starts_with(":++")
                    || value.starts_with(":--")
                    || value.starts_with(":~~")
                    || value.starts_with(":~>")
                    || value.starts_with("/"),
                children,
                position,
            )
        }) {
            // Serialize MDAST nodes to text so that fragments such as
            // URLS (for `::: include`), and `$`s for styles with interpolated variables
            // which are otherwise consumed into non text nodes are included in parse value
            let value = mds_to_string(children);

            if let Ok(divider) = divider(&mut value.as_str(), !boundaries.is_empty()) {
                let children = pop_blocks(&mut blocks, &mut boundaries);

                match divider {
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
                                        tracing::error!(
                                            "Expected there to be at least one if clause already"
                                        )
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

                                _ => tracing::warn!(
                                    "Found an `::: else` without a preceding `::: if` or `::: for`"
                                ),
                            }
                        }

                        boundaries.push(blocks.len());
                    }
                    Divider::Next => {
                        if let Some(position) = position {
                            context.map_end(position.end.offset);
                        }

                        if let Some(last_block) = blocks.last_mut() {
                            finalize(last_block, children, context);
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
                            if matches!(last_block, Block::IfBlock(..))
                                && let Some(position) = position
                            {
                                context.map_end(position.end.offset);
                            }
                        }
                    }
                }

                is_handled = true;
            } else if let Ok((is_if, if_clause)) = if_elif(&mut Located::new(value.as_str())) {
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
                            tracing::error!("Expected there to be at least one if clause already")
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
                    if let Block::InstructionBlock(InstructionBlock { content, .. }) = &block {
                        match content {
                            Some(content) => content.capacity() != 1,
                            None => false,
                        }
                    } else {
                        !matches!(
                            block,
                            Block::AppendixBreak(..)
                                | Block::IncludeBlock(..)
                                | Block::CallBlock(..)
                                | Block::PromptBlock(..)
                        )
                    };

                // Add boundary and map position
                if add_boundary {
                    boundaries.push(blocks.len() + 1);
                    if let (Some(position), Some(node_id)) = (position, block.node_id()) {
                        context.map_start(position.start.offset, block.node_type(), node_id);
                    }
                } else {
                    context.map_position(position, block.node_type(), block.node_id());
                }

                blocks.push(block);
                is_handled = true;
            }
        }

        if is_handled {
            continue;
        }

        // Handle walkthrough steps
        if para
            .map(|(children, ..)| mds_to_string(children) == "...")
            .unwrap_or_default()
        {
            let content = pop_blocks(&mut blocks, &mut boundaries);

            if let Some(Block::Walkthrough(walkthrough)) = blocks.last_mut() {
                walkthrough.steps.push(WalkthroughStep {
                    content,
                    ..Default::default()
                });
            } else {
                blocks.push(Block::Walkthrough(Walkthrough {
                    ..Default::default()
                }));
            }

            boundaries.push(blocks.len());
            has_walkthrough = true;

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
                && !matches!(next, Block::SuggestionBlock(..))
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

                match blocks.get_mut(index) {
                    Some(Block::InstructionBlock(InstructionBlock {
                        content: Some(content),
                        ..
                    })) => content.push(next),
                    Some(Block::SuggestionBlock(suggestion)) => {
                        suggestion_content_mut(suggestion).push(next)
                    }
                    _ => {}
                }

                step = false;
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
                && !matches!(next, Block::SuggestionBlock(..))
                && let Some(suggestion) = suggestions.last()
                && let content = suggestion_content(suggestion)
                && content.capacity() == 1
                && content.is_empty()
            {
                if let (Some(current_id), Some(next_id)) = (
                    blocks.get(index).and_then(|block| block.node_id()),
                    blocks.get(index + 1).and_then(|block| block.node_id()),
                ) && let Some(suggestion) = suggestions.last()
                {
                    let suggestion_id = suggestion.node_id();
                    context.map_extend(suggestion_id.clone(), next_id.clone());
                    context.map_extend(current_id, suggestion_id);
                }

                let next = blocks.remove(index + 1);

                if let Some(Block::InstructionBlock(InstructionBlock {
                    suggestions: Some(suggestions),
                    ..
                })) = blocks.get_mut(index)
                    && let Some(suggestion) = suggestions.last_mut()
                {
                    suggestion_content_mut(suggestion).push(next);
                }

                step = false;
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
                        if let Some(original) = &mut suggestion.original {
                            fold_blocks(original, context);
                        }
                        fold_blocks(&mut suggestion.content, context);
                    }
                } else if let Block::Walkthrough(Walkthrough { steps, .. }) = block {
                    for step in steps {
                        fold_blocks(&mut step.content, context);
                    }
                }
            }

            if step {
                index += 1;
            }
        }
    }

    // Finish walkthrough if necessary. This must be done after folding blocks.
    if has_walkthrough {
        let content = pop_blocks(&mut blocks, &mut boundaries);

        if let Some(Block::Walkthrough(walkthrough)) = blocks.last_mut() {
            walkthrough.steps.push(WalkthroughStep {
                content,
                ..Default::default()
            });
        }
    }

    blocks
}

fn suggestion_waiting_for_original(suggestion: &SuggestionBlock) -> bool {
    suggestion.suggestion_type == Some(SuggestionType::Replace)
        && suggestion
            .original
            .as_ref()
            .is_some_and(|original| original.is_empty())
}

fn suggestion_content_mut(suggestion: &mut SuggestionBlock) -> &mut Vec<Block> {
    if suggestion_waiting_for_original(suggestion) {
        suggestion.original.get_or_insert_default()
    } else {
        &mut suggestion.content
    }
}

fn suggestion_content(suggestion: &SuggestionBlock) -> &Vec<Block> {
    if suggestion_waiting_for_original(suggestion) {
        suggestion.original.as_ref().unwrap_or(&suggestion.content)
    } else {
        &suggestion.content
    }
}

/// Parse a "div": a paragraph starting with at least three colons, or `:++`/`:--`/`:~~` for suggestions
fn block(input: &mut Located<&str>) -> ModalResult<Block> {
    alt((
        chat,
        suggestion_block_critic,
        preceded(
            (take_while(3.., ':'), space0),
            alt((
                admonition_qmd,
                styled_block_qmd,
                appendix_break,
                call_block,
                include_block,
                prompt_block,
                code_chunk,
                figure,
                table,
                for_block,
                instruction_block,
                page,
                suggestion_block,
                chat_message,
                claim,
                styled_block,
                // Section parser is permissive of label so needs to
                // come last to avoid prematurely matching others above
                section,
            )),
        ),
    ))
    .parse_next(input)
}

/// Parse a [`Admonition`] node
fn admonition_qmd(input: &mut Located<&str>) -> ModalResult<Block> {
    delimited(
        "{.callout-",
        (
            take_while(0.., AsChar::is_alpha),
            opt(preceded(multispace1, attrs_list)),
        ),
        "}",
    )
    .map(|(admonition_type, options)| {
        let mut options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

        let is_folded = options
            .swap_remove("collapse")
            .and_then(|value| match value {
                Some(Node::Boolean(value)) => Some(value),
                Some(Node::String(value)) => match value.as_str() {
                    "true" | "yes" => Some(true),
                    "false" | "no" => Some(false),
                    _ => None,
                },
                _ => None,
            });

        let title = options.swap_remove("title").and_then(|value| match value {
            Some(Node::String(value)) => {
                // This is a QMD-specific parser, so use QMD format
                Some(
                    inlines(&value, &Format::Qmd)
                        .into_iter()
                        .map(|(node, ..)| node)
                        .collect(),
                )
            }
            _ => None,
        });

        Block::Admonition(Admonition {
            admonition_type: AdmonitionType::from_str(admonition_type).unwrap_or_default(),
            is_folded,
            title,
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`AppendixBreak`] node
fn appendix_break(input: &mut Located<&str>) -> ModalResult<Block> {
    ("appendix", multispace0)
        .map(|_| Block::AppendixBreak(AppendixBreak::default()))
        .parse_next(input)
}

/// Parse an argument to a `CallBlock`.
///
/// Arguments must be key-value or key-symbol pairs separated by `=`.
fn call_arg(input: &mut Located<&str>) -> ModalResult<CallArgument> {
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
fn call_block(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded(
        ("call", multispace0),
        (
            take_till(0.., '('),
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
fn include_block(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded(
        ("include", multispace0),
        (take_while(0.., |c| c != '{'), opt(attrs)),
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

/// Parse a [`PromptBlock`] node
fn prompt_block(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded(
        "prompt",
        (
            opt(preceded(multispace1, instruction_type)),
            opt(preceded(multispace1, relative_position)),
            opt(preceded(multispace1, block_node_type)),
            opt(preceded(multispace1, prompt)),
            opt(take_while(1.., |_| true)),
        ),
    )
    .map(
        |(instruction_type, relative_position, node_type, target, query)| {
            let node_types = node_type.map(|node_type| vec![node_type]);

            let query = query.and_then(|query| {
                let query = query.trim();
                (!query.is_empty()).then_some(query.to_string())
            });

            Block::PromptBlock(PromptBlock {
                instruction_type,
                relative_position,
                node_types,
                target: target.map(String::from),
                query,
                ..Default::default()
            })
        },
    )
    .parse_next(input)
}

/// Parse a [`Claim`] node
fn claim(input: &mut Located<&str>) -> ModalResult<Block> {
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
fn code_chunk(input: &mut Located<&str>) -> ModalResult<Block> {
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

/// Parse a [`Chat`] node
fn chat(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded(
        "/",
        (
            opt(preceded(multispace0, instruction_type)),
            opt(preceded(multispace1, relative_position)),
            opt(preceded(multispace1, block_node_type)),
            opt(preceded(multispace1, prompt)),
            opt(preceded(multispace1, model_parameters)),
            opt(take_while(1.., |_| true)),
        ),
    )
    .map(
        |(instruction_type, relative_position, node_type, target, model_parameters, query)| {
            let node_types = node_type.map(|node_type| vec![node_type]);

            let query = query.and_then(|query| {
                let query = query.trim();
                (!query.is_empty()).then_some(query)
            });

            let prompt = PromptBlock {
                instruction_type,
                relative_position,
                node_types,
                target: target.map(String::from),
                query: query.map(String::from),
                ..Default::default()
            };

            let model_parameters = model_parameters.map(Box::new).unwrap_or_default();

            Block::Chat(Chat {
                prompt,
                model_parameters,
                is_embedded: Some(true),
                ..Default::default()
            })
        },
    )
    .parse_next(input)
}

/// Parse a [`ChatMessage`] or [`ChatMessageGroup`] node
fn chat_message(input: &mut Located<&str>) -> ModalResult<Block> {
    (
        preceded(
            Caseless("msg/"),
            alt((
                Caseless("system"),
                Caseless("user"),
                Caseless("model"),
                Caseless("group"),
            )),
        ),
        opt(delimited(
            (multispace0, '['),
            take_until(0.., ']'),
            (']', multispace0),
        )),
    )
        .map(|(role, author): (&str, Option<&str>)| {
            if role == "group" {
                Block::ChatMessageGroup(ChatMessageGroup {
                    ..Default::default()
                })
            } else {
                let author = author.map(|author| {
                    Author::SoftwareApplication(SoftwareApplication {
                        id: Some(author.into()),
                        ..Default::default()
                    })
                });

                Block::ChatMessage(ChatMessage {
                    role: role.parse().ok().unwrap_or_default(),
                    options: Box::new(ChatMessageOptions {
                        author,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            }
        })
        .parse_next(input)
}

/// A property parsed from a figure or table opening fence line.
enum FenceProperty<'s> {
    Id(&'s str),
    Label(&'s str),
    Layout(&'s str),
    Attrs(Vec<(&'s str, Option<Node>)>),
}

/// Parsed fields from fence properties.
struct FenceFields {
    id: Option<String>,
    label: Option<String>,
    layout: Option<String>,
    attrs: IndexMap<String, Option<Node>>,
}

/// Parse a `#id` property from a fence line.
fn fence_id<'s>(input: &mut Located<&'s str>) -> ModalResult<FenceProperty<'s>> {
    preceded(
        '#',
        take_while(1.., |c: char| !c.is_whitespace() && c != '[' && c != '{'),
    )
    .map(FenceProperty::Id)
    .parse_next(input)
}

/// Parse a `[layout]` property from a fence line.
fn fence_layout<'s>(input: &mut Located<&'s str>) -> ModalResult<FenceProperty<'s>> {
    delimited('[', take_until(1.., ']'), ']')
        .map(FenceProperty::Layout)
        .parse_next(input)
}

/// Parse a `{attrs}` property from a fence line.
fn fence_attrs<'s>(input: &mut Located<&'s str>) -> ModalResult<FenceProperty<'s>> {
    attrs.map(FenceProperty::Attrs).parse_next(input)
}

/// Parse a label token from a fence line (greedy, stops at special chars).
fn fence_label<'s>(input: &mut Located<&'s str>) -> ModalResult<FenceProperty<'s>> {
    take_while(1.., |c: char| {
        !c.is_whitespace() && c != '#' && c != '[' && c != '{'
    })
    .map(FenceProperty::Label)
    .parse_next(input)
}

/// Single char fallback to mop up unmatched characters.
fn fence_char<'s>(input: &mut Located<&'s str>) -> ModalResult<FenceProperty<'s>> {
    take(1usize)
        .map(|c: &str| FenceProperty::Label(c))
        .parse_next(input)
}

/// Parse figure fence properties: id, label, layout, attrs in any order.
fn figure_properties<'s>(input: &mut Located<&'s str>) -> ModalResult<Vec<FenceProperty<'s>>> {
    repeat(
        0..,
        preceded(
            multispace0,
            alt((fence_id, fence_layout, fence_attrs, fence_label, fence_char)),
        ),
    )
    .parse_next(input)
}

/// Parse table fence properties: id and label in any order.
fn table_properties<'s>(input: &mut Located<&'s str>) -> ModalResult<Vec<FenceProperty<'s>>> {
    repeat(
        0..,
        preceded(multispace0, alt((fence_id, fence_label, fence_char))),
    )
    .parse_next(input)
}

/// Collect parsed fence properties into structured fields.
fn collect_fence_fields(props: Vec<FenceProperty>) -> FenceFields {
    let mut id = None;
    let mut label_parts = Vec::new();
    let mut layout = None;
    let mut attrs = IndexMap::new();
    for prop in props {
        match prop {
            FenceProperty::Id(v) => {
                if id.is_none() {
                    id = Some(v.to_string())
                }
            }
            FenceProperty::Label(v) => label_parts.push(v),
            FenceProperty::Layout(v) => layout = Some(v.to_string()),
            FenceProperty::Attrs(a) => {
                for (k, v) in a {
                    attrs.insert(k.to_string(), v);
                }
            }
        }
    }
    let label = if label_parts.is_empty() {
        None
    } else {
        Some(label_parts.join(" "))
    };
    FenceFields {
        id,
        label,
        layout,
        attrs,
    }
}

/// Parse a [`Figure`] node with a label and/or caption
fn figure(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded(
        (
            alt((Caseless("figure"), Caseless("fig"), Caseless("fig."))),
            multispace0,
        ),
        figure_properties,
    )
    .map(|props| {
        let mut fields = collect_fence_fields(props);
        Block::Figure(Figure {
            id: fields.id,
            label: fields.label.clone(),
            label_automatically: fields.label.is_some().then_some(false),
            options: Box::new(FigureOptions {
                layout: fields.layout,
                padding: fields
                    .attrs
                    .swap_remove("pad")
                    .flatten()
                    .map(node_to_string),
                ..Default::default()
            }),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`ForBlock`] node
fn for_block(input: &mut Located<&str>) -> ModalResult<Block> {
    alt((
        // Stencila Markdown
        preceded(
            ("for", multispace0),
            (
                opt(name),
                opt((multispace0, "in", multispace0)),
                alt((
                    delimited('`', take_until(0.., '`'), '`'),
                    take_while(0.., |c| c != '{'),
                )),
                opt(preceded(multispace0, attrs)),
            ),
        ),
        // MyST
        preceded(
            ("{for}", multispace0),
            (
                opt(name),
                opt((multispace0, "in", multispace0)),
                take_while(0.., |c| c != '{'),
                "".value(None),
            ),
        ),
    ))
    .map(|(variable, _, expr, options)| {
        let options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

        Block::ForBlock(ForBlock {
            variable: variable.map(|var| var.into()).unwrap_or_default(),
            code: expr.trim().into(),
            programming_language: options.first().map(|(name, _)| name.to_string()),
            execution_mode: execution_mode_from_options(options),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse an `if` or `elif` fenced div into an [`IfBlockClause`]
fn if_elif(input: &mut Located<&str>) -> ModalResult<(bool, IfBlockClause)> {
    alt((
        // Stencila Markdown
        (
            delimited(
                (take_while(3.., ':'), space0),
                alt(("if", "elif")),
                multispace0,
            ),
            alt((
                delimited('`', take_until(0.., '`'), '`'),
                take_while(0.., |c| c != '{'),
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
            take_while(0.., |_| true),
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
fn instruction_block(input: &mut Located<&str>) -> ModalResult<Block> {
    (
        instruction_type,
        opt(preceded(multispace1, execution_mode)),
        opt(preceded(multispace1, relative_position)),
        opt(preceded(multispace1, block_node_type)),
        opt(preceded(multispace1, prompt)),
        opt(preceded(multispace1, model_parameters)),
        opt(take_while(1.., |_| true)),
    )
        .map(
            |(
                instruction_type,
                execution_mode,
                relative_position,
                node_type,
                prompt,
                model_parameters,
                query,
            )| {
                let node_types = node_type.map(|node_type| vec![node_type.to_string()]);

                let mut prompt = PromptBlock {
                    instruction_type: Some(instruction_type),
                    relative_position,
                    node_types,
                    target: prompt.map(String::from),
                    query: query.map(String::from),
                    ..Default::default()
                };

                let model_parameters = model_parameters.map(Box::new).unwrap_or_default();

                let (message, capacity) = match query {
                    Some(message) => {
                        let message = message.trim();

                        let (message, capacity) = if let Some(message) = message.strip_suffix(":::")
                        {
                            (message.trim_end(), None)
                        } else if let Some(message) = message.strip_suffix(">>>") {
                            (message.trim_end(), Some(1))
                        } else {
                            (message, Some(2))
                        };

                        let message = (!message.is_empty()).then_some(message);

                        // Use the message as the query for the target prompt
                        prompt.query = message.map(String::from);

                        (message, capacity)
                    }
                    None => (None, Some(2)),
                };

                let message = InstructionMessage::new(decode_inlines(
                    message.unwrap_or_default(),
                    &mut Context::new(Format::Markdown),
                ));

                let content = capacity.map(Vec::with_capacity);

                Block::InstructionBlock(InstructionBlock {
                    instruction_type,
                    prompt,
                    message,
                    model_parameters,
                    content,
                    execution_mode,
                    ..Default::default()
                })
            },
        )
        .parse_next(input)
}

/// Parse a [`SuggestionBlock`] node using Critic Markup fence syntax (`:++`, `:--`, or `:~~`)
fn suggestion_block_critic(input: &mut Located<&str>) -> ModalResult<Block> {
    (
        alt((
            ":++".value(SuggestionType::Insert),
            ":--".value(SuggestionType::Delete),
            ":~~".value(SuggestionType::Replace),
        )),
        opt(preceded(multispace0, attrs)),
        opt(preceded(
            multispace0,
            alt((
                "accept".value(SuggestionStatus::Accepted),
                "reject".value(SuggestionStatus::Rejected),
            )),
        )),
        opt(preceded(multispace0, take_while(1.., |_| true))),
    )
        .map(
            |(suggestion_type, attrs, suggestion_status, feedback): (
                SuggestionType,
                Option<Attrs>,
                Option<SuggestionStatus>,
                Option<&str>,
            )| {
                let feedback = feedback.map(|f| f.trim()).filter(|f| !f.is_empty());
                let (authors, date_published) = suggestion_metadata_from_attrs(attrs);

                let mut block = SuggestionBlock {
                    suggestion_type: Some(suggestion_type),
                    suggestion_status,
                    authors,
                    date_published,
                    feedback: feedback.map(String::from),
                    content: Vec::new(),
                    ..Default::default()
                };

                if suggestion_type == SuggestionType::Replace {
                    block.original = Some(Vec::new());
                }

                Block::SuggestionBlock(block)
            },
        )
        .parse_next(input)
}

/// Parse a [`SuggestionBlock`] node using `::: suggest` syntax
fn suggestion_block(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded(
        ("suggest", multispace0),
        (
            opt(terminated(attrs, multispace0)),
            opt(terminated(
                alt((
                    "accept".value(SuggestionStatus::Accepted),
                    "reject".value(SuggestionStatus::Rejected),
                )),
                multispace0,
            )),
            opt(take_while(1.., |_| true)),
        ),
    )
    .map(
        |(attrs, suggestion_status, feedback): (
            Option<Attrs>,
            Option<SuggestionStatus>,
            Option<&str>,
        )| {
            let (feedback, capacity) = match feedback {
                Some(feedback) => {
                    let feedback = feedback.trim();
                    let (feedback, capacity) = if let Some(feedback) = feedback.strip_suffix("<<") {
                        (feedback.trim_end(), 0)
                    } else if let Some(feedback) = feedback.strip_suffix(">>") {
                        (feedback.trim_end(), 1)
                    } else {
                        (feedback, 2)
                    };

                    ((!feedback.is_empty()).then_some(feedback), capacity)
                }
                None => (None, 2),
            };

            let (authors, date_published) = suggestion_metadata_from_attrs(attrs);

            let content = Vec::with_capacity(capacity);

            Block::SuggestionBlock(SuggestionBlock {
                suggestion_status,
                authors,
                date_published,
                feedback: feedback.map(String::from),
                content,
                ..Default::default()
            })
        },
    )
    .parse_next(input)
}

/// Parse a [`Section`] node
fn section(input: &mut Located<&str>) -> ModalResult<Block> {
    take_while(1.., |_| true)
        .map(|section_type: &str| {
            // Allow for alternative casing of section type by converting to
            // casing expected by `SectionType::from_str`
            let section_type = section_type.to_pascal_case().parse().ok();

            Block::Section(Section {
                section_type,
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a [`StyledBlock`] node
fn styled_block(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded(
        (alt((Caseless("styled"), Caseless("style"))), multispace0),
        take_while(0.., |_| true),
    )
    .map(|code: &str| {
        Block::StyledBlock(StyledBlock {
            code: code.trim().into(),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a [`StyledBlock`] node from QMD format (e.g. `::: {.class-a .class-b}`)
fn styled_block_qmd(input: &mut Located<&str>) -> ModalResult<Block> {
    delimited('{', take_until(0.., '}'), '}')
        .map(|code: &str| {
            Block::StyledBlock(StyledBlock {
                code: code.trim().into(),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a [`Page`] node
fn page(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded((Caseless("page"), multispace0), take_while(0.., |_| true))
        .map(|code: &str| {
            Block::Page(Page {
                code: code.trim().into(),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a [`Table`] with a label and/or caption
fn table(input: &mut Located<&str>) -> ModalResult<Block> {
    preceded((Caseless("table"), multispace0), table_properties)
        .map(|props| {
            let fields = collect_fence_fields(props);
            Block::Table(Table {
                id: fields.id,
                label: fields.label.clone(),
                label_automatically: fields.label.is_some().then_some(false),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a divider between sections of content.
///
/// The `has_open_block` parameter controls whether `:++`/`:--`/`:~~` are recognized as
/// end dividers. These tokens serve as both openers and closers for suggestion blocks,
/// so they should only match as closers when there is an open block to close.
fn divider(input: &mut &str, has_open_block: bool) -> ModalResult<Divider> {
    if has_open_block {
        alt((
            // Critic Markup style suggestion block dividers: `:++`, `:--`, `:~~` or `:~>`
            delimited(alt((":++", ":--", ":~~")), space0, eof).map(|_| Divider::End),
            delimited(":~>", space0, eof).map(|_| Divider::Next),
            // Standard colon fence dividers
            delimited(
                (take_while(3.., ':'), space0),
                alt((
                    alt(("else", "{else}")).map(|_| Divider::Else),
                    "".map(|_| Divider::End),
                )),
                (space0, eof),
            ),
        ))
        .parse_next(input)
    } else {
        delimited(
            (take_while(3.., ':'), space0),
            alt((
                alt(("else", "{else}")).map(|_| Divider::Else),
                "".map(|_| Divider::End),
            )),
            (space0, eof),
        )
        .parse_next(input)
    }
}

#[derive(Debug, PartialEq)]
enum Divider {
    Else,
    Next,
    End,
}

/// Finalize a block by assigning children etc
fn finalize(parent: &mut Block, mut children: Vec<Block>, context: &mut Context) {
    if let Block::SuggestionBlock(suggestion) = parent {
        if suggestion_waiting_for_original(suggestion) {
            suggestion.original = Some(children);
        } else {
            suggestion.content = children;
        }
    } else if let Block::ChatMessage(ChatMessage { content, .. })
    | Block::Claim(Claim { content, .. })
    | Block::Page(Page { content, .. })
    | Block::Section(Section { content, .. })
    | Block::StyledBlock(StyledBlock { content, .. }) = parent
    {
        // Parent div is a node type where we just have to assign children
        // to content.
        *content = children;
    } else if let Block::ChatMessageGroup(ChatMessageGroup { messages, .. }) = parent {
        // Filter to only include chat messages
        *messages = children
            .into_iter()
            .filter_map(|block| match block {
                Block::ChatMessage(message) => Some(message),
                _ => None,
            })
            .collect();
    } else if let Block::Admonition(admonition) = parent {
        if matches!(context.format, Format::Qmd) {
            for block in children {
                if let Block::Heading(Heading {
                    content: inlines, ..
                }) = block
                {
                    admonition.title = Some(inlines);
                } else {
                    admonition.content.push(block);
                }
            }
        } else {
            admonition.content = children;
        }
    } else if let Block::CodeChunk(chunk) = parent {
        // Parent div code chunk with label and caption etc
        for child in children {
            if let Block::CodeChunk(inner) = child {
                let node_id = inner.node_id();
                chunk.programming_language = inner.programming_language;
                chunk.execution_mode = inner.execution_mode;
                chunk.execution_bounds = inner.execution_bounds;
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
        let code_chunk_count = children
            .iter()
            .filter(|block| matches!(block, Block::CodeChunk(..)))
            .count();
        let has_subfigure = children
            .iter()
            .any(|block| matches!(block, Block::Figure(..)));

        if code_chunk_count == 1 && !has_subfigure {
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
            extract_code_chunk_overlay(&mut chunk);

            // Replace the mapping entry for figure, with one for chunk
            context.map_remove(chunk.node_id());
            context.map_replace(figure.node_id(), chunk.node_type(), chunk.node_id());

            *parent = Block::CodeChunk(chunk);
        } else {
            // Put all paragraphs into the caption (unless they have just a single media object) and
            // everything else in the content
            let mut caption = vec![];
            let mut content = vec![];
            for child in children {
                if let Block::Paragraph(Paragraph {
                    content: inlines, ..
                }) = &child
                {
                    if let (
                        1,
                        Some(
                            Inline::ImageObject(..)
                            | Inline::AudioObject(..)
                            | Inline::VideoObject(..),
                        ),
                    ) = (inlines.len(), inlines.first())
                    {
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
            extract_figure_overlay(figure);
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
        if matches!(name, "always" | "auto" | "need" | "lock") && value.is_none() {
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

        mdast::Node::Blockquote(mdast::Blockquote { children, position }) => (
            mds_to_quote_block_or_admonition(children, context),
            position,
        ),

        mdast::Node::Code(code) => {
            let position = code.position.clone();
            let block = match context.format {
                Format::Myst => {
                    myst_to_block(&code, context).unwrap_or_else(|| code_to_block(code, context))
                }
                _ => code_to_block(code, context),
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
                    .and_then(|string| string.split_whitespace().next().map(String::from))
                    .or_else(|| Some("tex".into())),
                ..Default::default()
            }),
            position,
        ),

        mdast::Node::Paragraph(mdast::Paragraph { children, position }) => {
            let mut inlines = mds_to_inlines(children, context);

            if !context.preserve_newlines {
                // Replace newlines in paragraphs with a space
                for inline in inlines.iter_mut() {
                    if let Inline::Text(Text { value, .. }) = inline
                        && value.contains('\n')
                    {
                        value.string = value.replace('\n', " ");
                    }
                }
            }

            let block = if let (
                1,
                Some(Inline::CodeExpression(CodeExpression {
                    programming_language: Some(lang),
                    code,
                    ..
                })),
            ) = (inlines.len(), inlines.first())
            {
                if lang == "docsql" {
                    Block::CodeChunk(CodeChunk {
                        programming_language: Some(lang.to_string()),
                        code: code.clone(),
                        ..Default::default()
                    })
                } else {
                    Block::Paragraph(Paragraph::new(inlines))
                }
            } else if inlines.len() == 1 {
                match inlines.swap_remove(0) {
                    Inline::AudioObject(obj) => Block::AudioObject(obj),
                    Inline::ImageObject(obj) => Block::ImageObject(obj),
                    Inline::VideoObject(obj) => Block::VideoObject(obj),
                    inline => Block::Paragraph(Paragraph::new(vec![inline])),
                }
            } else {
                Block::Paragraph(Paragraph::new(inlines))
            };

            (block, position)
        }

        mdast::Node::Table(mdast::Table {
            children,
            align,
            position,
        }) => {
            // TODO: use table alignment
            (
                Block::Table(Table::new(mds_to_table_rows(children, align, context))),
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
fn myst_to_block(code: &mdast::Code, context: &mut Context) -> Option<Block> {
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

    // Create a new context, with the same format (MyST) so that the decode map
    // does not have position restarting of zero when `value` is re-parsed in `decode_blocks`
    let context = &mut Context::new(context.format.clone());

    if let Some(claim_type) = name.strip_prefix("prf:") {
        return Some(Block::Claim(Claim {
            claim_type: claim_type.parse().unwrap_or_default(),
            label: options.get("label").map(|label| label.to_string()),
            content: decode_blocks(&value, context),
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
                content: decode_blocks(&value, context),
                title: args.map(|arg| decode_inlines(arg, context)),
                is_folded: options.get("class").map(|&class| class == "dropdown"),
                ..Default::default()
            })
        }
        name if name == "code-cell" || is_executable_language(name) => {
            let programming_language = if is_executable_language(name) {
                Some(name.to_string())
            } else {
                args.map(String::from)
            };

            Block::CodeChunk(CodeChunk {
                code: value.into(),
                programming_language,
                is_echoed: options
                    .get("echo")
                    .and_then(|mode| mode.parse::<bool>().ok()),
                is_hidden: options
                    .get("hide")
                    .and_then(|mode| mode.parse::<bool>().ok()),
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
                    .map(|&caption| decode_blocks(caption, context)),
                ..Default::default()
            })
        }
        "figure" => {
            let content = code
                .meta
                .as_ref()
                .map(|url| {
                    vec![Block::ImageObject(ImageObject {
                        content_url: url.into(),
                        ..Default::default()
                    })]
                })
                .unwrap_or_default();
            let caption = decode_blocks(&value, context);

            let mut figure = Figure {
                label: options.get("label").map(|label| label.to_string()),
                label_automatically: options.contains_key("label").then_some(false),
                caption: (!caption.is_empty()).then_some(caption),
                content,
                options: Box::new(FigureOptions {
                    layout: options.get("layout").map(|label| label.to_string()),
                    padding: options.get("padding").map(|padding| padding.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            extract_figure_overlay(&mut figure);

            Block::Figure(figure)
        }
        "table" => {
            let rows = if let Some(Block::Table(Table { rows, .. })) =
                decode_blocks(&value, context).first()
            {
                rows.clone()
            } else {
                Vec::new()
            };

            Block::Table(Table {
                label: options.get("label").map(|label| label.to_string()),
                label_automatically: options.contains_key("label").then_some(false),
                caption: args.map(|arg| decode_blocks(arg, context)),
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
        "create" | "edit" | "fix" | "describe" => {
            let prompt = options
                .get("prompt")
                .map(|prompt| PromptBlock {
                    target: Some(prompt.to_string()),
                    ..Default::default()
                })
                .unwrap_or_default();

            // Use deserialization aliases inherent in schema to permissively
            // parse model_parameters
            let model_parameters = serde_json::from_value(json!(options)).unwrap_or_default();

            let execution_mode = options.get("mode").and_then(|value| value.parse().ok());

            Block::InstructionBlock(InstructionBlock {
                instruction_type: name.parse().unwrap_or_default(),
                prompt,
                message: args.map(InstructionMessage::from).unwrap_or_default(),
                model_parameters,
                execution_mode,
                content: if !value.trim().is_empty() {
                    Some(decode_blocks(&value, context))
                } else {
                    None
                },
                ..Default::default()
            })
        }
        "suggest" => Block::SuggestionBlock(SuggestionBlock {
            feedback: args.map(|value| value.to_string()),
            suggestion_status: options
                .get("status")
                .and_then(|value| SuggestionStatus::from_keyword(value).ok()),
            content: decode_blocks(&value, context),
            ..Default::default()
        }),
        "style" => Block::StyledBlock(StyledBlock {
            code: args.unwrap_or_default().into(),
            content: decode_blocks(&value, context),
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

/// Normalize extracted overlay content for stable round-tripping.
fn normalize_overlay(overlay: &str) -> String {
    overlay.strip_suffix('\n').unwrap_or(overlay).to_string()
}

/// Determine whether a code block is an explicit SVG overlay fence.
fn is_svg_overlay(programming_language: &Option<String>) -> bool {
    programming_language.as_deref() == Some("svg overlay")
}

/// Promote an explicit SVG overlay fence in a figure-labeled code chunk's
/// caption into the chunk's `overlay` property.
///
/// This is needed when collapsing a single-code-chunk `::: figure` into a
/// `CodeChunk`, because the overlay fence is initially parsed as a caption
/// child rather than as a chunk property. Code chunks only derive overlays
/// when labeled as figures; in all other cases an `svg overlay` fence remains
/// a normal code block.
fn extract_code_chunk_overlay(chunk: &mut CodeChunk) {
    if !matches!(chunk.label_type, Some(LabelType::FigureLabel)) {
        return;
    }

    if let Some(caption) = &mut chunk.caption {
        let mut overlay = None;
        let mut blocks = Vec::with_capacity(caption.len());

        for block in caption.drain(..) {
            match block {
                Block::CodeBlock(CodeBlock {
                    programming_language,
                    code,
                    ..
                }) if chunk.overlay.is_none()
                    && overlay.is_none()
                    && is_svg_overlay(&programming_language) =>
                {
                    overlay = Some(normalize_overlay(&code));
                }
                block => blocks.push(block),
            }
        }

        *caption = blocks;
        if chunk.overlay.is_none() {
            chunk.overlay = overlay;
        }
        if caption.is_empty() {
            chunk.caption = None;
        }
    }
}

/// Promote an explicit SVG overlay fence in a figure's content or caption into
/// the figure's `overlay` property.
///
/// Figures only derive overlays from explicit `svg overlay` fences. Extracted
/// fences are removed from content or caption, and the first such fence wins.
fn extract_figure_overlay(figure: &mut Figure) {
    let mut overlay = None;
    let mut content = Vec::with_capacity(figure.content.len());

    for block in figure.content.drain(..) {
        match block {
            Block::CodeBlock(CodeBlock {
                programming_language,
                code,
                ..
            }) if overlay.is_none() && is_svg_overlay(&programming_language) => {
                overlay = Some(normalize_overlay(&code));
            }
            block => content.push(block),
        }
    }

    figure.content = content;
    if figure.options.overlay.is_none() {
        figure.options.overlay = overlay;
    }

    if let Some(caption) = &mut figure.caption {
        let mut overlay = None;
        let mut blocks = Vec::with_capacity(caption.len());

        for block in caption.drain(..) {
            match block {
                Block::CodeBlock(CodeBlock {
                    programming_language,
                    code,
                    ..
                }) if figure.options.overlay.is_none()
                    && overlay.is_none()
                    && is_svg_overlay(&programming_language) =>
                {
                    overlay = Some(normalize_overlay(&code));
                }
                block => blocks.push(block),
            }
        }

        *caption = blocks;
        if figure.options.overlay.is_none() {
            figure.options.overlay = overlay;
        }
        if caption.is_empty() {
            figure.caption = None;
        }
    }
}

/// Transform a [`mdast::Code`] node to a Stencila [`Block`]
fn code_to_block(code: mdast::Code, context: &mut Context) -> Block {
    let mdast::Code {
        lang, meta, value, ..
    } = code;

    let meta = meta.unwrap_or_default();
    let code_id = parse_code_id(&meta);
    let is_svg_overlay = lang.as_deref() == Some("svg") && meta.trim() == "overlay";
    let is_exec = meta.starts_with("exec")
        || lang.as_deref() == Some("exec")
        || lang.as_deref().is_some_and(is_executable_language)
        || lang
            .as_ref()
            .map(|lang| lang.starts_with("{") && lang.ends_with("}"))
            .unwrap_or_default();
    let is_raw = meta.starts_with("raw") || lang.as_deref() == Some("raw");
    let is_demo = meta.starts_with("demo");

    if is_exec {
        let lang = lang.and_then(|lang| {
            let lang = lang
                .trim_start_matches("{")
                .trim_end_matches("}")
                .to_string();
            (!lang.is_empty()).then_some(lang)
        });

        let meta = meta.strip_prefix("exec").unwrap_or_default().trim();

        #[derive(Clone, PartialEq)]
        enum Keyword {
            IsEchoed,
            IsHidden,
            ExecutionMode(ExecutionMode),
            ExecutionBounds(ExecutionBounds),
            Ignore,
        }
        let keywords: Vec<Keyword> = separated(
            0..,
            alt((
                "echo".value(Keyword::IsEchoed),
                "hide".value(Keyword::IsHidden),
                execution_mode.map(Keyword::ExecutionMode),
                execution_bounds.map(Keyword::ExecutionBounds),
                take_while(1.., |c| !AsChar::is_space(c)).value(Keyword::Ignore),
            )),
            multispace1,
        )
        .parse_next(&mut Located::new(meta))
        .unwrap_or_default();

        let is_echoed = keywords.contains(&Keyword::IsEchoed).then_some(true);
        let is_hidden = keywords.contains(&Keyword::IsHidden).then_some(true);
        let execution_mode = keywords.iter().find_map(|kw| match kw {
            Keyword::ExecutionMode(mode) => Some(*mode),
            _ => None,
        });
        let execution_bounds = keywords.iter().find_map(|kw| match kw {
            Keyword::ExecutionBounds(bounds) => Some(*bounds),
            _ => None,
        });

        let mut label_automatically = None;
        let mut label_type = None;
        let mut label = None;
        let mut caption = None;
        if matches!(context.format, Format::Qmd) {
            for line in value.lines() {
                if let Some(rest) = line
                    .strip_prefix("#| ")
                    .or_else(|| line.strip_prefix("//| "))
                {
                    if let Some(value) = rest.strip_prefix("label:") {
                        label_automatically = Some(false);
                        label = Some(value.trim().to_string());
                    } else if let Some(value) = rest.strip_prefix("fig-cap:") {
                        label_type = Some(LabelType::FigureLabel);
                        caption = Some(decode_blocks(
                            value.trim().trim_start_matches('"').trim_end_matches('"'),
                            context,
                        ));
                    } else if let Some(value) = rest.strip_prefix("tbl-cap:") {
                        label_type = Some(LabelType::TableLabel);
                        caption = Some(decode_blocks(
                            value.trim().trim_start_matches('"').trim_end_matches('"'),
                            context,
                        ));
                    }
                } else {
                    break;
                }
            }
        }

        Block::CodeChunk(CodeChunk {
            id: code_id,
            code: value.into(),
            programming_language: if lang.as_deref() == Some("exec") {
                None
            } else {
                lang
            },
            execution_mode,
            execution_bounds,
            is_echoed,
            is_hidden,
            label_automatically,
            label_type,
            label,
            caption,
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
            id: code_id,
            code: value.into(),
            programming_language: if is_svg_overlay {
                Some("svg overlay".to_string())
            } else {
                lang
            },
            is_demo: is_demo.then_some(true),
            ..Default::default()
        })
    }
}

fn parse_code_id(meta: &str) -> Option<String> {
    meta.split_whitespace().find_map(|token| {
        token
            .strip_prefix('#')
            .filter(|id| !id.is_empty())
            .map(ToString::to_string)
            .or_else(|| {
                token
                    .strip_prefix("id=")
                    .filter(|id| !id.is_empty())
                    .map(|id| id.trim_matches(['"', '\'']).to_string())
            })
    })
}

fn mds_to_quote_block_or_admonition(mut mds: Vec<mdast::Node>, context: &mut Context) -> Block {
    let mut content = Vec::new();
    if let Some(mdast::Node::Paragraph(..)) = mds.first() {
        // If the first node is a paragraph (usually the case) then
        // convert it to a block with newlines preserved for the following
        // parsing of the first line
        context.preserve_newlines = true;
        if let Some((block, position)) = md_to_block(mds.remove(0), context) {
            context.map_position(&position, block.node_type(), block.node_id());
            content.push(block);
        }
        context.preserve_newlines = false;
    }
    content.append(&mut mds_to_blocks(mds, context));

    let mut first_para = content.first_mut().and_then(|node| {
        if let Block::Paragraph(para) = node {
            Some(para)
        } else {
            None
        }
    });

    let first_text = first_para
        .as_mut()
        .and_then(|para| para.content.first_mut())
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
    let parsed: ModalResult<(&str, (&str, Option<&str>, Option<&str>, Option<char>))> = (
        delimited("[!", take_until(1.., "]"), "]"),
        opt(preceded(space0, alt(("+", "-")))),
        opt(preceded(space0, take_while(1.., |c| c != '\n'))),
        opt('\n'),
    )
        .parse_peek(first_string.as_str());

    if let Ok((rest, (admonition_type, fold, title, ..))) = parsed
        && let Ok(admonition_type) = admonition_type.parse::<AdmonitionType>()
    {
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
            if let Some(first_para) = first_para {
                if first_para.content.len() > 1 {
                    first_para.content.remove(0);
                } else {
                    content.remove(0);
                }
            }
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

fn mds_to_table_rows(
    mdast_rows: Vec<mdast::Node>,
    column_alignments: Vec<AlignKind>,
    context: &mut Context,
) -> Vec<TableRow> {
    let mut first = true;
    mdast_rows
        .into_iter()
        .filter_map(|mdast_row| {
            if let mdast::Node::TableRow(mdast::TableRow { children, position }) = mdast_row {
                let (row_type, cell_type) = if first {
                    first = false;
                    (
                        Some(TableRowType::HeaderRow),
                        Some(TableCellType::HeaderCell),
                    )
                } else {
                    (None, None)
                };

                let node = TableRow {
                    cells: mds_to_table_cells(children, cell_type, &column_alignments, context),
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

fn mds_to_table_cells(
    mdast_cells: Vec<mdast::Node>,
    cell_type: Option<TableCellType>,
    column_alignments: &[AlignKind],
    context: &mut Context,
) -> Vec<TableCell> {
    mdast_cells
        .into_iter()
        .enumerate()
        .filter_map(|(index, mdast_cell)| {
            if let mdast::Node::TableCell(mdast::TableCell { children, position }) = mdast_cell {
                let content = if children.is_empty() {
                    Vec::new()
                } else {
                    vec![Block::Paragraph(Paragraph::new(mds_to_inlines(
                        children, context,
                    )))]
                };

                let horizontal_alignment =
                    column_alignments
                        .get(index)
                        .and_then(|align_kind| match align_kind {
                            AlignKind::Left => Some(HorizontalAlignment::AlignLeft),
                            AlignKind::Center => Some(HorizontalAlignment::AlignCenter),
                            AlignKind::Right => Some(HorizontalAlignment::AlignRight),
                            AlignKind::None => None,
                        });

                let node = TableCell {
                    content,
                    cell_type,
                    options: Box::new(TableCellOptions {
                        horizontal_alignment,
                        ..Default::default()
                    }),
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
#[allow(clippy::unwrap_used)]
mod tests {
    use pretty_assertions::assert_eq;
    use stencila_codec::{
        DecodeOptions, EncodeOptions,
        stencila_format::Format,
        stencila_schema::{
            Block, ClaimType, CodeChunk, ExecutionMode, Figure, LabelType, Node, Paragraph,
        },
    };

    use super::*;
    use crate::{decode, encode};

    fn decode_smd(md: &str) -> Vec<Block> {
        let (node, _) = decode(
            md,
            Some(DecodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
        )
        .expect("decode should succeed");

        let Node::Article(article) = node else {
            panic!("expected article")
        };

        article.content
    }

    #[test]
    fn test_call_arg() {
        call_arg(&mut Located::new("arg=1")).unwrap();
        call_arg(&mut Located::new("arg = 1")).unwrap();
        call_arg(&mut Located::new("arg=`1*1`")).unwrap();
    }

    #[test]
    fn test_myst_figure_layout() {
        let code = mdast::Code {
            value: [":layout: 2", "", "Caption"].join("\n"),
            lang: Some("{figure}".into()),
            meta: Some("plot.png".into()),
            position: None,
        };

        let mut context = Context::new(Format::Myst);
        let Some(Block::Figure(figure)) = myst_to_block(&code, &mut context) else {
            panic!("expected figure")
        };
        assert_eq!(figure.options.layout.as_deref(), Some("2"));
    }

    #[test]
    fn test_decode_figure_overlay_from_content() {
        let blocks = decode_smd(
            "::: figure\n\n![](plot.png)\n\n```svg overlay\n<svg>content</svg>\n```\n\nCaption\n\n:::\n",
        );

        let Some(Block::Figure(Figure {
            options,
            content,
            caption,
            ..
        })) = blocks.first()
        else {
            panic!("expected figure")
        };

        assert_eq!(options.overlay.as_deref(), Some("<svg>content</svg>"));
        assert!(
            content
                .iter()
                .all(|block| !matches!(block, Block::CodeBlock(CodeBlock { programming_language: Some(lang), .. }) if lang == "svg overlay"))
        );
        assert!(caption.is_some());
    }

    #[test]
    fn test_decode_figure_overlay_from_caption() {
        let blocks = decode_smd(
            "::: figure\n\n![](plot.png)\n\nCaption\n\n```svg overlay\n<svg>caption</svg>\n```\n\n:::\n",
        );

        let Some(Block::Figure(Figure {
            options, caption, ..
        })) = blocks.first()
        else {
            panic!("expected figure")
        };

        assert_eq!(options.overlay.as_deref(), Some("<svg>caption</svg>"));
        assert!(
            caption
                .as_ref()
                .into_iter()
                .flatten()
                .all(|block| !matches!(block, Block::CodeBlock(CodeBlock { programming_language: Some(lang), .. }) if lang == "svg overlay"))
        );
    }

    #[test]
    fn test_decode_figure_with_image_and_code_chunk_subfigures() {
        let blocks = decode_smd(
            r#"::: figure [2]

    ::: figure

    ![](example.com/cat.jpg)

    First subfigure.

    :::

    ::: figure

    ```plotly exec
    {
      "data": [
        {
          "type": "bar",
          "x": ["Capture", "Draft", "Review", "Publish"],
          "y": [6, 10, 14, 9]
        }
      ]
    }
    ```

    Second subfigure.

    :::

A two-panel figure combining a real image with an executable plot.

:::
"#,
        );

        let Some(Block::Figure(Figure {
            content,
            caption,
            options,
            ..
        })) = blocks.first()
        else {
            panic!("expected figure")
        };

        assert_eq!(options.layout.as_deref(), Some("2"));
        assert_eq!(content.len(), 2);
        assert!(matches!(content.first(), Some(Block::Figure(..))));
        assert!(matches!(content.get(1), Some(Block::CodeChunk(..))));
        assert!(matches!(
            caption.as_ref().and_then(|caption| caption.first()),
            Some(Block::Paragraph(..))
        ));
    }

    #[test]
    fn test_decode_figure_with_code_chunk_and_image_subfigures() {
        let blocks = decode_smd(
            r#"::: figure [2]

    ::: figure

    ```plotly exec
    {
      "data": [
        {
          "type": "bar",
          "x": ["Capture", "Draft", "Review", "Publish"],
          "y": [6, 10, 14, 9]
        }
      ]
    }
    ```

    First subfigure.

    :::

    ::: figure

    ![](example.com/cat.jpg)

    Second subfigure.

    :::

A two-panel figure combining an executable plot with a real image.

:::
"#,
        );

        let Some(Block::Figure(Figure {
            content,
            caption,
            options,
            ..
        })) = blocks.first()
        else {
            panic!("expected figure")
        };

        assert_eq!(options.layout.as_deref(), Some("2"));
        assert_eq!(content.len(), 2);
        assert!(matches!(content.first(), Some(Block::CodeChunk(..))));
        assert!(matches!(content.get(1), Some(Block::Figure(..))));
        assert!(matches!(
            caption.as_ref().and_then(|caption| caption.first()),
            Some(Block::Paragraph(..))
        ));
    }

    #[test]
    fn test_decode_figure_without_overlay() {
        let blocks = decode_smd("::: figure\n\n![](plot.png)\n\nCaption\n\n:::\n");

        let Some(Block::Figure(Figure { options, .. })) = blocks.first() else {
            panic!("expected figure")
        };

        assert_eq!(options.overlay, None);
    }

    #[test]
    fn test_decode_single_chunk_figure_overlay_to_code_chunk() {
        let blocks = decode_smd(
            "::: figure 1\n\n```r exec\nplot(y~x)\n```\n\n```svg overlay\n<svg>chunk</svg>\n```\n\nCaption\n\n:::\n",
        );

        let Some(Block::CodeChunk(CodeChunk {
            label_type,
            overlay,
            caption,
            ..
        })) = blocks.first()
        else {
            panic!("expected code chunk")
        };

        assert_eq!(label_type, &Some(LabelType::FigureLabel));
        assert_eq!(overlay.as_deref(), Some("<svg>chunk</svg>"));
        assert!(
            caption
                .as_ref()
                .into_iter()
                .flatten()
                .all(|block| !matches!(block, Block::CodeBlock(CodeBlock { programming_language: Some(lang), .. }) if lang == "svg overlay"))
        );
    }

    #[test]
    fn test_do_not_extract_overlay_for_table_labeled_chunk() {
        let mut chunk = CodeChunk {
            label_type: Some(LabelType::TableLabel),
            caption: Some(vec![
                Block::CodeBlock(CodeBlock {
                    programming_language: Some("svg overlay".to_string()),
                    code: "<svg>table</svg>".into(),
                    ..Default::default()
                }),
                Block::Paragraph(Paragraph {
                    content: vec![Inline::Text(Text::from("Caption"))],
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        };

        extract_code_chunk_overlay(&mut chunk);

        assert_eq!(chunk.overlay, None);
        assert!(matches!(
            chunk.caption.as_ref().and_then(|caption| caption.first()),
            Some(Block::CodeBlock(CodeBlock { programming_language: Some(lang), .. })) if lang == "svg overlay"
        ));
    }

    #[test]
    fn test_smd_overlay_roundtrip() {
        let original = "::: figure\n\n![](plot.png)\n\n```svg overlay\n<svg>roundtrip</svg>\n```\n\nCaption\n\n:::\n";

        let (node, _) = decode(
            original,
            Some(DecodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
        )
        .expect("decode should succeed");

        let (encoded, _) = encode(
            &node,
            Some(EncodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
        )
        .expect("encode should succeed");

        assert!(encoded.contains("```svg overlay\n<svg>roundtrip</svg>\n```"));

        let (decoded_again, _) = decode(
            &encoded,
            Some(DecodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
        )
        .expect("decode should succeed");

        let Node::Article(article) = decoded_again else {
            panic!("expected article")
        };
        let Some(Block::Figure(Figure { options, .. })) = article.content.first() else {
            panic!("expected figure")
        };

        assert_eq!(options.overlay.as_deref(), Some("<svg>roundtrip</svg>"));
    }

    #[test]
    fn test_figure_with_attrs() {
        let Block::Figure(Figure { options, .. }) =
            figure(&mut Located::new("figure [2]")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(options.layout.as_deref(), Some("2"));
        assert_eq!(options.padding, None);

        let Block::Figure(Figure { options, .. }) =
            figure(&mut Located::new("figure [30 70]")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(options.layout.as_deref(), Some("30 70"));
        assert_eq!(options.padding, None);

        let Block::Figure(Figure { label, options, .. }) =
            figure(&mut Located::new("figure {pad=50}")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(label, None);
        assert_eq!(options.layout, None);
        assert_eq!(options.padding.as_deref(), Some("50"));

        let Block::Figure(Figure { label, options, .. }) =
            figure(&mut Located::new("figure 1 {pad=50}")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(label.as_deref(), Some("1"));
        assert_eq!(options.layout, None);
        assert_eq!(options.padding.as_deref(), Some("50"));

        let Block::Figure(Figure { label, options, .. }) =
            figure(&mut Located::new("figure [2] {pad=50}")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(label, None);
        assert_eq!(options.layout.as_deref(), Some("2"));
        assert_eq!(options.padding.as_deref(), Some("50"));

        let Block::Figure(Figure { label, options, .. }) =
            figure(&mut Located::new("figure 1 [2] {pad=\"30 60\"}")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(label.as_deref(), Some("1"));
        assert_eq!(options.layout.as_deref(), Some("2"));
        assert_eq!(options.padding.as_deref(), Some("30 60"));

        let Block::Figure(Figure { label, options, .. }) =
            figure(&mut Located::new("figure  {pad=50}")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(label, None);
        assert_eq!(options.padding.as_deref(), Some("50"));
    }

    #[test]
    fn test_figure_with_id() {
        let Block::Figure(Figure { id, label, .. }) =
            figure(&mut Located::new("figure #data-plot")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(id.as_deref(), Some("data-plot"));
        assert_eq!(label, None);

        let Block::Figure(Figure {
            id, label, options, ..
        }) = figure(&mut Located::new("figure 1 #data-plot")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(id.as_deref(), Some("data-plot"));
        assert_eq!(label.as_deref(), Some("1"));
        assert_eq!(options.layout, None);

        let Block::Figure(Figure { id, label, .. }) =
            figure(&mut Located::new("figure #data-plot 1")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(id.as_deref(), Some("data-plot"));
        assert_eq!(label.as_deref(), Some("1"));

        let Block::Figure(Figure {
            id, label, options, ..
        }) = figure(&mut Located::new("figure 1 #data-plot [2] {pad=50}")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(id.as_deref(), Some("data-plot"));
        assert_eq!(label.as_deref(), Some("1"));
        assert_eq!(options.layout.as_deref(), Some("2"));
        assert_eq!(options.padding.as_deref(), Some("50"));

        let Block::Figure(Figure {
            id, label, options, ..
        }) = figure(&mut Located::new("figure #data-plot [2]")).unwrap()
        else {
            panic!("expected figure")
        };
        assert_eq!(id.as_deref(), Some("data-plot"));
        assert_eq!(label, None);
        assert_eq!(options.layout.as_deref(), Some("2"));
    }

    #[test]
    fn test_table_with_id() {
        let Block::Table(Table { id, label, .. }) =
            table(&mut Located::new("table #summary-table")).unwrap()
        else {
            panic!("expected table")
        };
        assert_eq!(id.as_deref(), Some("summary-table"));
        assert_eq!(label, None);

        let Block::Table(Table { id, label, .. }) =
            table(&mut Located::new("table 1 #summary-table")).unwrap()
        else {
            panic!("expected table")
        };
        assert_eq!(id.as_deref(), Some("summary-table"));
        assert_eq!(label.as_deref(), Some("1"));

        let Block::Table(Table { id, label, .. }) =
            table(&mut Located::new("table #summary-table 1")).unwrap()
        else {
            panic!("expected table")
        };
        assert_eq!(id.as_deref(), Some("summary-table"));
        assert_eq!(label.as_deref(), Some("1"));
    }

    #[test]
    fn test_incomplete_block() {
        // Incomplete (e.g. partially written in editor)
        assert_eq!(
            include_block(&mut Located::new("include")).unwrap(),
            Block::IncludeBlock(IncludeBlock {
                source: "".to_string(),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_call_block() {
        // Incomplete (e.g. partially written in editor)
        assert_eq!(
            call_block(&mut Located::new("call")).unwrap(),
            Block::CallBlock(CallBlock {
                source: "".to_string(),
                ..Default::default()
            })
        );
        assert_eq!(
            call_block(&mut Located::new("call file.md")).unwrap(),
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                ..Default::default()
            })
        );

        // No args
        assert_eq!(
            call_block(&mut Located::new("call file.md ()")).unwrap(),
            Block::CallBlock(CallBlock {
                source: "file.md".to_string(),
                ..Default::default()
            })
        );

        // With args
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
        // Incomplete (e.g. partially written in editor)
        assert_eq!(
            for_block(&mut Located::new("for")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "".to_string(),
                code: "".into(),
                ..Default::default()
            })
        );
        assert_eq!(
            for_block(&mut Located::new("for item")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "".into(),
                ..Default::default()
            })
        );
        assert_eq!(
            for_block(&mut Located::new("for item in")).unwrap(),
            Block::ForBlock(ForBlock {
                variable: "item".to_string(),
                code: "".into(),
                ..Default::default()
            })
        );

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
        // Incomplete (e.g. partially written in editor)
        assert_eq!(
            if_elif(&mut Located::new("::: if")).unwrap(),
            (
                true,
                IfBlockClause {
                    code: "".into(),
                    ..Default::default()
                }
            )
        );

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
            styled_block(&mut Located::new("style")).unwrap(),
            Block::StyledBlock(StyledBlock {
                ..Default::default()
            })
        );

        assert_eq!(
            styled_block(&mut Located::new("styled")).unwrap(),
            Block::StyledBlock(StyledBlock {
                ..Default::default()
            })
        );

        assert_eq!(
            styled_block(&mut Located::new("style color:red;")).unwrap(),
            Block::StyledBlock(StyledBlock {
                code: "color:red;".into(),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_styled_block_qmd() {
        // Single class
        assert_eq!(
            styled_block_qmd(&mut Located::new("{.class-a}")).unwrap(),
            Block::StyledBlock(StyledBlock {
                code: ".class-a".into(),
                ..Default::default()
            })
        );

        // Multiple classes
        assert_eq!(
            styled_block_qmd(&mut Located::new("{.class-a .class-b}")).unwrap(),
            Block::StyledBlock(StyledBlock {
                code: ".class-a .class-b".into(),
                ..Default::default()
            })
        );

        // With extra spacing (trimmed)
        assert_eq!(
            styled_block_qmd(&mut Located::new("{ .class-a  .class-b }")).unwrap(),
            Block::StyledBlock(StyledBlock {
                code: ".class-a  .class-b".into(),
                ..Default::default()
            })
        );

        // Multiple classes with underscores and numbers
        assert_eq!(
            styled_block_qmd(&mut Located::new("{.tw-flex .tw-items-center .col-2}")).unwrap(),
            Block::StyledBlock(StyledBlock {
                code: ".tw-flex .tw-items-center .col-2".into(),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_divider() {
        assert_eq!(divider(&mut "::: else", true).unwrap(), Divider::Else);
        assert_eq!(divider(&mut "::::: else  ", true).unwrap(), Divider::Else);

        assert_eq!(divider(&mut ":::", true).unwrap(), Divider::End);
        assert_eq!(divider(&mut "::::", true).unwrap(), Divider::End);
        assert_eq!(divider(&mut "::::::", true).unwrap(), Divider::End);

        // Critic Markup fences only match as dividers when there is an open block
        assert_eq!(divider(&mut ":++", true).unwrap(), Divider::End);
        assert_eq!(divider(&mut ":--", true).unwrap(), Divider::End);
        assert_eq!(divider(&mut ":~~", true).unwrap(), Divider::End);
        assert_eq!(divider(&mut ":~>", true).unwrap(), Divider::Next);
        assert!(divider(&mut ":++", false).is_err());
        assert!(divider(&mut ":--", false).is_err());
        assert!(divider(&mut ":~~", false).is_err());
        assert!(divider(&mut ":~>", false).is_err());

        assert!(divider(&mut "::: some chars", true).is_err());
        assert!(divider(&mut "::: with :::", true).is_err());
        assert!(divider(&mut "::", true).is_err());
        assert!(divider(&mut ":", true).is_err());
    }

    // Mermaid code block should be treated as executable CodeChunk by default
    #[test]
    fn test_mermaid_code_block() {
        let code = mdast::Code {
            lang: Some("mermaid".to_string()),
            meta: None,
            value: "graph TD\n    A --> B".to_string(),
            position: None,
        };
        let block = code_to_block(code, &mut Context::new(Format::Markdown));

        assert!(matches!(block, Block::CodeChunk(CodeChunk { .. })));

        if let Block::CodeChunk(chunk) = block {
            assert_eq!(chunk.programming_language, Some("mermaid".to_string()));
            assert_eq!(chunk.code.to_string(), "graph TD\n    A --> B");
        }
    }

    // Graphviz code block should be treated as executable CodeChunk by default
    #[test]
    fn test_graphviz_code_block() {
        let code = mdast::Code {
            lang: Some("graphviz".to_string()),
            meta: None,
            value: "digraph { A -> B }".to_string(),
            position: None,
        };
        let block = code_to_block(code, &mut Context::new(Format::Markdown));

        assert!(matches!(block, Block::CodeChunk(CodeChunk { .. })));

        if let Block::CodeChunk(chunk) = block {
            assert_eq!(chunk.programming_language, Some("graphviz".to_string()));
            assert_eq!(chunk.code.to_string(), "digraph { A -> B }");
        }
    }

    // Dot code block should be treated as executable CodeChunk by default
    #[test]
    fn test_dot_code_block() {
        let code = mdast::Code {
            lang: Some("dot".to_string()),
            meta: None,
            value: "digraph { A -> B }".to_string(),
            position: None,
        };
        let block = code_to_block(code, &mut Context::new(Format::Markdown));

        assert!(matches!(block, Block::CodeChunk(CodeChunk { .. })));

        if let Block::CodeChunk(chunk) = block {
            assert_eq!(chunk.programming_language, Some("dot".to_string()));
            assert_eq!(chunk.code.to_string(), "digraph { A -> B }");
        }
    }

    #[test]
    fn test_code_block_id_from_meta() {
        let code = mdast::Code {
            lang: Some("text".to_string()),
            meta: Some("#creator-prompt".to_string()),
            value: "Create something".to_string(),
            position: None,
        };
        let block = code_to_block(code, &mut Context::new(Format::Markdown));

        assert!(matches!(block, Block::CodeBlock(CodeBlock { .. })));

        if let Block::CodeBlock(block) = block {
            assert_eq!(block.id.as_deref(), Some("creator-prompt"));
        }
    }

    #[test]
    fn test_code_chunk_id_from_meta() {
        let code = mdast::Code {
            lang: Some("exec".to_string()),
            meta: Some("exec id=run-script".to_string()),
            value: "print('hello')".to_string(),
            position: None,
        };
        let block = code_to_block(code, &mut Context::new(Format::Markdown));

        assert!(matches!(block, Block::CodeChunk(CodeChunk { .. })));

        if let Block::CodeChunk(chunk) = block {
            assert_eq!(chunk.id.as_deref(), Some("run-script"));
        }
    }
}
