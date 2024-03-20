use common::itertools::Itertools;
use schema::{ForBlock, Section, SectionType};

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for ForBlock {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Pending ForBlock {node_id}");

        pending_impl!(executor, &node_id);

        // Break to avoid making executable nodes in `content` as pending
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_code(&node_id) {
            tracing::debug!("Skipping ForBlock {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Executing ForBlock {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        let started = Timestamp::now();

        let mut messages = Vec::new();
        let mut has_iterations = false;
        let mut is_empty = true;

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
                        vec![error_to_message("While evaluating expression", error)],
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

            // Clone the `content` for each iteration
            let iterations = iterator
                .iter()
                .map(|_| Section {
                    section_type: Some(SectionType::Iteration),
                    content: self.content.clone(),
                    ..Default::default()
                })
                .collect_vec();

            // Update `iterations` and then load back from the store. This is necessary so that we obtain distinct
            // `NodeId`s for the executable nodes from the store (they will just be cloned ids now) in each
            // iteration so that when they are executed, the correct node is updated
            let mut iterations: Vec<Section> = match executor
                .swap_property(&node_id, Property::Iterations, iterations.into())
                .await
            {
                Ok(iterations) => iterations,
                Err(error) => {
                    messages.push(error_to_message("While loading iterations", error));
                    Vec::new()
                }
            };

            // Iterate over iterable, and iterations, setting the variable and executing each iteration.
            for (node, iteration) in iterator.iter().zip(iterations.iter_mut()) {
                has_iterations = true;

                // Set the loop's variable
                if let Err(error) = executor.kernels.write().await.set(variable, node).await {
                    messages.push(error_to_message("While setting iteration variable", error));
                };

                // Execute the iteration
                if let Err(error) = iteration.walk_async(executor).await {
                    messages.push(error_to_message("While executing iteration", error));
                }
            }

            // Remove the loop's variable (if it was set)
            if has_iterations {
                if let Err(error) = executor.kernels.write().await.remove(&self.variable).await {
                    messages.push(error_to_message("While removing iteration variable", error));
                }
            };
        }

        // If there were no iterations and `otherwise` is some, then execute that
        if let (true, Some(otherwise)) = (!has_iterations, self.otherwise.as_mut()) {
            is_empty = false;

            if let Err(error) = otherwise.walk_async(executor).await {
                messages.push(error_to_message("While executing otherwise", error))
            }
        }

        let messages = (!messages.is_empty()).then_some(messages);

        let ended = Timestamp::now();

        if !is_empty {
            let status = execution_status(&messages);
            let required = execution_required(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            executor.replace_properties(
                &node_id,
                [
                    (Property::ExecutionStatus, status.into()),
                    (Property::ExecutionRequired, required.into()),
                    (Property::ExecutionMessages, messages.into()),
                    (Property::ExecutionDuration, duration.into()),
                    (Property::ExecutionEnded, ended.into()),
                    (Property::ExecutionCount, count.into()),
                ],
            );
        } else {
            executor.replace_properties(
                &node_id,
                [
                    (Property::Iterations, Value::None),
                    (Property::ExecutionStatus, ExecutionStatus::Empty.into()),
                    (Property::ExecutionRequired, ExecutionRequired::No.into()),
                    (Property::ExecutionMessages, messages.into()),
                    (Property::ExecutionDuration, Value::None),
                    (Property::ExecutionEnded, Value::None),
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
