use common::serde_yaml;
use codec_dom_trait::DomCodec;

use crate::{prelude::*, Chat, ExecutionBounds, ExecutionMode, SuggestionBlock};

impl Chat {
    /// Custom implementation of [`PatchNode::apply`].
    ///
    /// Only allow operations on the `content` vector if the chat is not nested.
    pub fn apply_with(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        if path.is_empty() && matches!(op, PatchOp::Archive) {
            // Add this instruction to the root's archive
            context.op_additional(
                PatchPath::from(NodeProperty::Archive),
                PatchOp::Push(self.to_value()?),
            );

            // Remove this from the containing vector, if any
            let mut path = context.path();
            if let Some(PatchSlot::Index(index)) = path.pop_back() {
                context.op_additional(path, PatchOp::Remove(vec![index]));
            }

            return Ok(true);
        }

        if let Some(PatchSlot::Property(NodeProperty::Content)) = path.front() {
            // Only apply patches to the content of the chat if the patch is
            // associated with no, or a lossless, format, or if it is a root
            // node (not nested)
            let lossless_format = context
                .format
                .as_ref()
                .map(|format| format.is_lossless())
                .unwrap_or(true);
            let is_root = self.is_ephemeral.is_none();

            if lossless_format || is_root {
                path.pop_front();
                context.within_property(NodeProperty::Content, |context| {
                    self.content.apply(path, op.clone(), context)
                })?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Custom implementation of `to_dom` for the `suggestions` property to use
    /// custom carousel components which can not have slots between parent and items.
    pub fn suggestions_to_dom_elem(
        name: &str,
        suggestions: &Vec<SuggestionBlock>,
        context: &mut DomEncodeContext,
    ) {
        context.enter_elem_attrs("stencila-chat-suggestions", [("slot", name)]);
        for suggestion in suggestions {
            context.enter_elem("stencila-chat-suggestions-item");
            suggestion.to_dom(context);
            context.exit_elem();
        }
        context.exit_elem();
    }
}

impl MarkdownCodec for Chat {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // If not the root node (i.e. within an `Article` or other document) then
        // just represent as a single line so that the user knows that the chat is there
        // and can interact with it (e.g. via code lenses or key bindings)
        if !context.is_root() {
            context
                .enter_node(self.node_type(), self.node_id())
                .push_colons()
                .push_str(" chat ");

            if let Some(mode) = &self.execution_mode {
                if !matches!(mode, ExecutionMode::Default) {
                    context
                        .push_prop_str(
                            NodeProperty::ExecutionMode,
                            &mode.to_string().to_lowercase(),
                        )
                        .space();
                }
            }

            if let Some(bounds) = &self.execution_bounds {
                if !matches!(bounds, ExecutionBounds::Default) {
                    context
                        .push_prop_str(
                            NodeProperty::ExecutionBounds,
                            &bounds.to_string().to_lowercase(),
                        )
                        .space();
                }
            }

            if let Some(target) = &self.prompt.target {
                context
                    .push_str("@")
                    .push_prop_str(NodeProperty::Prompt, target)
                    .space();
            }

            context.push_prop_fn(NodeProperty::ModelParameters, |context| {
                self.model_parameters.to_markdown(context);
            });

            context.trim_end().push_str("\n\n").exit_node();

            return;
        }

        // The following is based on `Article::to_markdown` but with some differences
        // (e.g. not yet supporting authors)

        context.enter_node(self.node_type(), self.node_id());

        // Create a header version of self that has no content and can be stripped
        let mut header = Self {
            // Avoid serializing content unnecessarily
            content: Vec::new(),
            ..self.clone()
        };

        // Strip properties from header that are designated as not supported by Markdown.
        // This would be better to do based on the "patch formats" declaration in the
        // schema but that is not accessible from here. So we have to do it "manually"
        header.strip(&StripTargets {
            scopes: vec![
                StripScope::Provenance,
                StripScope::Execution,
                StripScope::Code,
                StripScope::Output,
                StripScope::Archive,
            ],
            ..Default::default()
        });
        header.options.authors = None;

        let mut yaml = serde_yaml::to_value(header).unwrap_or_default();
        if let Some(yaml) = yaml.as_mapping_mut() {
            // Remove the (now empty) content array
            yaml.remove("content");

            // Encode YAML header
            let yaml = serde_yaml::to_string(&yaml).unwrap_or_default();
            context.push_str("---\n");
            context.push_str(&yaml);
            context.push_str("---\n\n");
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        context.append_footnotes();

        context.exit_node_final();
    }
}
