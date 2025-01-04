use codec_dom_trait::DomCodec;
use common::serde_yaml;

use crate::{prelude::*, Chat, Node, SuggestionBlock};

impl Chat {
    /// Custom implementation of [`PatchNode::apply`]
    pub fn apply_with(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        if path.is_empty() && matches!(op, PatchOp::Archive | PatchOp::Temporize) {
            // Add this instruction to the root's archive or temporary set
            let property = match op {
                PatchOp::Archive => NodeProperty::Archive,
                _ => NodeProperty::Temporary,
            };
            context.op_additional(
                PatchPath::from(property),
                PatchOp::Push(PatchValue::Node(Node::Chat(self.clone()))),
            );

            // Remove this from the containing vector, if any
            let mut path = context.path();
            if let Some(PatchSlot::Index(index)) = path.pop_back() {
                context.op_additional(path, PatchOp::Remove(vec![index]));
            }

            return Ok(true);
        }

        if matches!(
            path.front(),
            Some(PatchSlot::Property(NodeProperty::Content))
        ) {
            // To allow for placeholder `::: chat` blocks with no content in Markdown formats,
            // only apply patches to the content of the chat if the patch is
            // associated with no, or a lossless, format, or if it is a root
            // node (not nested)
            if context.format_is_lossless() || self.is_temporary.is_none() {
                // Apply the patch
                path.pop_front();
                context.within_property(NodeProperty::Content, |context| {
                    self.content.apply(path, op.clone(), context)
                })?;
            }

            // Return true, even if not applied, so as to ignore op
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
                .push_str("/")
                .push_prop_str(
                    NodeProperty::InstructionType,
                    &self
                        .prompt
                        .instruction_type
                        .clone()
                        .unwrap_or_default()
                        .to_string()
                        .to_lowercase(),
                );

            if let Some(value) = &self.prompt.relative_position {
                context.space().push_prop_str(
                    NodeProperty::RelativePosition,
                    &value.to_string().to_lowercase(),
                );
            }

            if let Some(value) = self.prompt.node_types.iter().flatten().next() {
                context
                    .space()
                    .push_prop_str(NodeProperty::NodeTypes, &value.to_string().to_lowercase());
            }

            if let Some(target) = &self.prompt.target {
                // Do not encode inferred target
                if !target.ends_with("?") {
                    context
                        .push_str(" @")
                        .push_prop_str(NodeProperty::Prompt, target);
                }
            }

            context.push_prop_fn(NodeProperty::ModelParameters, |context| {
                self.model_parameters.to_markdown(context);
            });

            if let Some(query) = &self.prompt.query {
                // Do not encode implied query
                if !query.ends_with("   ") {
                    context.space().push_prop_str(NodeProperty::Prompt, query);
                }
            }

            context.push_str("\n\n").exit_node();

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
