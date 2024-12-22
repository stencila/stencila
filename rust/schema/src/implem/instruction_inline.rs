use codec_info::{lost_exec_options, lost_options};
use common::tracing;

use crate::{patch, prelude::*, InstructionInline, SuggestionStatus};

impl InstructionInline {
    /// Custom implementation of [`PatchNode::apply`]
    pub fn apply_with(
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

                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl MarkdownCodec for InstructionInline {
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
            .merge_losses(lost_exec_options!(self))
            .push_str("[[")
            .push_prop_str(
                NodeProperty::InstructionType,
                self.instruction_type.to_string().to_lowercase().as_str(),
            )
            .push_str(" ");

        context
            .push_str("@")
            .push_prop_str(NodeProperty::Prompt, &self.prompt.prompt)
            .push_str(" ");

        context.push_prop_fn(NodeProperty::Message, |context| {
            self.message.to_markdown(context)
        });

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
