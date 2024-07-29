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
                // Accept the suggestion and remove any other suggestions that have not been explicitly
                // accepted or rejected, or which have no feedback
                if let Some(suggestions) = &mut self.suggestions {
                    suggestions.retain_mut(|suggestion| {
                        if &suggestion.node_id() == suggestion_id {
                            suggestion.suggestion_status = Some(SuggestionStatus::Accepted);

                            let accepter_patch = context.authors_as_accepters();
                            let mut content = suggestion.content.clone();
                            for node in &mut content {
                                if let Err(error) = patch(node, accepter_patch.clone()) {
                                    tracing::error!("While accepting block suggestion: {error}");
                                }
                            }

                            self.content = Some(content);
                        }

                        if matches!(
                            suggestion.suggestion_status,
                            None | Some(SuggestionStatus::Proposed)
                        ) || suggestion.feedback.is_none()
                        {
                            false
                        } else {
                            true
                        }
                    })
                }

                return Ok(true);
            }
        } else if matches!(
            path.front(),
            Some(PatchSlot::Property(NodeProperty::Suggestions))
        ) && matches!(context.format, Some(Format::Markdown | Format::Myst))
        {
            // Manually apply remove and clear patches on suggestions.
            // Prevent accepted and rejected suggestions, which are not encoded to Markdown,
            // from being deleted (because they may be archived).
            if let Some(suggestions) = &mut self.suggestions {
                if let PatchOp::Remove(indices) = op {
                    let mut index = 0usize;
                    suggestions.retain(|suggestion| {
                        let retain = matches!(
                            suggestion.suggestion_status,
                            Some(SuggestionStatus::Accepted | SuggestionStatus::Rejected)
                        ) || !indices.contains(&index);

                        index += 1;

                        retain
                    });
                    return Ok(true);
                }

                if matches!(op, PatchOp::Clear) {
                    suggestions.retain(|suggestion| {
                        matches!(
                            suggestion.suggestion_status,
                            Some(SuggestionStatus::Accepted | SuggestionStatus::Rejected)
                        )
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
            if self.content.is_some() {
                context.push_semis().push_str(" ");
            } else {
                context.push_str("/ ");
            }

            context.push_str(&instruction_type).push_str(" ");

            if let Some(assignee) = &self.assignee {
                context.push_str("@").push_str(assignee).push_str(" ");
            }

            if let Some(model) = self.model.as_ref().and_then(|model| model.name.as_ref()) {
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
                context
                    .push_prop_fn(NodeProperty::Message, |context| {
                        message.to_markdown(context)
                    })
                    .newline();
            }

            if let Some(content) = &self.content {
                context
                    .newline()
                    .push_prop_fn(NodeProperty::Content, |context| {
                        content.to_markdown(context)
                    });
            };

            if self.content.is_some() {
                context.push_semis().newline();
            }

            context.newline();
        }

        if let Some(suggestions) = &self.suggestions {
            context.push_prop_fn(NodeProperty::Suggestions, |context| {
                suggestions.to_markdown(context)
            });
        }

        context.exit_node();
    }
}
