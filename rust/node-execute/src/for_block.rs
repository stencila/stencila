use schema::{ForBlock, Section, SectionType};

use crate::prelude::*;

impl Executable for ForBlock {
    #[tracing::instrument(skip_all)]
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        tracing::trace!("Executing ForBlock {}", self.node_id());

        // Ensure iterations array and clear any existing iterations
        let iterations = self.iterations.get_or_insert_with(Vec::new);
        iterations.clear();

        let started = Timestamp::now();
        let mut messages = Vec::new();

        let mut not_empty = false;

        let code = self.code.trim();
        if !code.is_empty() {
            not_empty = true;

            // Evaluate code in kernels to get the iterable
            let (output, mut code_messages) = executor
                .kernels
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

            // Iterate over iterable, executing the content each time and adding to `iterations` field
            for node in &iterator {
                // Set the loop's variable
                if let Err(error) = executor.kernels.set(&self.variable, node).await {
                    messages.push(ExecutionMessage::new(
                        MessageLevel::Error,
                        error.to_string(),
                    ));
                };

                // Clone the content, execute it and add it as an iteration
                let mut content = self.content.clone();
                if let Err(error) = content.walk_async(executor).await {
                    messages.push(error_to_message("While executing iteration", error));
                }
                iterations.push(Section {
                    section_type: Some(SectionType::Iteration),
                    content,
                    ..Default::default()
                });
            }

            // Remove the loop's variable
            if let Err(error) = executor.kernels.remove(&self.variable).await {
                messages.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    error.to_string(),
                ));
            };
        }

        // If there were no iterations and `otherwise` is some, then execute that
        if let (true, Some(otherwise)) = (iterations.is_empty(), self.otherwise.as_mut()) {
            not_empty = true;

            if let Err(error) = otherwise.walk_async(executor).await {
                messages.push(error_to_message("While executing otherwise", error))
            }
        }

        let ended = Timestamp::now();

        if not_empty {
            self.options.execution_status = execution_status(&messages);
            self.options.execution_required = execution_required(&self.options.execution_status);
            self.options.execution_messages = execution_messages(messages);
            self.options.execution_duration = execution_duration(&started, &ended);
            self.options.execution_ended = Some(ended);
            self.options.execution_count.get_or_insert(0).add_assign(1);
        } else {
            self.options.execution_status = Some(ExecutionStatus::Empty);
            self.options.execution_required = Some(ExecutionRequired::No);
            self.options.execution_messages = None;
            self.options.execution_ended = None;
            self.options.execution_duration = None;
        }

        WalkControl::Break
    }
}
