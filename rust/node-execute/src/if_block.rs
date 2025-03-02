use node_contains::contains;
use schema::{CompilationDigest, IfBlock, IfBlockClause};

use crate::{interrupt_impl, prelude::*};

impl Executable for IfBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling IfBlock {node_id}");

        // The compilation digest is a digest of the digests each of the individual clauses.
        let mut clauses_digest = 0u64;
        for clause in self.clauses.iter_mut() {
            clause.compile(executor).await;

            if let Some(digest) = &clause.options.compilation_digest {
                add_to_digest(
                    &mut clauses_digest,
                    &digest
                        .semantic_digest
                        .unwrap_or(digest.state_digest)
                        .to_be_bytes(),
                );
            }
        }

        let compilation_digest = CompilationDigest::new(clauses_digest);
        let execution_required =
            execution_required_digests(&self.options.execution_digest, &compilation_digest);

        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, compilation_digest),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing IfBlock {node_id}");

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            self.options.execution_status = Some(status);
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Continue walk so that `content` of clauses is made pending if necessary.
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            tracing::trace!("Skipping IfBlock {node_id}");

            // Continue walk so that `content` of clauses is executed if necessary.
            return WalkControl::Continue;
        }

        tracing::debug!("Executing IfBlock {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let compilation_digest = self.options.compilation_digest.clone();

        if !self.clauses.is_empty() {
            let started = Timestamp::now();

            // Explicitly re-set all clauses to inactive and skipped, so it is possible to shortcut
            // evaluation by breaking on the first truthy clause. Because of the short-cutting a
            // patch needs to be sent here too.
            for clause in self.clauses.iter_mut() {
                clause.is_active = None;
                clause.options.execution_status = Some(ExecutionStatus::Skipped);

                executor.patch(
                    &clause.node_id(),
                    [
                        none(NodeProperty::IsActive),
                        set(NodeProperty::ExecutionStatus, ExecutionStatus::Skipped),
                    ],
                );
            }

            // Iterate over clauses breaking on the first that is active
            // and determine execution status based on highest status of executed clauses
            let mut status = ExecutionStatus::Succeeded;
            let last_index = self.clauses.len() - 1;
            for (index, clause) in self.clauses.iter_mut().enumerate() {
                executor.is_last = index == last_index;

                execute_if_block_clause(clause, executor).await;

                if let Some(clause_status) = &clause.options.execution_status {
                    if clause_status > &status {
                        status = *clause_status;
                    }
                }

                if clause.is_active.unwrap_or_default() {
                    break;
                }
            }

            let ended = Timestamp::now();

            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            executor.patch(
                &node_id,
                [
                    set(NodeProperty::ExecutionStatus, status),
                    set(NodeProperty::ExecutionRequired, required),
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
        tracing::debug!("Interrupting IfBlock {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue walk to interrupt `clauses` and any executable nodes within them
        WalkControl::Continue
    }
}

impl Executable for IfBlockClause {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling IfBlockClause {node_id}");

        // Get the programming language, falling back to using the executor's current language
        let lang = executor.programming_language(&self.programming_language);

        // Parse the code to determine if it or the language has changed since last time
        let info = parsers::parse(&self.code, &lang, &self.options.compilation_digest);

        // Add code to the linting context
        executor.linting_code(&node_id, &self.code.to_string(), &lang, info.changed.yes());

        // Note that, unlike a `ForBlock`, the `content` of the clause does not need to be part of
        // the compilation digest because it does not affect the result of execution (which is
        // just to determine if the clause `is_active`).

        let execution_required =
            execution_required_digests(&self.options.execution_digest, &info.compilation_digest);

        // Update `compilation_digest` on `self` so that parent `IfBlock`
        // can update its own `compilation_digest` based on all the clauses
        self.options.compilation_digest = Some(info.compilation_digest.clone());

        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, info.compilation_digest),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        // Continue walk to compile `content`
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing IfBlockClause {node_id} {:?}", self.is_active);

        // Do not change the execution status of this clause because not every clause
        // in the parent if block will be executed (because it breaks on the first truthy).
        // As such, setting status to `Pending` here could never be overwritten.

        // If the executor has node ids and this clause contains
        // any of those node ids, then continue the walk to set those as pending
        if let Some(node_ids) = &executor.node_ids {
            if contains(&self.content, node_ids.clone()).is_some() {
                return WalkControl::Continue;
            }
        }

        // Break walk to be consistent with behavior of `execute` method (see note there),
        // otherwise nodes can be left in `Pending` state.
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        // If the executor has node ids and this clause contains
        // any of those node ids, then continue the walk to execute those
        if let Some(node_ids) = &executor.node_ids {
            if contains(&self.content, node_ids.clone()).is_some() {
                return WalkControl::Continue;
            }
        }

        // Break walk because do not want to execute `content`, even if active because that should be
        // controlled by the parent `IfBlock` and is done in `execute_if_block_clause`
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting IfBlockClause {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue walk to interrupt executable nodes in `content`
        WalkControl::Continue
    }
}

#[tracing::instrument(skip_all)]
async fn execute_if_block_clause(clause: &mut IfBlockClause, executor: &mut Executor) {
    let node_id = clause.node_id();
    tracing::debug!("Executing IfBlockClause {node_id}");

    executor.patch(
        &node_id,
        [
            set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
            none(NodeProperty::ExecutionMessages),
        ],
    );

    let mut messages = Vec::new();
    let started = Timestamp::now();

    let is_empty = clause.code.trim().is_empty();
    let (is_active, mut status) = if !is_empty {
        // Get the programming language, falling back to using the executor's current language
        let lang = executor.programming_language(&clause.programming_language);

        // Evaluate code in kernels
        let (output, mut code_messages, ..) = executor
            .kernels
            .write()
            .await
            .evaluate(&clause.code, lang.as_deref())
            .await
            .unwrap_or_else(|error| {
                (
                    Node::Null(Null),
                    vec![error_to_execution_message("While evaluating clause", error)],
                    String::new(),
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

        // Execute nodes in `content` if truthy
        if truthy {
            tracing::trace!("Executing if clause content");
            // Compile, prepare and execute the content. Need to do prepare at least because
            // this is not done in `IfBlock::prepare` (that method intentionally breaks the walk)
            // Force re-execution of all nodes in content because they may have become stale since
            // last time they were executed.
            executor.force_all = true;
            if let Err(error) = executor.compile_prepare_execute(&mut clause.content).await {
                messages.push(error_to_execution_message(
                    "While executing if clause content",
                    error,
                ))
            };
            executor.force_all = false;
        }

        (truthy, ExecutionStatus::Running)
    } else if is_empty && executor.is_last {
        // If code is empty and this is the last clause then this is an `else` clause so will always
        // be active (if the `IfBlock` got this far in its execution)
        // As for other clauses force re-execution of nodes.
        tracing::trace!("Executing else clause content");
        executor.force_all = true;
        if let Err(error) = executor.compile_prepare_execute(&mut clause.content).await {
            messages.push(error_to_execution_message(
                "While executing else clause content",
                error,
            ))
        };
        executor.force_all = false;

        (true, ExecutionStatus::Running)
    } else {
        // Just skip if empty code and not last
        (false, ExecutionStatus::Empty)
    };

    let messages = (!messages.is_empty()).then_some(messages);

    let ended = Timestamp::now();

    if status != ExecutionStatus::Skipped {
        status = execution_status(&messages)
    }
    let required = execution_required_status(&status);
    let duration = execution_duration(&started, &ended);
    let count = clause.options.execution_count.unwrap_or_default() + 1;

    // Update `is_active` on `self` so that parent `IfBlock` can break
    // on first active clause
    clause.is_active = Some(is_active);

    // Update `execution_status` on `self` so that parent `IfBlock`
    // can update its status based on clauses
    clause.options.execution_status = Some(status);

    // Set other properties that may be using in rendering
    clause.options.execution_messages = messages.clone();

    executor.patch(
        &node_id,
        [
            set(NodeProperty::IsActive, is_active),
            set(NodeProperty::ExecutionStatus, status),
            set(NodeProperty::ExecutionRequired, required),
            set(NodeProperty::ExecutionMessages, messages),
            set(NodeProperty::ExecutionDuration, duration),
            set(NodeProperty::ExecutionEnded, ended),
            set(NodeProperty::ExecutionCount, count),
            set(
                NodeProperty::ExecutionDigest,
                clause.options.compilation_digest.clone(),
            ),
        ],
    );
}
