use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, PromptBlock};

impl PromptBlock {
    /// Custom implementation of [`PatchNode::apply`]
    pub fn apply_patch_op(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        if context.format_is_lossy() {
            if let (
                Some(PatchSlot::Property(NodeProperty::Target)),
                PatchOp::Set(PatchValue::None),
            ) = (path.front(), op)
            {
                // Ignore attempt to clear inferred target
                if self
                    .target
                    .as_ref()
                    .map(|target| target.ends_with("?"))
                    .unwrap_or_default()
                {
                    return Ok(true);
                }
            } else if let (
                Some(PatchSlot::Property(NodeProperty::Hint)),
                PatchOp::Set(PatchValue::None),
            ) = (path.front(), op)
            {
                // Ignore attempt to clear implied hint
                if self
                    .hint
                    .as_ref()
                    .map(|hint| hint.ends_with("   "))
                    .unwrap_or_default()
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

impl MarkdownCodec for PromptBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(context.format, Format::Llmd) {
            // Do not encode at all
            return;
        }

        if context.render {
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
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self));

        if matches!(context.format, Format::Myst) {
            context.myst_directive(
                '`',
                "prompt",
                |context| {
                    if let Some(target) = &self.target {
                        context.space().push_prop_str(NodeProperty::Target, target);
                    }
                },
                |_| {},
                |_| {},
            );
        } else {
            context.push_colons().push_str(" prompt");

            if let Some(instruction_type) = &self.instruction_type {
                context.space().push_prop_str(
                    NodeProperty::InstructionType,
                    &instruction_type.to_string().to_lowercase(),
                );
            }

            if let Some(target) = &self.target {
                if !target.ends_with("?") {
                    context
                        .push_str(" @")
                        .push_prop_str(NodeProperty::Target, target);
                }
            }

            if let Some(hint) = &self.hint {
                context.space().push_prop_str(NodeProperty::Hint, hint);
            }
        }

        context.newline().exit_node().newline();
    }
}
