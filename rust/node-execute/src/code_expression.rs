use schema::{CodeExpression, ExecutionMode};

use crate::{interrupt_impl, prelude::*};

impl Executable for CodeExpression {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling CodeExpression {node_id}");

        // Get the programming language, falling back to using the executor's current language
        let lang = executor.programming_language(&self.programming_language);

        // Parse the code to determine if it or the language has changed since last time
        let info = parsers::parse(&self.code, &lang, &self.options.compilation_digest);

        // Add code to the linting context
        executor.linting_code(&node_id, &self.code.to_string(), &lang, info.changed.yes());

        // Return early if no change
        if info.changed.no() {
            tracing::trace!("Skipping compiling CodeExpression {node_id}");

            return WalkControl::Break;
        }

        let execution_required =
            execution_required_digests(&self.options.execution_digest, &info.compilation_digest);
        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, info.compilation_digest),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing CodeExpression {node_id}");

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            &self.execution_mode.or(Some(ExecutionMode::Always)),
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            self.options.execution_status = Some(status);
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Break the walk since none of the child nodes are executed
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            tracing::trace!("Skipping CodeExpression {node_id}");
            return WalkControl::Break;
        }

        tracing::debug!("Executing CodeExpression {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let compilation_digest = self.options.compilation_digest.clone();

        if !self.code.trim().is_empty() {
            let started = Timestamp::now();

            // Get the programming language, falling back to using the executor's current language
            let lang = executor.programming_language(&self.programming_language);

            let (output, messages, instance) = executor
                .kernels
                .write()
                .await
                .evaluate(&self.code, lang.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_execution_message(
                            "While evaluating expression",
                            error,
                        )],
                        String::new(),
                    )
                });

            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            // Set properties that may be using in rendering
            self.output = Some(Box::new(output.clone()));
            self.options.execution_messages = messages.clone();

            // Patch outputs using kernel instance as `AuthorRole` if possible
            if let Some(author) = executor.node_execution_author_role(&instance).await {
                executor.patch_with_authors(
                    &node_id,
                    vec![author],
                    [set(NodeProperty::Output, output)],
                );
            } else {
                executor.patch(&node_id, [set(NodeProperty::Output, output)]);
            }

            executor.patch(
                &node_id,
                [
                    set(NodeProperty::ExecutionStatus, status),
                    set(NodeProperty::ExecutionRequired, required),
                    set(NodeProperty::ExecutionMessages, messages),
                    set(NodeProperty::ExecutionDuration, duration),
                    set(NodeProperty::ExecutionEnded, ended),
                    set(NodeProperty::ExecutionCount, count),
                    set(NodeProperty::ExecutionDigest, compilation_digest),
                ],
            );
        } else {
            executor.patch(
                &node_id,
                [
                    set(NodeProperty::ExecutionStatus, ExecutionStatus::Empty),
                    set(NodeProperty::ExecutionRequired, ExecutionRequired::No),
                    none(NodeProperty::ExecutionMessages),
                    none(NodeProperty::ExecutionDuration),
                    none(NodeProperty::ExecutionEnded),
                    set(NodeProperty::ExecutionDigest, compilation_digest),
                ],
            );
        };

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting CodeExpression {node_id}");

        interrupt_impl!(self, executor, &node_id);

        WalkControl::Break
    }
}
