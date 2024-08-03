use codec_info::{lost_exec_options, lost_options};
use common::tracing;

use crate::{patch, prelude::*, InstructionBlock, SuggestionStatus};

impl InstructionBlock {
    pub fn apply_patch_op(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        if path.is_empty() {
            if let PatchOp::Accept(suggestion_id) = op {
                if let Some(suggestions) = &mut self.suggestions {
                    for suggestion in suggestions.iter_mut() {
                        if &suggestion.node_id() == suggestion_id {
                            // Mark the accepted suggestion as such
                            suggestion.suggestion_status = SuggestionStatus::Accepted;

                            // Record the patcher as the acceptor
                            let accepter_patch = context.authors_as_acceptors();
                            let mut content = suggestion.content.clone();
                            for node in &mut content {
                                if let Err(error) = patch(node, accepter_patch.clone()) {
                                    tracing::error!("While accepting block suggestion: {error}");
                                }
                            }

                            // Make the content of the suggestion the content of the instruction
                            self.content = Some(content);
                        } else if matches!(suggestion.suggestion_status, SuggestionStatus::Proposed)
                        {
                            // Mark suggestions that are proposed as unaccepted
                            // (i.e. not accepted, but also not explicitly rejected)
                            suggestion.suggestion_status = SuggestionStatus::Unaccepted;
                        }
                    }
                }

                return Ok(true);
            }
        } else if matches!(
            path.front(),
            Some(PatchSlot::Property(NodeProperty::Suggestions))
        ) && matches!(context.format, Some(Format::Markdown | Format::Myst))
        {
            // Manually apply remove and clear patches on suggestions.
            // Prevent non-proposed suggestions, which are not encoded to Markdown,
            // from being deleted.
            if let Some(suggestions) = &mut self.suggestions {
                if let PatchOp::Remove(indices) = op {
                    let mut index = 0usize;
                    suggestions.retain(|suggestion| {
                        let retain =
                            !matches!(suggestion.suggestion_status, SuggestionStatus::Proposed)
                                || !indices.contains(&index);

                        index += 1;

                        retain
                    });
                    return Ok(true);
                } else if matches!(op, PatchOp::Clear | PatchOp::Set(PatchValue::None)) {
                    suggestions.retain(|suggestion| {
                        !matches!(suggestion.suggestion_status, SuggestionStatus::Proposed)
                    });
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

impl MarkdownCodec for InstructionBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
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
                        if let Some(assignee) = &self.assignee {
                            context.myst_directive_option(
                                NodeProperty::Assignee,
                                Some("assign"),
                                assignee,
                            );
                        }
                        if let Some(reps) = &self.replicates {
                            context.myst_directive_option(
                                NodeProperty::Assignee,
                                Some("reps"),
                                &reps.to_string(),
                            );
                        }
                        if let Some(thresh) =
                            &self.model.as_ref().and_then(|model| model.minimum_score)
                        {
                            context.myst_directive_option(
                                NodeProperty::Assignee,
                                Some("thresh"),
                                &thresh.to_string(),
                            );
                        }
                        if let Some(temp) = &self.model.as_ref().and_then(|model| model.temperature)
                        {
                            context.myst_directive_option(
                                NodeProperty::Assignee,
                                Some("temp"),
                                &temp.to_string(),
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

            if let Some(assignee) = &self.assignee {
                context.push_str("@").push_str(assignee).push_str(" ");
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
                if content.is_empty() {
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
