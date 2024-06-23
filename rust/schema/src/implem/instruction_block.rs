use codec_info::{lost_exec_options, lost_options};

use crate::{
    authorship, prelude::*, AuthorRole, AuthorRoleName, InstructionBlock, SuggestionStatus,
};

impl InstructionBlock {
    pub fn apply_patch_op(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        _context: &mut PatchContext,
    ) -> Result<bool> {
        if path.is_empty() {
            if let PatchOp::Choose(suggestion_id) = op {
                for suggestion in self.suggestions.iter_mut().flatten() {
                    if &suggestion.node_id() == suggestion_id {
                        suggestion.suggestion_status = Some(SuggestionStatus::Accepted);

                        let mut content = suggestion.content.clone();
                        authorship(
                            &mut content,
                            vec![AuthorRole::anon(AuthorRoleName::Generator)],
                        )?;
                        self.content = Some(content);
                    } else if matches!(
                        suggestion.suggestion_status,
                        Some(SuggestionStatus::Accepted)
                    ) {
                        suggestion.suggestion_status = None;
                    }
                }

                if self.hide_suggestions.is_none() {
                    self.hide_suggestions = Some(true);
                }

                return Ok(true);
            }
        } else if matches!(
            path.front(),
            Some(PatchSlot::Property(NodeProperty::Suggestions))
        ) {
            // Ignore any patch on suggestions if suggestions are hidden.
            // Prevents suggestions being cleared when patching from Markdown when suggestions are hidden.
            if self.hide_suggestions == Some(true) {
                return Ok(true);
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
            .merge_losses(lost_options!(self, id, auto_exec))
            .merge_losses(lost_exec_options!(self))
            .push_semis()
            .push_str(" ")
            .push_str(self.instruction_type.to_string().to_lowercase().as_str())
            .push_str(" ");

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

        if let Some(part) = self
            .messages
            .last()
            .and_then(|message| message.parts.first())
        {
            context
                .push_prop_fn(NodeProperty::Messages, |context| part.to_markdown(context))
                .newline();
        }

        if let Some(content) = &self.content {
            context
                .newline()
                .push_prop_fn(NodeProperty::Content, |context| {
                    content.to_markdown(context)
                });
        };

        context.push_semis().newline().newline();

        if !self.hide_suggestions.unwrap_or_default() {
            if let Some(suggestions) = &self.suggestions {
                context.push_prop_fn(NodeProperty::Suggestions, |context| {
                    suggestions.to_markdown(context)
                });
            }
        }

        context.exit_node();
    }
}
