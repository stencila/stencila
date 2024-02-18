use std::ops::AddAssign;

use common::{
    eyre::{Report, Result},
    itertools::Itertools,
};
use kernels::Kernels;
use schema::{
    walk::{VisitorAsync, WalkControl, WalkNode},
    Array, Block, CodeChunk, CodeExpression, Duration, ExecutionMessage, ExecutionStatus, ForBlock,
    IfBlock, IfBlockClause, Inline, MessageLevel, Node, Null, Primitive, Section, SectionType,
    Timestamp,
};

/// Walk over a node and execute it and all its child nodes
pub async fn execute<T: WalkNode>(node: &mut T, kernels: &mut Kernels) -> Result<()> {
    let mut executor = Executor { kernels };
    node.walk_async(&mut executor).await
}

/// A visitor that walks over a tree of nodes and collects
/// execution tasks from nodes

struct Executor<'lt> {
    kernels: &'lt mut Kernels,
}

impl<'lt> VisitorAsync for Executor<'lt> {
    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        use Block::*;
        Ok(match block {
            // TODO: CallBlock(node) => node.execute(self).await,
            CodeChunk(node) => node.execute(self).await,
            ForBlock(node) => node.execute(self).await,
            IfBlock(node) => node.execute(self).await,
            // TODO: IncludeBlock(node) => node.execute(self).await,
            // TODO: InstructionBlock(node) => node.execute(self).await,
            // TODO: MathBlock(node) => node.execute(self).await,
            // TODO: StyledBlock(node) => node.execute(self).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_inline(&mut self, inline: &mut Inline) -> Result<WalkControl> {
        use Inline::*;
        Ok(match inline {
            CodeExpression(node) => node.execute(self).await,
            // TODO: InstructionInline(node) => node.execute(self).await,
            // TODO: MathInline(node) => node.execute(self).await,
            // TODO: StyledInline(node) => node.execute(self).await,
            _ => WalkControl::Continue,
        })
    }
}

/// A trait for an executable node
trait Executable {
    /// Execute the node
    ///
    /// Note that this method is intentionally infallible because we want
    /// executable nodes to handle any errors associated with their execution
    /// and record them in `execution_messages` so that they are visible
    /// to the user.
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl;
}

/// Create an `ExecutionMessage` from an `eyre::Report`
fn error_to_message(context: &str, error: Report) -> ExecutionMessage {
    ExecutionMessage {
        level: MessageLevel::Error,
        message: error.to_string(),
        stack_trace: Some(context.to_string()),
        ..Default::default()
    }
}

/// Create a value for `execution_status` from a vector of `ExecutionMessage`s
fn execution_status(messages: &[ExecutionMessage]) -> Option<ExecutionStatus> {
    messages
        .iter()
        .any(|message| message.level == MessageLevel::Error)
        .then_some(ExecutionStatus::Failed)
        .or(Some(ExecutionStatus::Succeeded))
}

/// Create a value for `execution_messages` from a vector of `ExecutionMessage`s
fn execution_messages(messages: Vec<ExecutionMessage>) -> Option<Vec<ExecutionMessage>> {
    if !messages.is_empty() {
        Some(messages)
    } else {
        None
    }
}

/// Create a value for `execution_duration` from start and end timestamps
fn execution_duration(started: &Timestamp, ended: &Timestamp) -> Option<Duration> {
    Some(
        ended
            .duration(started)
            .expect("should use compatible timestamps"),
    )
}

impl Executable for CodeChunk {
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        // Execute code (if it is not empty) in kernels
        let code = self.code.trim();
        if !code.is_empty() {
            let started = Timestamp::now();
            let (outputs, messages) = executor
                .kernels
                .execute(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        vec![],
                        vec![error_to_message("While executing code", error)],
                    )
                });
            let ended = Timestamp::now();

            self.outputs = if !outputs.is_empty() {
                Some(outputs)
            } else {
                None
            };

            self.options.execution_status = execution_status(&messages);
            self.options.execution_messages = execution_messages(messages);
            self.options.execution_duration = execution_duration(&started, &ended);
            self.options.execution_ended = Some(ended);
            self.options.execution_count.get_or_insert(0).add_assign(1);
        } else {
            self.options.execution_status = Some(ExecutionStatus::Empty);
            self.options.execution_messages = None;
            self.options.execution_ended = None;
            self.options.execution_duration = None;
        }

        WalkControl::Break
    }
}

impl Executable for CodeExpression {
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        // Evaluate code (if it is not empty) in kernels
        let code = self.code.trim();
        if !code.is_empty() {
            let started = Timestamp::now();
            let (output, messages) = executor
                .kernels
                .evaluate(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_message("While evaluating expression", error)],
                    )
                });
            let ended = Timestamp::now();

            self.output = Some(Box::new(output));

            self.options.execution_status = execution_status(&messages);
            self.options.execution_messages = execution_messages(messages);
            self.options.execution_duration = execution_duration(&started, &ended);
            self.options.execution_ended = Some(ended);
            self.options.execution_count.get_or_insert(0).add_assign(1);
        } else {
            self.options.execution_status = Some(ExecutionStatus::Empty);
            self.options.execution_messages = None;
            self.options.execution_ended = None;
            self.options.execution_duration = None;
        }

        WalkControl::Break
    }
}

impl Executable for ForBlock {
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
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
                        (0..(int as u64)).map(Node::UnsignedInteger).collect_vec()
                    } else {
                        vec![]
                    }
                }
                Node::UnsignedInteger(uint) => {
                    if uint > 0 {
                        (0..uint).map(Node::UnsignedInteger).collect_vec()
                    } else {
                        vec![]
                    }
                }
                Node::Number(num) => {
                    if num > 0. {
                        (0..(num as u64)).map(Node::UnsignedInteger).collect_vec()
                    } else {
                        vec![]
                    }
                }
                Node::String(string) => string
                    .chars()
                    .map(|char| Node::String(char.to_string()))
                    .collect_vec(),
                Node::Array(array) => array.iter().map(|item| item.clone().into()).collect_vec(),
                Node::Object(object) => object
                    .iter()
                    .map(|(key, value)| {
                        Node::Array(Array(vec![Primitive::String(key.clone()), value.clone()]))
                    })
                    .collect_vec(),
                _ => vec![],
            };

            // Iterate over iterable, executing the content each time and adding to `iterations` field
            for node in &iterator {
                // Set the loop's variable
                if let Err(error) = executor.kernels.set(&self.variable, &node).await {
                    messages.push(ExecutionMessage::new(
                        MessageLevel::Error,
                        error.to_string(),
                    ));
                };

                // Clone the content, execute it and add it as an iteration
                let mut content = self.content.clone();
                if let Err(error) = content.walk_async(executor).await {
                    messages.push(error_to_message("While executing otherwise", error));
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
            self.options.execution_messages = execution_messages(messages);
            self.options.execution_duration = execution_duration(&started, &ended);
            self.options.execution_ended = Some(ended);
            self.options.execution_count.get_or_insert(0).add_assign(1);
        } else {
            self.options.execution_status = Some(ExecutionStatus::Empty);
            self.options.execution_messages = None;
            self.options.execution_ended = None;
            self.options.execution_duration = None;
        }

        WalkControl::Break
    }
}

impl Executable for IfBlock {
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        if !self.clauses.is_empty() {
            let started = Timestamp::now();

            // Explicitly re-set all clauses to inactive so it is possible to shortcut
            // evaluation by breaking on the first truthy clause
            for clause in self.clauses.iter_mut() {
                clause.is_active = Some(false);
            }

            // Iterate over clauses breaking on the first that is active
            for clause in self.clauses.iter_mut() {
                clause.execute(executor).await;

                if clause.is_active.unwrap_or_default() {
                    break;
                }
            }

            let ended = Timestamp::now();

            self.options.execution_status = Some(ExecutionStatus::Succeeded);
            self.options.execution_duration = execution_duration(&started, &ended);
            self.options.execution_ended = Some(ended);
            self.options.execution_count.get_or_insert(0).add_assign(1);
        } else {
            self.options.execution_status = Some(ExecutionStatus::Empty);
            self.options.execution_messages = None;
            self.options.execution_ended = None;
            self.options.execution_duration = None;
        }

        WalkControl::Break
    }
}

impl Executable for IfBlockClause {
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        let mut messages = Vec::new();
        let started = Timestamp::now();

        let code = self.code.trim();
        if !code.is_empty() {
            // Evaluate code in kernels
            let (output, mut code_messages) = executor
                .kernels
                .evaluate(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_message("While evaluating clause", error)],
                    )
                });
            messages.append(&mut code_messages);

            // Determine truthy-ness of the code's output value
            let truthy = match &output {
                Node::Null(..) => false,
                Node::Boolean(bool) => *bool,
                Node::Integer(int) => *int > 0,
                Node::UnsignedInteger(uint) => *uint > 0,
                Node::Number(number) => *number > 0.,
                Node::String(string) => !string.is_empty(),
                Node::Array(array) => !array.is_empty(),
                Node::Object(object) => !object.is_empty(),
                _ => true,
            };

            // Execute nodes in content if truthy
            if truthy {
                if let Err(error) = self.content.walk_async(executor).await {
                    messages.push(error_to_message("While executing content", error))
                };
            }

            self.is_active = truthy.then_some(true).or(Some(false));
        } else {
            // If code is empty then this is an `else` clause so will always
            // be active (if the `IfBlock` got this far in its execution)
            if let Err(error) = self.content.walk_async(executor).await {
                messages.push(error_to_message("While executing content", error))
            };

            self.is_active = Some(true);
        }

        let ended = Timestamp::now();

        self.options.execution_status = execution_status(&messages);
        self.options.execution_messages = execution_messages(messages);
        self.options.execution_duration = execution_duration(&started, &ended);
        self.options.execution_ended = Some(ended);
        self.options.execution_count.get_or_insert(0).add_assign(1);

        WalkControl::Break
    }
}
