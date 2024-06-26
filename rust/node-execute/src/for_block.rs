use codec_cbor::r#trait::CborCodec;
use schema::{replicate, Block, ForBlock, Section, SectionType};

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for ForBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling ForBlock {node_id}");

        let language = self.programming_language.as_deref().unwrap_or_default();
        let ParseInfo {
            mut compilation_digest,
            ..
        } = parsers::parse(&self.code, language);

        // Add a digest of the `content` to the state digest given that
        // if the content changes all the `iterations` become stale.
        // Use CBOR for this since is it faster and more compact to encode than JSON etc
        match self.content.to_cbor() {
            Ok(bytes) => add_to_digest(&mut compilation_digest.state_digest, &bytes),
            Err(error) => {
                tracing::error!("While encoding `content` to CBOR: {error}")
            }
        }

        executor.patch(
            &node_id,
            [set(NodeProperty::CompilationDigest, compilation_digest)],
        );

        // Walk over `otherwise` here because this function returns `Break` so it
        // will not be walked over otherwise (pardon the pun) but needs to be
        if let Err(error) = self.otherwise.walk_async(executor).await {
            tracing::error!("While compiling `otherwise`: {error}")
        }

        // Break walk to avoid walking over `content` (already captured in state digest)
        // and `iterations` (compilation digest is not required)
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Pending ForBlock {node_id}");

        pending_impl!(executor, &node_id);

        // Break to avoid making executable nodes in `content` as pending
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_code(
            &node_id,
            &self.auto_exec,
            &self.options.compilation_digest,
            &self.options.execution_digest,
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
        let code = self.code.trim();
        if !(variable.is_empty() && code.is_empty()) {
            is_empty = false;

            // Evaluate code in kernels to get the iterable
            let (output, mut code_messages) = executor
                .kernels
                .write()
                .await
                .evaluate(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_execution_message(
                            "While evaluating expression",
                            error,
                        )],
                    )
                });
            messages.append(&mut code_messages);

            // Derive an iterator from the code's output value
            let iterator = match output {
                Node::Null(..) => vec![],
                Node::Boolean(bool) => {
                    if bool {
                        vec![output]
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
                _ => vec![],
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
                if let Err(error) = executor.kernels.write().await.set(variable, node).await {
                    messages.push(error_to_execution_message(
                        "While setting iteration variable",
                        error,
                    ));
                };

                // Execute the iteration
                if let Err(error) = iteration.walk_async(executor).await {
                    messages.push(error_to_execution_message(
                        "While executing iteration",
                        error,
                    ));
                }
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

        let messages = (!messages.is_empty()).then_some(messages);

        let ended = Timestamp::now();

        if !is_empty {
            let status = execution_status(&messages);
            let required = execution_required(&status);
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
