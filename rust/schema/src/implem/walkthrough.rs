use serde_json::Value;

use crate::{prelude::*, Walkthrough, WalkthroughStep};

impl Walkthrough {
    /// Custom implementation of [`PatchNode::apply`].
    ///
    /// Only allow operations on the `steps` vector if the walkthrough is expanded, and only passes
    /// through operations to individual steps if the step is active.
    ///
    /// In addition captures operations on [`NodeProperty::IsCollapsed`] to also collapse all steps.
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
                // If the operation is for a step, then only apply if the walkthrough or the step is not collapsed
                !self.is_collapsed.unwrap_or_default()
                    && !self
                        .steps
                        .get(*index)
                        .and_then(|step| step.is_collapsed)
                        .unwrap_or_default()
            } else {
                // If the operation is for the steps vector (e.g. adding or removing a step),
                // only apply if the walkthrough is not collapsed
                !self.is_collapsed.unwrap_or_default()
            };
            if apply {
                self.steps.apply(path, op.clone(), context)?;
            }

            // If, after applying the patch to the step, all steps are now expanded
            // then ensure that the walkthrough is expanded. This allows the user
            // to edit the walkthrough (e.g add more steps) when it is finished
            if self
                .steps
                .iter()
                .all(|step| !matches!(step.is_collapsed, Some(true)))
            {
                self.is_collapsed = None;
            }

            return Ok(true);
        }

        if matches!(property, NodeProperty::IsCollapsed) {
            if let PatchOp::Set(PatchValue::Json(Value::Bool(value))) = op {
                // Expand or collapse the walkthrough
                if *value {
                    self.is_collapsed = Some(true);
                    for step in self.steps.iter_mut() {
                        step.is_collapsed = Some(true);
                    }
                } else {
                    self.is_collapsed = None;
                    for step in self.steps.iter_mut() {
                        step.is_collapsed = None;
                    }
                }
            } else {
                // Other patch ops are not expected but handle them anyway
                self.is_collapsed
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

            // Break the loop if the step is collapsed. This means that the content
            // of this step and all the successive steps will be not shown.
            if step.is_collapsed.unwrap_or_default() {
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
