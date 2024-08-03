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
