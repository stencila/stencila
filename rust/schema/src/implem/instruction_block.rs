use codec_info::{lost_exec_options, lost_options};
use common::{eyre::Ok, tracing};

use crate::{merge, patch, prelude::*, InstructionBlock, InstructionType, Node};

/// Implementation of [`PatchNode`] for [`InstructionBlock`] to customize diffing and
/// patching from Markdown-based formats
impl PatchNode for InstructionBlock {
    fn to_value(&self) -> Result<PatchValue> {
        Ok(PatchValue::Node(Node::InstructionBlock(self.clone())))
    }

    fn from_value(value: PatchValue) -> Result<Self> {
        match value {
            PatchValue::Node(Node::InstructionBlock(block)) => Ok(block),
            PatchValue::Json(value) => Ok(serde_json::from_value(value)?),
            _ => bail!("Invalid value for `InstructionBlock`"),
        }
    }

    fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
        PatchContext::flatten_provenance(vec![
            self.content.provenance(),
            self.suggestions.provenance(),
        ])
    }

    fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
        macro_rules! compare_property {
            ($field:ident) => {
                self.$field.similarity(&other.$field, context)?
            };
        }

        let mut values = vec![
            compare_property!(instruction_type),
            compare_property!(prompt),
            compare_property!(model),
            compare_property!(replicates),
            compare_property!(recursion),
            compare_property!(message),
        ];

        if context
            .format
            .as_ref()
            .map(|format| format.is_markdown_flavor())
            .unwrap_or_default()
        {
            if let Some(other_content) = &other.content {
                // The `other` instruction is from a Markdown-based format so compare its `content` to the active suggestion
                // if there is any, or to the `content` if no suggestions
                let mut compare_content = true;
                if let Some(suggestions) = &self.suggestions {
                    if !suggestions.is_empty() {
                        compare_content = false;

                        let last = suggestions.len().saturating_sub(1);
                        let index = match self.active_suggestion {
                            Some(active) => (active as usize).min(last),
                            None => last,
                        };
                        let suggestion = &suggestions[index];

                        let similarity = suggestion.content.similarity(other_content, context)?;
                        values.push(similarity);
                    }
                }

                if compare_content {
                    if let Some(self_content) = &self.content {
                        let similarity = self_content.similarity(other_content, context)?;
                        values.push(similarity);
                    }
                }
            }
        } else {
            // Calculate similarity based on both content and all suggestion
            values.push(self.content.similarity(&other.content, context)?);
            values.push(self.suggestions.similarity(&other.suggestions, context)?);
        }

        PatchContext::mean_similarity(values)
    }

    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        macro_rules! diff_property {
            ($property:ident, $field:ident) => {
                context.enter_property(NodeProperty::$property);
                self.$field.diff(&other.$field, context)?;
                context.exit_property();
            };
        }

        diff_property!(InstructionType, instruction_type);
        diff_property!(Prompt, prompt);
        diff_property!(Model, model);
        diff_property!(Replicates, replicates);
        diff_property!(Recursion, recursion);
        diff_property!(Message, message);

        if context
            .format
            .as_ref()
            .map(|format| format.is_markdown_flavor())
            .unwrap_or_default()
        {
            // Other node is from a Markdown based format where the `content`` is either the
            // original or the content of the active active suggestion
            if let Some(other_content) = &other.content {
                let suggestions_count = self.suggestions.iter().flatten().count() as u64;
                if let (true, Some(suggestions), Some(active_suggestion)) = (
                    suggestions_count > 0,
                    &self.suggestions,
                    self.active_suggestion,
                ) {
                    let index = active_suggestion.min(suggestions_count - 1) as usize;
                    let suggestion = &suggestions[index];

                    context
                        .enter_property(NodeProperty::Suggestions)
                        .enter_index(index)
                        .enter_property(NodeProperty::Content);
                    suggestion.content.diff(other_content, context)?;
                    context.exit_property().exit_index().exit_property();
                } else {
                    context.enter_property(NodeProperty::Content);
                    self.content.diff(&other.content, context)?;
                    context.exit_property();
                }
            } else {
                context.enter_property(NodeProperty::Content);
                self.content.diff(&other.content, context)?;
                context.exit_property();
            }
        } else {
            // Other node is from a non-Markdown format so
            // calculate diff based on both content and all suggestion
            self.content.diff(&other.content, context)?;
            self.suggestions.diff(&other.suggestions, context)?;
        }

        Ok(())
    }

    fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
        if let Some(node_id) = patch.node_id.as_ref() {
            if node_id == &self.node_id() {
                return patch.apply(self, context);
            }
        } else {
            return patch.apply(self, context);
        }

        macro_rules! patch_properties {
            ($( ($property:ident, $field:expr), )*) => {
                $(
                    context.enter_property(NodeProperty::$property);
                    if $field.patch(patch, context)? {
                        return Ok(true);
                    }
                    context.exit_property();
                )*
            };
        }

        patch_properties!(
            // Core
            (ExecutionMode, self.execution_mode),
            (InstructionType, self.instruction_type),
            (Message, self.message),
            (Prompt, self.prompt),
            (Model, self.model),
            (Replicates, self.replicates),
            (Recursion, self.recursion),
            (Content, self.content),
            (Suggestions, self.suggestions),
            (ActiveSuggestion, self.active_suggestion),
            // Options
            (CompilationDigest, self.options.compilation_digest),
            (CompilationMessages, self.options.compilation_messages),
            (ExecutionDigest, self.options.execution_digest),
            (ExecutionDependencies, self.options.execution_dependencies),
            (ExecutionDependants, self.options.execution_dependants),
            (ExecutionTags, self.options.execution_tags),
            (ExecutionCount, self.options.execution_count),
            (ExecutionRequired, self.options.execution_required),
            (ExecutionStatus, self.options.execution_status),
            (ExecutionInstance, self.options.execution_instance),
            (ExecutionKind, self.options.execution_kind),
            (ExecutionEnded, self.options.execution_ended),
            (ExecutionDuration, self.options.execution_duration),
            (ExecutionMessages, self.options.execution_messages),
            (PromptProvided, self.options.prompt_provided),
        );

        Ok(false)
    }

    fn apply(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        // Handle a patch to archive this instruction
        if path.is_empty() && matches!(op, PatchOp::Archive) {
            // Add this instruction to the root's archive
            context.op_additional(
                PatchPath::from(NodeProperty::Archive),
                PatchOp::Push(self.to_value()?),
            );

            // Get the path and index for applying the additional op
            let mut path = context.path();
            let index = match path.pop_back() {
                Some(PatchSlot::Index(index)) => index,
                slot => bail!("Expected index slot, got: {slot:?}"),
            };

            // Get the accepted content: either the content of the active suggestion, or the original content
            let accepted = match self.active_suggestion {
                Some(active_suggestion) => self
                    .suggestions
                    .iter()
                    .flatten()
                    .nth(active_suggestion as usize)
                    .map(|suggestion| suggestion.content.clone()),
                None => self.content.clone(),
            };

            let Some(mut accepted) = accepted else {
                // No accepted content, so just delete instruction
                context.op_additional(path, PatchOp::Remove(vec![index]));
                return Ok(());
            };

            if accepted.is_empty() {
                // Accepted content is empty, so just delete instruction
                context.op_additional(path, PatchOp::Remove(vec![index]));
                return Ok(());
            }

            // Record the patcher as the acceptor of each block in the accepted content
            let accepter_patch = context.authors_as_acceptors();
            for node in &mut accepted {
                if let Err(error) = patch(node, accepter_patch.clone()) {
                    tracing::error!("While accepting block suggestion: {error}");
                }
            }

            match &self.instruction_type {
                InstructionType::Create => {
                    if accepted.len() == 1 {
                        // Just one block in accepted suggestion, so replace with it
                        context.op_additional(
                            path,
                            PatchOp::Replace(vec![(index, accepted[0].to_value()?)]),
                        );
                    } else {
                        // More than one block in accepted suggestion so remove instruction and
                        // insert blocks in its place
                        let mut blocks = Vec::with_capacity(accepted.len());
                        for (offset, block) in accepted.iter().enumerate() {
                            blocks.push((index + offset, block.to_value()?))
                        }
                        context
                            .op_additional(path.clone(), PatchOp::Remove(vec![index]))
                            .op_additional(path, PatchOp::Insert(blocks));
                    }
                }
                InstructionType::Edit | InstructionType::Fix | InstructionType::Describe => {
                    // Merge the accepted content into the existing content and replace
                    // the instruction with that merged content
                    let merged = if let Some(content) = self.content.as_ref() {
                        let len = content.len().max(accepted.len());
                        let mut merged = Vec::with_capacity(len);
                        for index in 0..len {
                            let old = content.get(index);
                            let new = accepted.get(index);
                            match (old, new) {
                                (Some(old), Some(new)) => {
                                    let mut old = old.clone();
                                    merge(&mut old, new, None, None)?;
                                    merged.push(old);
                                }
                                (Some(old), None) => {
                                    merged.push(old.clone());
                                }
                                (None, Some(new)) => {
                                    merged.push(new.clone());
                                }
                                (None, None) => {}
                            }
                        }
                        merged
                    } else {
                        accepted
                    };

                    let mut blocks = Vec::with_capacity(merged.len());
                    for (offset, block) in merged.iter().enumerate() {
                        blocks.push((index + offset, block.to_value()?))
                    }

                    context
                        .op_additional(path.clone(), PatchOp::Remove(vec![index]))
                        .op_additional(path, PatchOp::Insert(blocks));
                }
            };

            return Ok(());
        }

        let Some(slot) = path.pop_front() else {
            bail!("Patch path for instruction is unexpectedly empty")
        };

        let PatchSlot::Property(property) = slot else {
            bail!("Patch path for instruction starts with index not property")
        };

        // Intercept push and append operations to adjust `active_suggestion` to the new
        // length of the suggestions
        if matches!(property, NodeProperty::Suggestions)
            && matches!(
                op,
                PatchOp::Set(..) | PatchOp::Push(..) | PatchOp::Append(..)
            )
        {
            let old_count = self.suggestions.iter().flatten().count();

            self.suggestions.apply(path, op, context)?;

            let new_count = self.suggestions.iter().flatten().count();
            if new_count > 0 && new_count > old_count {
                self.active_suggestion = Some(new_count.saturating_sub(1) as u64);
            }

            return Ok(());
        }

        // Intercept operations on `active_suggestion` to implement wrapping (for carousel type behavior) including
        // decrement to `None` (i.e. `content`) from 0 (if there is `content`)
        if matches!(property, NodeProperty::ActiveSuggestion) {
            let suggestions_count = self.suggestions.iter().flatten().count();
            if suggestions_count == 0 {
                self.active_suggestion = None;
            } else if matches!(op, PatchOp::Increment) {
                self.active_suggestion = match self.active_suggestion {
                    Some(index) => {
                        if index >= (suggestions_count - 1) as u64 {
                            // Wrap to original, or first suggestion
                            if self.content.is_some() {
                                None
                            } else {
                                Some(0)
                            }
                        } else {
                            Some(index + 1)
                        }
                    }
                    None => Some(0),
                };
            } else if matches!(op, PatchOp::Decrement) {
                self.active_suggestion = match self.active_suggestion {
                    Some(index) => {
                        if index == 0 {
                            // Go to original, or wrap to last suggestion
                            if self.content.is_some() {
                                None
                            } else {
                                Some((suggestions_count - 1) as u64)
                            }
                        } else {
                            Some(index - 1)
                        }
                    }
                    // Wrap from original to last
                    None => Some((suggestions_count - 1) as u64),
                };
            } else if let PatchOp::Set(value) = op {
                self.active_suggestion = match value {
                    PatchValue::None | PatchValue::Json(serde_json::Value::Null) => None,
                    _ => Some(u64::from_value(value)?),
                }
                .map(|value| value.clamp(0, (suggestions_count - 1) as u64));
            }

            return Ok(());
        }

        // Intercept patches to feedback (which does not exist on this type) and apply to the
        // active suggestion
        if matches!(property, NodeProperty::Feedback) {
            if let (Some(index), Some(suggestions)) =
                (self.active_suggestion, &mut self.suggestions)
            {
                if let Some(suggestion) = suggestions.get_mut(index as usize) {
                    path.push_back(PatchSlot::Property(NodeProperty::Feedback));
                    return suggestion.apply(path, op, context);
                }
            } else {
                bail!("Unable to set feedback on instruction because no active suggestion")
            }
        }

        macro_rules! apply_properties {
            ($( ($property:ident, $field:expr), )*) => {
                match property {
                    $(NodeProperty::$property => $field.apply(path, op, context),)*
                    _ => bail!("Patch operation not applied to instruction property `{property}`"),
                }
            };
        }
        apply_properties!(
            // Core
            (ExecutionMode, self.execution_mode),
            (InstructionType, self.instruction_type),
            (Message, self.message),
            (Prompt, self.prompt),
            (Model, self.model),
            (Replicates, self.replicates),
            (Recursion, self.recursion),
            (Content, self.content),
            (Suggestions, self.suggestions),
            // Options
            (CompilationDigest, self.options.compilation_digest),
            (CompilationMessages, self.options.compilation_messages),
            (ExecutionDigest, self.options.execution_digest),
            (ExecutionDependencies, self.options.execution_dependencies),
            (ExecutionDependants, self.options.execution_dependants),
            (ExecutionTags, self.options.execution_tags),
            (ExecutionCount, self.options.execution_count),
            (ExecutionRequired, self.options.execution_required),
            (ExecutionStatus, self.options.execution_status),
            (ExecutionInstance, self.options.execution_instance),
            (ExecutionKind, self.options.execution_kind),
            (ExecutionEnded, self.options.execution_ended),
            (ExecutionDuration, self.options.execution_duration),
            (ExecutionMessages, self.options.execution_messages),
            (PromptProvided, self.options.prompt_provided),
        )
    }
}

impl MarkdownCodec for InstructionBlock {
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
            .merge_losses(lost_exec_options!(self));

        let instruction_type = self.instruction_type.to_string().to_lowercase();

        if matches!(context.format, Format::Myst) {
            context
                .myst_directive(
                    ':',
                    &instruction_type,
                    |context| {
                        if let Some(message) = &self.message {
                            context
                                .push_str(" ")
                                .push_prop_fn(NodeProperty::Message, |context| {
                                    message.to_markdown(context)
                                });
                        }
                    },
                    |context| {
                        if let Some(prompt) = &self.prompt {
                            context.myst_directive_option(
                                NodeProperty::Prompt,
                                Some("prompt"),
                                prompt,
                            );
                        }
                        if let Some(reps) = &self.replicates {
                            context.myst_directive_option(
                                NodeProperty::Replicates,
                                Some("reps"),
                                &reps.to_string(),
                            );
                        }
                    },
                    |context| {
                        // Show the active suggestion (if any) falling back to content (if any)
                        let suggestions_count = self.suggestions.iter().flatten().count() as u64;
                        if let (true, Some(suggestions), Some(active_suggestion)) = (
                            suggestions_count > 0,
                            &self.suggestions,
                            self.active_suggestion,
                        ) {
                            let index = active_suggestion.min(suggestions_count - 1) as usize;
                            let suggestion = &suggestions[index];
                            context.push_prop_fn(NodeProperty::Suggestions, |context| {
                                context
                                    .enter_node(suggestion.node_type(), suggestion.node_id())
                                    .push_prop_fn(NodeProperty::Content, |context| {
                                        suggestion.content.to_markdown(context)
                                    })
                                    .exit_node();
                            });
                        } else if let Some(content) = &self.content {
                            context.push_prop_fn(NodeProperty::Content, |context| {
                                content.to_markdown(context)
                            });
                        }
                    },
                )
                .newline()
                .exit_node();

            return;
        }

        context
            .push_colons()
            .push_str(" ")
            .push_str(&instruction_type)
            .push_str(" ");

        if let Some(prompt) = &self.prompt {
            context.push_str("@").push_str(prompt).push_str(" ");
        }

        if let Some(model) = self
            .model
            .as_ref()
            .and_then(|model| model.id_pattern.as_ref())
        {
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
            .and_then(|model| model.minimum_score.as_ref())
        {
            context
                .push_str("y")
                .push_str(&value.to_string())
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

        if let Some(recursion) = &self.recursion {
            context.push_str(&recursion.to_string()).push_str(" ");
        }

        if let Some(message) = &self.message {
            context.push_prop_fn(NodeProperty::Message, |context| {
                message.to_markdown(context)
            });
        }

        // Show the active suggestion (if any) falling back to content (if any)
        let suggestions_count = self.suggestions.iter().flatten().count() as u64;
        if let (true, Some(suggestions), Some(active_suggestion)) = (
            suggestions_count > 0,
            &self.suggestions,
            self.active_suggestion,
        ) {
            let index = active_suggestion.min(suggestions_count - 1) as usize;
            let suggestion = &suggestions[index];

            if suggestion.content.len() == 1 {
                context.push_str(" >>");
            }
            context.newline().newline();

            context
                .increase_depth()
                .push_prop_fn(NodeProperty::Suggestions, |context| {
                    context
                        .enter_node(suggestion.node_type(), suggestion.node_id())
                        .push_prop_fn(NodeProperty::Content, |context| {
                            suggestion.content.to_markdown(context)
                        })
                        .exit_node();
                })
                .decrease_depth();

            if suggestion.content.len() != 1 {
                context.push_colons().newline().newline();
            }
        } else if let Some(content) = &self.content {
            if content.len() == 1 {
                context.push_str(" >>");
            }
            context.newline().newline();

            context.push_prop_fn(NodeProperty::Content, |context| {
                content.to_markdown(context)
            });

            if content.len() != 1 {
                context.push_colons().newline().newline();
            }
        } else {
            context.push_str(" <<").newline().newline();
        }

        context.exit_node();
    }
}
