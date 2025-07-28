use codec_markdown::decode_frontmatter;
use schema::{diff, Article, NodeSlot, NodeType, Reference};

use crate::{interrupt_impl, prelude::*, HeadingInfo};

impl Executable for Article {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Article {node_id}");

        let mut messages = Vec::new();

        // Check frontmatter for syntactic and semantic errors
        if let Some(yaml) = self.frontmatter.as_deref() {
            let (.., mut fm_messages) = decode_frontmatter(yaml, Some(NodeType::Article));
            messages.append(&mut fm_messages);
        }

        if messages.is_empty() {
            self.options.compilation_messages = None;
            executor.patch(&node_id, [none(NodeProperty::CompilationMessages)]);
        } else {
            self.options.compilation_messages = Some(messages.clone());
            executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);
        }

        // Clear the executor's headings
        executor.headings.clear();

        // Add references that have an id and/or DOI to the executor's targets
        // so that citations can link to them
        executor.bibliography.clear();
        for reference in self.references.iter().flatten() {
            // Note that we allow for each reference to be targeted using either
            // custom id or DOI
            if let Some(id) = &reference.id {
                if !executor.bibliography.contains_key(id) {
                    executor.bibliography.insert(id.into(), reference.clone());
                }
            }
            if let Some(doi) = &reference.doi {
                if !executor.bibliography.contains_key(doi) {
                    executor.bibliography.insert(doi.into(), reference.clone());
                }
            }
        }

        // Compile `content` and other properties
        if let Err(error) = async {
            self.title.walk_async(executor).await?;
            self.r#abstract.walk_async(executor).await?;
            self.content.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While compiling article: {error}")
        }

        // Ensure any trailing headings are collapsed into their parents
        HeadingInfo::collapse(1, &mut executor.headings);

        // Transform the executors heading info
        let headings = (!executor.headings.is_empty())
            .then(|| HeadingInfo::into_list(executor.headings.drain(..).collect()));

        // Diff the headings list with the current, prepend any generated diff ops
        // with the path to headings and send a patch if necessary
        match diff(&self.headings, &headings, None, None) {
            Ok(mut patch) => {
                if !patch.ops.is_empty() {
                    patch.node_id = Some(node_id);
                    patch.prepend_paths(vec![NodeSlot::Property(NodeProperty::Headings)]);
                    executor.send_patch(patch);
                }
            }
            Err(error) => {
                tracing::error!("While diffing article headings: {error}")
            }
        }

        // Break because properties compiled above
        WalkControl::Break
    }

    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Article {node_id}");

        // Clear the executor's references list
        executor.references.clear();

        // Link `content` and other properties
        if let Err(error) = async {
            self.title.walk_async(executor).await?;
            self.r#abstract.walk_async(executor).await?;
            self.content.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While linking article: {error}")
        }

        // Update the article's references leaving only those that are cited.
        let references: Vec<Reference> = executor
            .references
            .iter()
            .filter_map(|id| executor.bibliography.get(id).cloned())
            .collect();
        if references.is_empty() {
            executor.patch(&node_id, [none(NodeProperty::References)]);
        } else if let Some(current) = &self.references {
            match diff(current, &references, None, None) {
                Ok(mut patch) => {
                    if !patch.ops.is_empty() {
                        patch.node_id = Some(node_id);
                        patch.prepend_paths(vec![NodeSlot::Property(NodeProperty::References)]);
                        executor.send_patch(patch);
                    }
                }
                Err(error) => {
                    tracing::error!("While diffing article references: {error}")
                }
            }
        } else {
            executor.patch(&node_id, [set(NodeProperty::References, references)]);
        }

        // Break because properties linked above
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing Article {node_id}");

        // Add article metadata to document context
        executor.document_context.metadata = (&*self).into();

        // Set execution status
        self.options.execution_status = Some(ExecutionStatus::Pending);
        executor.patch(
            &node_id,
            [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
        );

        // Continue to prepare executable nodes within properties
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Executing Article {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let started = Timestamp::now();

        let messages = if let Err(error) = async {
            self.title.walk_async(executor).await?;
            self.content.walk_async(executor).await
        }
        .await
        {
            Some(vec![error_to_execution_message(
                "While executing article",
                error,
            )])
        } else {
            None
        };

        let ended = Timestamp::now();

        // TODO: set status based on the execution status of
        // child executable nodes

        let status = execution_status(&messages);
        let required = execution_required_status(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, status),
                set(NodeProperty::ExecutionRequired, required),
                set(NodeProperty::ExecutionMessages, messages),
                set(NodeProperty::ExecutionDuration, duration),
                set(NodeProperty::ExecutionEnded, ended),
                set(NodeProperty::ExecutionCount, count),
            ],
        );

        // Break because properties already executed above
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting Article {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content` and other properties
        WalkControl::Continue
    }
}
