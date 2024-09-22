use codec_info::{lost_exec_options, lost_options};
use common::tracing;

use crate::{merge, patch, prelude::*, InstructionBlock, InstructionType, SuggestionStatus};

impl InstructionBlock {
    pub fn apply_patch_op(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        if !path.is_empty() {
            // Early return for ops that have a path (i.e. that target a descendant in the tree)
            return Ok(false);
        }

        if let PatchOp::Accept(suggestion_id) = op {
            if let Some(suggestions) = &mut self.suggestions {
                for suggestion in suggestions.iter_mut() {
                    if &suggestion.node_id() == suggestion_id {
                        // Mark the accepted suggestion as such
                        suggestion.suggestion_status = Some(SuggestionStatus::Accepted);

                        // Record the patcher as the acceptor
                        let accepter_patch = context.authors_as_acceptors();
                        let mut content = suggestion.content.clone();
                        for node in &mut content {
                            if let Err(error) = patch(node, accepter_patch.clone()) {
                                tracing::error!("While accepting block suggestion: {error}");
                            }
                        }
                    } else {
                        // Implicitly reject other suggestions
                        suggestion.suggestion_status = Some(SuggestionStatus::Rejected);
                    }
                }
            }

            Ok(true)
        } else if matches!(op, PatchOp::Archive) {
            // Add this instruction to the root's archive
            context.op_additional(
                PatchPath::from(NodeProperty::Archive),
                PatchOp::Push(self.to_value()?),
            );

            // Get the path and index for applying the additional op
            let mut path = context.path();
            let index = match path.pop_back() {
                Some(PatchSlot::Index(index)) => index,
                slot => bail!("Expected index slot, got: {slot:?}"),
            };

            // Get the accepted suggestion
            let accepted = self.suggestions.iter().flatten().find_map(|suggestion| {
                matches!(
                    suggestion.suggestion_status,
                    Some(SuggestionStatus::Accepted)
                )
                .then_some(suggestion.content.clone())
            });

            let Some(accepted) = accepted else {
                // No accepted suggestion, so just delete instruction
                context.op_additional(path, PatchOp::Remove(vec![index]));
                return Ok(true);
            };

            if accepted.is_empty() {
                // No content in accepted suggestion, so just delete instruction
                context.op_additional(path, PatchOp::Remove(vec![index]));
                return Ok(true);
            }

            match &self.instruction_type {
                InstructionType::New => {
                    if accepted.len() == 1 {
                        // Just one block in accepted suggestion, so replace with it
                        context.op_additional(
                            path,
                            PatchOp::Replace(vec![(index, accepted[0].to_value()?)]),
                        );
                    } else {
                        // More than one block in accepted suggestion so remove instruction and
                        // insert blocks in its place
                        let mut blocks = Vec::with_capacity(accepted.len());
                        for (offset, block) in accepted.iter().enumerate() {
                            blocks.push((index + offset, block.to_value()?))
                        }
                        context
                            .op_additional(path.clone(), PatchOp::Remove(vec![index]))
                            .op_additional(path, PatchOp::Insert(blocks));
                    }
                }
                InstructionType::Edit => {
                    // Merge the accepted content into the existing content and replace
                    // the instruction with that merged content
                    let merged = if let Some(content) = self.content.as_ref() {
                        let len = content.len().max(accepted.len());
                        let mut merged = Vec::with_capacity(len);
                        for index in 0..len {
                            let old = content.get(index);
                            let new = accepted.get(index);
                            match (old, new) {
                                (Some(old), Some(new)) => {
                                    let mut old = old.clone();
                                    merge(&mut old, new, None, None)?;
                                    merged.push(old);
                                }
                                (Some(old), None) => {
                                    merged.push(old.clone());
                                }
                                (None, Some(new)) => {
                                    merged.push(new.clone());
                                }
                                (None, None) => {}
                            }
                        }
                        merged
                    } else {
                        accepted
                    };

                    let mut blocks = Vec::with_capacity(merged.len());
                    for (offset, block) in merged.iter().enumerate() {
                        blocks.push((index + offset, block.to_value()?))
                    }

                    context
                        .op_additional(path.clone(), PatchOp::Remove(vec![index]))
                        .op_additional(path, PatchOp::Insert(blocks));
                }
                _ => {}
            };

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MarkdownCodec for InstructionBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render || matches!(context.format, Format::Llmd) {
            // Record any execution messages
            if let Some(messages) = &self.options.execution_messages {
                for message in messages {
                    context.add_message(
                        self.node_type(),
                        self.node_id(),
                        message.level.clone().into(),
                        message.message.to_string(),
                    );
                }
            }

            // Encode content only
            if let Some(content) = &self.content {
                content.to_markdown(context);
            }

            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, execution_mode))
            .merge_losses(lost_exec_options!(self));

        let instruction_type = self.instruction_type.to_string().to_lowercase();

        if matches!(context.format, Format::Myst) {
            context
                .myst_directive(
                    ':',
                    &instruction_type,
                    |context| {
                        if let Some(message) = &self.message {
                            context
                                .push_str(" ")
                                .push_prop_fn(NodeProperty::Message, |context| {
                                    message.to_markdown(context)
                                });
                        }
                    },
                    |context| {
                        if let Some(prompt) = &self.prompt {
                            context.myst_directive_option(
                                NodeProperty::Prompt,
                                Some("prompt"),
                                prompt,
                            );
                        }
                        if let Some(reps) = &self.replicates {
                            context.myst_directive_option(
                                NodeProperty::Replicates,
                                Some("reps"),
                                &reps.to_string(),
                            );
                        }
                    },
                    |context| {
                        if let Some(content) = &self.content {
                            context.push_prop_fn(NodeProperty::Content, |context| {
                                content.to_markdown(context)
                            });
                        }
                    },
                )
                .newline();
        } else {
            context
                .push_colons()
                .push_str(" ")
                .push_str(&instruction_type)
                .push_str(" ");

            if let Some(prompt) = &self.prompt {
                context.push_str("@").push_str(prompt).push_str(" ");
            }

            if let Some(model) = self
                .model
                .as_ref()
                .and_then(|model| model.id_pattern.as_ref())
            {
                context.push_str("[").push_str(model).push_str("] ");
            }

            if let Some(replicates) = &self.replicates {
                context
                    .push_str("x")
                    .push_str(&replicates.to_string())
                    .push_str(" ");
            }

            if let Some(value) = self
                .model
                .as_ref()
                .and_then(|model| model.minimum_score.as_ref())
            {
                context
                    .push_str("y")
                    .push_str(&value.to_string())
                    .push_str(" ");
            }

            if let Some(value) = self
                .model
                .as_ref()
                .and_then(|model| model.temperature.as_ref())
            {
                context
                    .push_str("t")
                    .push_str(&value.to_string())
                    .push_str(" ");
            }

            if let Some(value) = self
                .model
                .as_ref()
                .and_then(|model| model.quality_weight.as_ref())
            {
                context
                    .push_str("q")
                    .push_str(&value.to_string())
                    .push_str(" ");
            }

            if let Some(value) = self
                .model
                .as_ref()
                .and_then(|model| model.speed_weight.as_ref())
            {
                context
                    .push_str("s")
                    .push_str(&value.to_string())
                    .push_str(" ");
            }

            if let Some(value) = self
                .model
                .as_ref()
                .and_then(|model| model.cost_weight.as_ref())
            {
                context
                    .push_str("c")
                    .push_str(&value.to_string())
                    .push_str(" ");
            }

            if let Some(message) = &self.message {
                context.push_prop_fn(NodeProperty::Message, |context| {
                    message.to_markdown(context)
                });
            }

            if let Some(content) = &self.content {
                if content.is_empty() && !matches!(self.instruction_type, InstructionType::New) {
                    context.push_str(" <").newline().newline();
                } else {
                    if content.len() == 1 {
                        context.push_str(" >");
                    }

                    context
                        .newline()
                        .newline()
                        .push_prop_fn(NodeProperty::Content, |context| {
                            content.to_markdown(context)
                        });

                    if content.len() > 1 {
                        context.push_colons().newline().newline();
                    }
                }
            } else {
                context.push_str(" <").newline().newline();
            }
        }

        if let Some(suggestions) = &self.suggestions {
            context.push_prop_fn(NodeProperty::Suggestions, |context| {
                suggestions.to_markdown(context)
            });
        }

        context.exit_node();
    }
}
