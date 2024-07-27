use codec_info::{lost_exec_options, lost_options};
use common::tracing;

use crate::{patch, prelude::*, InstructionInline, SuggestionStatus};

impl InstructionInline {
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
                                    tracing::error!("While accepting inline suggestion: {error}");
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
        }

        Ok(false)
    }
}

impl MarkdownCodec for InstructionInline {
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
            .merge_losses(lost_exec_options!(self))
            .push_str("[[")
            .push_str(self.instruction_type.to_string().to_lowercase().as_str())
            .push_str(" ");

        if let Some(assignee) = &self.assignee {
            context.push_str("@").push_str(assignee).push_str(" ");
        }

        if let Some(message) = &self.message {
            context.push_prop_fn(NodeProperty::Message, |context| {
                message.to_markdown(context)
            });
        }

        if let Some(content) = &self.content {
            context
                .push_str(">>")
                .push_prop_fn(NodeProperty::Content, |context| {
                    content.to_markdown(context)
                });
        }

        context.push_str("]]");

        if let Some(suggestions) = &self.suggestions {
            context.push_prop_fn(NodeProperty::Suggestions, |context| {
                suggestions.to_markdown(context)
            });
        }

        context.exit_node();
    }
}
