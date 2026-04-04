use std::hash::{Hash, Hasher};

use stencila_schema::{CompilationDigest, CompilationMessage, Figure, LabelType, NodeProperty};

use crate::prelude::*;

impl Executable for Figure {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Figure {node_id}");

        // Update automatic label if necessary.
        // Figures nested inside another figure get subfigure labels (e.g. "1A")
        // rather than incrementing the top-level figure count.
        if self.label_automatically.unwrap_or(true) {
            let label = if executor.has_figure_ancestor() {
                executor.subfigure_label()
            } else {
                executor.figure_label()
            };
            if Some(&label) != self.label.as_ref() {
                self.label = Some(label.clone());
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        // Auto-generate an id from the label if the figure doesn't already
        // have a user-supplied id
        if let Some(label) = &self.label
            && let Some(id) = Executor::auto_id(&LabelType::FigureLabel, label, &self.id)
        {
            self.id = Some(id.clone());
            executor.patch(&node_id, [set(NodeProperty::Id, id)]);
        }

        // If have id and label then register as a link target
        if let (Some(id), Some(label)) = (&self.id, &self.label) {
            executor
                .labels
                .insert(id.clone(), (LabelType::FigureLabel, label.clone()));
        }

        // Compile overlay if present
        if let Some(overlay) = &self.options.overlay {
            let new_digest = {
                let mut hasher = seahash::SeaHasher::new();
                overlay.hash(&mut hasher);
                hasher.finish()
            };
            let needs_compile = self
                .options
                .compilation_digest
                .as_ref()
                .is_none_or(|d| d.state_digest != new_digest);

            if needs_compile {
                let result = stencila_svg_components::compile(overlay);

                let messages: Option<Vec<CompilationMessage>> = if result.messages.is_empty() {
                    None
                } else {
                    Some(
                        result
                            .messages
                            .into_iter()
                            .map(|m| CompilationMessage {
                                level: match m.level {
                                    stencila_svg_components::diagnostics::MessageLevel::Warning => {
                                        stencila_schema::MessageLevel::Warning
                                    }
                                    stencila_svg_components::diagnostics::MessageLevel::Error => {
                                        stencila_schema::MessageLevel::Error
                                    }
                                },
                                message: m.message,
                                ..Default::default()
                            })
                            .collect(),
                    )
                };

                let digest = CompilationDigest {
                    state_digest: new_digest,
                    ..Default::default()
                };

                executor.patch(
                    &node_id,
                    [
                        set(NodeProperty::OverlayCompiled, result.compiled),
                        set(NodeProperty::CompilationMessages, messages),
                        set(NodeProperty::CompilationDigest, digest),
                    ],
                );
            }
        } else if self.options.overlay_compiled.is_some() {
            // Overlay was removed — clear compiled output
            executor.patch(
                &node_id,
                [
                    none(NodeProperty::OverlayCompiled),
                    none(NodeProperty::CompilationMessages),
                    none(NodeProperty::CompilationDigest),
                ],
            );
        }

        WalkControl::Continue
    }
}
