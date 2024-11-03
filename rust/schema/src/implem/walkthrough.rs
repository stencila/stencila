use serde_json::Value;

use crate::{prelude::*, Walkthrough, WalkthroughStep};

impl Walkthrough {
    /// Custom implementation of [`PatchNode::apply`].
    ///
    /// Only allow operations on the `steps` vector if the walkthrough is expanded, and only passes
    /// through operations to individual steps if the step is active.
    ///
    /// In addition captures operations on [`NodeProperty::IsExpanded`] to deactivate all steps
    /// when set to false.
    pub fn apply_with(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        let Some(PatchSlot::Property(property)) = path.pop_front() else {
            bail!("Invalid patch patch for walkthrough, expected a property slot")
        };

        if matches!(property, NodeProperty::Steps) {
            let apply = if let Some(PatchSlot::Index(index)) = path.front() {
                // If the operation is for a step, then only apply if the walkthrough or the step is active
                self.is_expanded.unwrap_or_default()
                    || self
                        .steps
                        .get(*index)
                        .and_then(|step| step.is_active)
                        .unwrap_or_default()
            } else {
                // If the operation is for the steps vector, only apply if the walkthrough is active
                self.is_expanded.unwrap_or_default()
            };
            if apply {
                self.steps.apply(path, op.clone(), context)?;
            }

            return Ok(true);
        }

        if matches!(property, NodeProperty::IsExpanded) {
            if let PatchOp::Set(PatchValue::Json(Value::Bool(value))) = op {
                // Expand or collapse and reset walkthrough
                if *value {
                    self.is_expanded = Some(true)
                } else {
                    self.is_expanded = None;
                    for step in self.steps.iter_mut() {
                        step.is_active = None;
                    }
                }
            } else {
                // Other patch ops are not expected but handle them anyway
                self.is_expanded
                    .apply(&mut PatchPath::new(), op.clone(), context)?;
            }

            return Ok(true);
        }

        Ok(false)
    }
}

impl MarkdownCodec for Walkthrough {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        for step in &self.steps {
            context
                .enter_node(step.node_type(), step.node_id())
                .push_str("...\n\n");

            // Break the loop if the step is not active. This means that its content
            // and none of the successive steps will be shown.
            if !(self.is_expanded.unwrap_or_default() || step.is_active.unwrap_or_default()) {
                break;
            }

            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    step.content.to_markdown(context);
                })
                .exit_node();
        }

        context.exit_node();
    }
}

impl MarkdownCodec for WalkthroughStep {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // Used only when getting Markdown content to type into source documents
        self.content.to_markdown(context);
    }
}
