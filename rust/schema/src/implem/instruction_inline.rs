use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, InstructionInline, SuggestionStatus};

impl InstructionInline {
    pub fn apply_patch_op(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        _context: &mut PatchContext,
    ) -> Result<bool> {
        if path.is_empty() {
            if let PatchOp::Accept(suggestion_id) = op {
                for suggestion in self.suggestions.iter_mut().flatten() {
                    if &suggestion.node_id() == suggestion_id {
                        suggestion.suggestion_status = Some(SuggestionStatus::Accepted);
                        // TODO: add a the current author (from the context) with the accepter role
                        self.content = Some(suggestion.content.clone());
                    } else if matches!(
                        suggestion.suggestion_status,
                        Some(SuggestionStatus::Accepted)
                    ) {
                        suggestion.suggestion_status = None;
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

        if let Some(part) = self
            .messages
            .first()
            .and_then(|message| message.parts.first())
        {
            context.push_prop_fn(NodeProperty::Message, |context| part.to_markdown(context));
        }

        if let Some(content) = &self.content {
            context
                .push_str(">>")
                .push_prop_fn(NodeProperty::Content, |context| {
                    content.to_markdown(context)
                });
        };

        context.push_str("]]");

        if let Some(suggestions) = &self.suggestions {
            context.push_prop_fn(NodeProperty::Suggestions, |context| {
                suggestions.to_markdown(context)
            });
        }

        context.exit_node();
    }
}
