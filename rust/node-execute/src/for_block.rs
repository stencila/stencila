use codec_cbor::r#trait::CborCodec;
use schema::{Block, ExecutionMode, ForBlock, Section, SectionType, replicate};

use crate::{interrupt_impl, prelude::*};

impl Executable for ForBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling ForBlock {node_id}");

        // Get the programming language, falling back to using the executor's current language
        let lang = executor.programming_language(&self.programming_language);

        // Parse the code to determine if it or the language has changed since last time
        let mut info = parsers::parse(&self.code, &lang, &self.options.compilation_digest);

        // Add variable and code to linting context
        executor.linting_variable(&self.variable, &lang, info.changed.yes());
        executor.linting_code(&node_id, &self.code.to_string(), &lang, info.changed.yes());

        // Compile content. Do this here so any updates to content during compilation
        // are included in the following state digest. Also for linting of content.
        if let Err(error) = self.content.walk_async(executor).await {
            tracing::error!("While compiling `ForBlock.content`: {error}");
        }

        // Add a digest of the `content` to the state digest given that
        // if the content changes all the `iterations` become stale.
        // Use CBOR for this since is it faster and more compact to encode than JSON etc
        match self.content.to_cbor() {
            Ok(bytes) => add_to_digest(&mut info.compilation_digest.state_digest, &bytes),
            Err(error) => {
                tracing::error!("While encoding `content` to CBOR: {error}")
            }
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

        // Walk over `otherwise` here because this function returns `Break` so it
        // will not be walked over otherwise (pardon the pun) but needs to be
        if let Err(error) = self.otherwise.walk_async(executor).await {
            tracing::error!("While compiling `otherwise`: {error}")
        }

        // Break walk: `content` already compiled above, and avoid walking over `iterations`
        // (compilation digest is not required)
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing ForBlock {node_id}");

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            // Until dependency analysis is implemented, defaults to always executing
            &self.execution_mode.or(Some(ExecutionMode::Always)),
            &self.options.execution_required,
        ) {
            self.options.execution_status = Some(status);
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Break to avoid making executable nodes in `content` as pending
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            tracing::trace!("Skipping ForBlock {node_id}");
            return WalkControl::Break;
        }

        tracing::debug!("Executing ForBlock {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let started = Timestamp::now();

        let mut messages = Vec::new();
        let mut has_iterations = false;
        let mut is_empty = true;

        let compilation_digest = self.options.compilation_digest.clone();
        let variable = self.variable.trim();
        let trimmed = self.code.trim();
        let mut iterations = Vec::new();
        if !(variable.is_empty() && trimmed.trim().is_empty()) {
            is_empty = false;

            // The value to iterate over
            let mut value = None;

            // If the programming language is none, and the code matches a variable name,
            // then try to get that variable to use as the value
            if self.programming_language.is_none() && is_valid_variable_name(trimmed) {
                if let Ok(Some(node)) = executor.kernels.read().await.get(trimmed).await {
                    value = Some(node);
                }
            }

            // Get the programming language, falling back to using the executor's current language
            let lang = executor.programming_language(&self.programming_language);

            let value = if let Some(value) = value {
                value
            } else {
                // Evaluate code in kernels to get the iterable
                let (output, mut code_messages, _instance) = executor
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
                messages.append(&mut code_messages);

                output
            };

            // Derive an iterator from the code's output value
            let iterator = match value {
                Node::Null(..) => vec![],
                Node::Boolean(bool) => {
                    if bool {
                        vec![value]
                    } else {
                        vec![]
                    }
                }
                Node::Integer(int) => {
                    if int > 0 {
                        (0..(int as u64)).map(Node::UnsignedInteger).collect()
                    } else {
                        vec![]
                    }
                }
                Node::UnsignedInteger(uint) => {
                    if uint > 0 {
                        (0..uint).map(Node::UnsignedInteger).collect()
                    } else {
                        vec![]
                    }
                }
                Node::Number(num) => {
                    if num > 0. {
                        (0..(num as u64)).map(Node::UnsignedInteger).collect()
                    } else {
                        vec![]
                    }
                }
                Node::String(string) => string
                    .chars()
                    .map(|char| Node::String(char.to_string()))
                    .collect(),
                Node::Array(array) => array.iter().map(|item| item.clone().into()).collect(),
                Node::Object(object) => object
                    .iter()
                    .map(|(key, value)| {
                        Node::Array(Array(vec![Primitive::String(key.clone()), value.clone()]))
                    })
                    .collect(),
                Node::Datatable(dt) => dt.to_values().into_iter().map(Node::Object).collect(),
                _ => {
                    messages.push(ExecutionMessage::new(
                        MessageLevel::Warning,
                        format!("Expression evaluated to a non-iterable type: {value}"),
                    ));
                    Vec::new()
                }
            };

            // Clear any existing iterations while ensuring an array to push to later
            let reset = if self.iterations.is_some() {
                clear(NodeProperty::Iterations)
            } else {
                set(NodeProperty::Iterations, Vec::<Section>::new())
            };
            executor.patch(&node_id, [reset]);

            // Iterate over iterable, and iterations, setting the variable and executing each iteration.
            for node in iterator.iter() {
                has_iterations = true;

                // Replicate the content, rather than clone it so it has different
                // ids from the original when executed.
                let content = replicate(&self.content).unwrap_or_default();

                // Add the iteration so it can be patched when it is executed
                let mut iteration = Block::Section(Section {
                    section_type: Some(SectionType::Iteration),
                    content,
                    ..Default::default()
                });
                executor.patch(
                    &node_id,
                    [push(NodeProperty::Iterations, iteration.clone())],
                );

                // Set the loop's variable
                if let Err(error) = executor
                    .kernels
                    .write()
                    .await
                    .set(variable, node, lang.as_deref())
                    .await
                {
                    messages.push(error_to_execution_message(
                        "While setting iteration variable",
                        error,
                    ));
                };

                // Execute the iteration
                // Temporarily remove any executor node ids so that nodes within
                // the iteration content are executed.
                let node_ids = executor.node_ids.take();
                if let Err(error) = executor.compile_prepare_execute(&mut iteration).await {
                    messages.push(error_to_execution_message(
                        "While executing iteration",
                        error,
                    ));
                }
                executor.node_ids = node_ids;

                // Store iteration for using later
                iterations.push(iteration)
            }

            // Remove the loop's variable (if it was set)
            if has_iterations {
                if let Err(error) = executor.kernels.write().await.remove(&self.variable).await {
                    messages.push(error_to_execution_message(
                        "While removing iteration variable",
                        error,
                    ));
                }
            };
        }

        // If there were no iterations and `otherwise` is some, then execute that
        if let (true, Some(otherwise)) = (!has_iterations, self.otherwise.as_mut()) {
            is_empty = false;

            if let Err(error) = otherwise.walk_async(executor).await {
                messages.push(error_to_execution_message(
                    "While executing otherwise",
                    error,
                ))
            }
        }

        let ended = Timestamp::now();

        let messages = (!messages.is_empty()).then_some(messages);

        if !is_empty {
            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            // Set properties that may be using in rendering
            self.iterations = has_iterations.then_some(iterations);
            self.options.execution_messages = messages.clone();

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
                    none(NodeProperty::Iterations),
                    set(NodeProperty::ExecutionStatus, ExecutionStatus::Empty),
                    set(NodeProperty::ExecutionRequired, ExecutionRequired::No),
                    set(NodeProperty::ExecutionMessages, messages),
                    none(NodeProperty::ExecutionDuration),
                    none(NodeProperty::ExecutionEnded),
                    set(NodeProperty::ExecutionDigest, compilation_digest),
                ],
            );
        }

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting ForBlock {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`, `iterations`,
        // and `otherwise`
        WalkControl::Continue
    }
}
