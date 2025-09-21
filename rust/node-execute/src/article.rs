use itertools::Itertools;

use stencila_codec_biblio::encode::render_citations;
use stencila_codec_markdown::decode_frontmatter;
use stencila_schema::{Article, NodeSlot, NodeType, diff};

use crate::{HeadingInfo, interrupt_impl, prelude::*};

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

        // Clear the executor's headings before walking over content so that
        // they can be repopulated
        executor.headings.clear();

        // Clear the executor's bibliography and add the article's references
        // before walking over content so that citations can link to them
        executor.bibliography.clear();
        for reference in self.references.iter().flatten() {
            // Note that we allow for each reference to be targeted using either
            // custom id or DOI but if a reference has neither then they are not
            // able to be added
            if let Some(id) = &reference.id
                && !executor.bibliography.contains_key(id)
            {
                executor.bibliography.insert(id.into(), reference.clone());
            }
            if let Some(doi) = &reference.doi
                && !executor.bibliography.contains_key(doi)
            {
                executor.bibliography.insert(doi.into(), reference.clone());
            }
        }

        // Clear the executor's citations list before waling over content so
        // that citations and citation groups can add themselves to it
        executor.citations.clear();

        // Walk over `content` and other properties to compile the article
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

        // Transform the executor's heading info into a list
        let headings = (!executor.headings.is_empty())
            .then(|| HeadingInfo::into_list(executor.headings.drain(..).collect()));

        // Diff the headings list with the current, prepend any generated diff ops
        // with the path to headings and send a patch if necessary
        match diff(&self.options.headings, &headings, None, None) {
            Ok(mut patch) => {
                if !patch.ops.is_empty() {
                    patch.node_id = Some(node_id.clone());
                    patch.prepend_paths(vec![NodeSlot::Property(NodeProperty::Headings)]);
                    executor.send_patch(patch);
                }
            }
            Err(error) => {
                tracing::error!("While diffing article headings: {error}")
            }
        }

        // Render the executor's citations, using the article's citation style,
        // so that the rendered content can be applied to citations, citation groups,
        // and the article's references.
        let citation_style = self
            .options
            .config
            .as_ref()
            .and_then(|config| config.citation_style.as_deref());
        let citations = executor
            .citations
            .iter()
            .map(|(.., (citation_group, ..))| citation_group)
            .collect_vec();
        match render_citations(citations, citation_style).await {
            Ok((citations_content, references)) => {
                // Assign the rendered citation content to each citation or citation or citation group
                // so they can be applied to those in the `link` phase.
                executor
                    .citations
                    .iter_mut()
                    .zip(citations_content)
                    .for_each(|((.., (.., old_content)), new_content)| {
                        *old_content = Some(new_content);
                    });

                // Assign the rendered references content to each of the article's references
                if references.is_empty() {
                    if self.references.is_some() {
                        self.references = None;
                        executor.patch(&node_id, [none(NodeProperty::References)]);
                    }
                } else {
                    self.references = Some(references.clone());
                    executor.patch(&node_id, [set(NodeProperty::References, references)]);
                }
            }
            Err(error) => {
                messages.push(error_to_compilation_message(error));
            }
        }

        // Update compilation messages
        if messages.is_empty() {
            if self.options.compilation_messages.is_some() {
                self.options.compilation_messages = None;
                executor.patch(&node_id, [none(NodeProperty::CompilationMessages)]);
            }
        } else {
            self.options.compilation_messages = Some(messages.clone());
            executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);
        }

        // Break because properties compiled above
        WalkControl::Break
    }

    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Article {node_id}");

        // Link `content` and other properties of the article
        if let Err(error) = async {
            self.title.walk_async(executor).await?;
            self.r#abstract.walk_async(executor).await?;
            self.content.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While linking article: {error}")
        }

        // Break because properties linked above
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing Article {node_id}");

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
