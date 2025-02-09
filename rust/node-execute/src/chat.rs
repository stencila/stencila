use common::{
    futures::{stream::FuturesUnordered, StreamExt},
    itertools::Itertools,
    serde_json::json,
    tokio,
};
use models::ModelTask;
use node_diagnostics::{diagnostics_gte, DiagnosticLevel};
use schema::{
    authorship,
    shortcuts::{ci, h3, p, t},
    Author, AuthorRole, AuthorRoleName, Block, Chat, ChatMessage, ChatMessageGroup,
    ChatMessageOptions, ExecutionBounds, InstructionMessage, InstructionType, MessagePart,
    MessageRole, ModelParameters, Patch, PatchPath, SoftwareApplication,
};

use crate::{
    code_utils::apply_execution_bounds,
    interrupt_impl,
    model_utils::{
        blocks_to_message_part, blocks_to_system_message, file_to_message_part,
        model_task_to_blocks_and_authors,
    },
    prelude::*,
    ExecuteOptions,
};

impl Executable for Chat {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Chat {node_id}");

        let mut ops = Vec::new();

        // If the prompt does not yet have a target then default to a general discussion prompt
        // if not embedded in a document, otherwise a document focussed prompt. This is a fallback and
        // it is better to set these at creation, if possible
        if self.prompt.target.is_none()
            && matches!(
                self.prompt.instruction_type,
                None | Some(InstructionType::Discuss)
            )
        {
            let target = if self.is_temporary.is_some() {
                "stencila/discuss/document"
            } else {
                "stencila/discuss/anything"
            }
            .to_string();

            self.prompt.target = Some(target.clone());
            ops.push((
                PatchPath::from([NodeProperty::Prompt, NodeProperty::Target]),
                PatchOp::Set(PatchValue::String(target)),
            ));
        }

        // Apply configuration settings to model parameters
        if let Some(config) = executor
            .config
            .as_ref()
            .and_then(|config| config.models.as_ref())
        {
            if self.model_parameters.execute_content.is_none() {
                self.model_parameters.execute_content = config.execute_content;
                ops.push((
                    PatchPath::from([NodeProperty::ModelParameters, NodeProperty::ExecuteContent]),
                    PatchOp::Set(PatchValue::Json(json!(config.execute_content))),
                ));
            }
            if self.model_parameters.execution_bounds.is_none() {
                self.model_parameters.execution_bounds = config.execution_bounds;
                ops.push((
                    PatchPath::from([NodeProperty::ModelParameters, NodeProperty::ExecutionBounds]),
                    PatchOp::Set(PatchValue::Json(json!(config.execution_bounds))),
                ));
            }
            if self.model_parameters.maximum_retries.is_none() {
                let num = config.maximum_retries.map(|num| num as u64);
                self.model_parameters.maximum_retries = num;
                ops.push((
                    PatchPath::from([NodeProperty::ModelParameters, NodeProperty::MaximumRetries]),
                    PatchOp::Set(PatchValue::Json(json!(num))),
                ));
            }
        }

        if !ops.is_empty() {
            executor.send_patch(Patch {
                node_id: Some(node_id),
                ops,
                ..Default::default()
            });
        }

        // Call `prompt.compile` directly because a `PromptBlock` that
        // is not a `Block::PromptBlock` variant is not walked over
        self.prompt.compile(executor).await;

        // Continue walk to compile other properties
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing Chat {node_id}");

        let ExecuteOptions { force_all, .. } = executor.execute_options.clone().unwrap_or_default();

        // Check if this chat is to be executed:
        // - force_all is on
        // - no node ids and this is not an embedded chat (`is_temporary` is None)
        // - node id is this chat
        let is_pending = force_all
            || (executor.node_ids.is_none() && self.is_temporary.is_none())
            || executor.node_ids.iter().flatten().any(|id| id == &node_id);

        // If not to be executed, then return early and continue walking document
        // to prepare nodes in the chat's `content`
        if !is_pending {
            return WalkControl::Continue;
        }

        // Set execution status
        self.options.execution_status = Some(ExecutionStatus::Pending);
        executor.patch(
            &node_id,
            [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
        );

        // Do not continue to prepare nodes in `content` because the
        // chat itself is being executed
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            // Chat itself not marked as pending so continue walk
            // to execute nodes in `content`

            if let Err(error) = self.content.walk_async(executor).await {
                tracing::error!("While executing chat content: {error}")
            };

            // Previously, rather than do the above walk, this returned `WalkControl::Continue`.
            // However, for some unresolved reason, that approach did not execute code chunks
            // nested within temporary chats (it worked for standalone-chats only). The
            // approach works for both types of chats
            return WalkControl::Break;
        }

        tracing::debug!("Executing Chat {node_id}");

        // Set status to running and clear execution messages
        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let started = Timestamp::now();

        let mut instruction_messages: Vec<InstructionMessage> = Vec::new();

        // Execute the prompt and render to a system message
        self.prompt.execute(executor).await;
        if let Some(content) = &self.prompt.content {
            instruction_messages.push(blocks_to_system_message(content));
        }

        // If there are no messages yet, and the prompt block contains a query
        // then construct the first message from it
        if let (true, Some(query)) = (self.content.is_empty(), &self.prompt.query) {
            let mut parts = Vec::new();

            if let Some(instruction_type) = &self.prompt.instruction_type {
                parts.push(instruction_type.to_string());
            }

            if let Some(relative_position) = &self.prompt.relative_position {
                parts.push(relative_position.to_string().to_lowercase());
            }

            if let Some(node_type) = &self.prompt.node_types.iter().flatten().next() {
                parts.push(node_type.to_string());
            }

            parts.push(query.to_string());

            let text = parts.join(" ");

            let message = Block::ChatMessage(ChatMessage {
                role: MessageRole::User,
                content: vec![p([t(text)])],
                ..Default::default()
            });

            self.content.push(message.clone());
            executor.patch(&node_id, [push(NodeProperty::Content, message)])
        }

        // Append the existing messages in this chat
        instruction_messages.append(
            &mut self
                .content
                .iter()
                .filter_map(|block| match block {
                    Block::ChatMessage(msg) => msg_to_instr_msg(msg),
                    Block::ChatMessageGroup(group) => group_to_instr_msg(group),
                    _ => None,
                })
                .collect(),
        );

        // Create an author role for the author (if any) of the last user message
        let user_author_role = self.content.iter().rev().find_map(|message| match message {
            Block::ChatMessage(ChatMessage {
                role: MessageRole::User,
                options,
                ..
            }) => options.author.as_ref().map(|author| AuthorRole {
                last_modified: Some(Timestamp::now()),
                ..author.clone().into_author_role(AuthorRoleName::Instructor)
            }),
            _ => None,
        });

        // Add a new model message, or message group, to the chat (with no content)
        // so the user can see it as running

        let model_ids = match &self.model_parameters.model_ids {
            Some(ids) => ids.clone(),
            // If no model ids specified, use the first available model
            None => models::list()
                .await
                .into_iter()
                .find(|model| model.is_available())
                .map(|model| vec![model.id()])
                .unwrap_or_else(|| vec!["stencila/router".to_string()]),
        };
        let replicates = self.model_parameters.replicates.unwrap_or(1) as usize;

        let model_ids = model_ids
            .iter()
            .flat_map(|x| vec![x; replicates])
            .cloned()
            .collect_vec();
        let models = models::list().await;
        let mut chat_messages = model_ids
            .iter()
            .map(|model_id| {
                let model = models.iter().find(|model| &model.id() == model_id);

                let author = model
                    .map(|model| model.to_software_application())
                    .unwrap_or(SoftwareApplication {
                        id: Some(model_id.into()),
                        ..Default::default()
                    });

                ChatMessage {
                    role: MessageRole::Model,
                    options: Box::new(ChatMessageOptions {
                        author: Some(Author::SoftwareApplication(author)),
                        execution_status: Some(ExecutionStatus::Running),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            })
            .collect_vec();

        let message_ids = chat_messages
            .iter()
            .map(|message| message.node_id())
            .collect_vec();

        let block = if chat_messages.len() == 1 {
            Block::ChatMessage(chat_messages.swap_remove(0))
        } else {
            Block::ChatMessageGroup(ChatMessageGroup {
                messages: chat_messages,
                ..Default::default()
            })
        };
        executor.patch(&node_id, [push(NodeProperty::Content, block)]);

        // Create futures for each message
        let mut futures = FuturesUnordered::new();
        for (model_id, message_id) in model_ids.into_iter().zip(message_ids.into_iter()) {
            tracing::trace!("Creating task for {model_id}");

            let task = ModelTask::new(
                self.prompt.instruction_type.unwrap_or_default(),
                ModelParameters {
                    model_ids: Some(vec![model_id.clone()]),
                    ..*self.model_parameters.clone()
                },
                instruction_messages.clone(),
            );
            futures.push(async move {
                let started = Timestamp::now();
                let result = model_task_to_blocks_and_authors(task.clone()).await;
                let ended = Timestamp::now();
                (model_id, message_id, task, started, ended, result)
            })
        }

        // Wait for each future to complete and patch content
        let execute_content = self.model_parameters.execute_content.unwrap_or(false);
        let execution_bounds = self
            .model_parameters
            .execution_bounds
            .unwrap_or(ExecutionBounds::Fork);
        let maximum_retries = self.model_parameters.maximum_retries.unwrap_or(0);
        while let Some((model_id, message_id, mut task, started, ended, result)) =
            futures.next().await
        {
            tracing::trace!("Model message finished {message_id}");

            let (content, messages) = match result {
                Ok((mut content, mut authors)) => {
                    if let Some(role) = &user_author_role {
                        authors.push(role.clone());
                    }

                    // Apply model and user authorship to blocks
                    if let Err(error) = authorship(&mut content, authors.clone()) {
                        tracing::error!("While applying authorship to content: {error}");
                    }

                    // Apply execution bounds
                    apply_execution_bounds(&mut content, execution_bounds);

                    // Execute the content. Note that this is spawned as an async task so that
                    // the message can be updated with the unexecuted content first.
                    if execute_content {
                        let mut fork = executor.fork_for_all();
                        let mut content = content.clone();
                        let message_id = message_id.clone();
                        tokio::spawn(async move {
                            let mut retries = 0;
                            loop {
                                // Format, fix & lint the content before executing
                                if let Err(error) =
                                    fork.compile_lint(&mut content, true, true).await
                                {
                                    tracing::error!("While linting content: {error}");
                                };

                                // Collect linting diagnostics
                                let diags = diagnostics_gte(&content, DiagnosticLevel::Warning);

                                // Execute only if no linting warnings or errors
                                let diags = if diags.is_empty() {
                                    if let Err(error) = fork.prepare_execute(&mut content).await {
                                        tracing::error!("While executing content: {error}");
                                    }
                                    diagnostics_gte(&content, DiagnosticLevel::Warning)
                                } else {
                                    diags
                                };

                                // Stop retrying if no retries or diagnostics
                                if maximum_retries == 0 || diags.is_empty() {
                                    break;
                                }

                                // Extract the level and message of the first diagnostic
                                let (level, message) = diags
                                    .first()
                                    .map(|diag| {
                                        (
                                            diag.level().to_string().to_lowercase(),
                                            diag.message().to_string(),
                                        )
                                    })
                                    .unwrap_or_default();

                                // Stop if hit maximum number of retries
                                // Note that this check occurs after the execute to allow the final
                                // retry to be executed
                                if retries >= maximum_retries {
                                    // Let the user know we are giving up
                                    if retries > 0 {
                                        let message = p([
                                            t(format!(
                                                "Giving up after {retries} {} with {level}: ",
                                                if retries == 1 { "retry" } else { "retries" }
                                            )),
                                            ci(truncate(message, 100)),
                                        ]);
                                        fork.patch(
                                            &message_id,
                                            [append(NodeProperty::Content, vec![message])],
                                        );
                                    }

                                    break;
                                }
                                retries += 1;

                                // Add content indicating the retry and the reason for it
                                // and update message status indicating that updating message
                                let message = vec![
                                    h3([t(format!("Retry {retries} of {maximum_retries}"))]),
                                    p([t(format!("Trying again due to {level}: ")), ci(message)]),
                                ];
                                fork.patch(
                                    &message_id,
                                    [
                                        append(NodeProperty::Content, message),
                                        set(
                                            NodeProperty::ExecutionStatus,
                                            ExecutionStatus::Running,
                                        ),
                                    ],
                                );

                                // Add a new message to the task with the diagnostic/s
                                let diags = diags
                                    .into_iter()
                                    .filter_map(|diag| diag.to_string_pretty("", "", &None).ok())
                                    .join("\n");
                                task.messages.push(InstructionMessage::user(
                                    format!("There was an error, please try again:\n\n{diags}"),
                                    None,
                                ));

                                // Run model task again with diagnostic added
                                let mut new_content =
                                    match model_task_to_blocks_and_authors(task.clone()).await {
                                        Ok((blocks, ..)) => blocks,
                                        Err(error) => {
                                            // If there was an error retrying e.g. model unavailable then
                                            // give up.
                                            let message = p([
                                                t(format!(
                                                    "Giving up after {retries} {} due to error: ",
                                                    if retries == 1 { "retry" } else { "retries" }
                                                )),
                                                ci(truncate(error.to_string(), 200)),
                                            ]);
                                            let exec_message = error_to_execution_message(
                                                &format!("While running model `{model_id}`"),
                                                error,
                                            );
                                            fork.patch(
                                                &message_id,
                                                [
                                                    append(NodeProperty::Content, vec![message]),
                                                    push(
                                                        NodeProperty::ExecutionMessages,
                                                        exec_message,
                                                    ),
                                                    set(
                                                        NodeProperty::ExecutionStatus,
                                                        ExecutionStatus::Errors,
                                                    ),
                                                ],
                                            );
                                            break;
                                        }
                                    };

                                // Apply model and user authorship and execution bounds to new blocks
                                if let Err(error) = authorship(&mut new_content, authors.clone()) {
                                    tracing::error!(
                                        "While applying authorship to content: {error}"
                                    );
                                }
                                apply_execution_bounds(&mut new_content, execution_bounds);

                                // Reset the content so only the new blocks are executed
                                content = new_content.clone();

                                // Append the new content to the message
                                fork.patch(
                                    &message_id,
                                    [
                                        append(NodeProperty::Content, new_content),
                                        set(
                                            NodeProperty::ExecutionStatus,
                                            ExecutionStatus::Succeeded,
                                        ),
                                    ],
                                );
                            }
                        });
                    }

                    (content, None)
                }
                Err(error) => (
                    vec![],
                    Some(vec![error_to_execution_message(
                        &format!("While running model `{model_id}`"),
                        error,
                    )]),
                ),
            };

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);

            // Patch the message with its execution details and new content
            executor.patch(
                &message_id,
                [
                    set(NodeProperty::ExecutionStatus, status),
                    set(NodeProperty::ExecutionRequired, required),
                    set(NodeProperty::ExecutionMessages, messages),
                    set(NodeProperty::ExecutionDuration, duration),
                    set(NodeProperty::ExecutionEnded, ended),
                    append(NodeProperty::Content, content.clone()),
                ],
            );
        }

        let messages = None;
        let ended = Timestamp::now();

        let status = execution_status(&messages);
        let required = execution_required_status(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;

        // Patch the chat's execution details
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

        // Break walk because the chat has been updated
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting Chat {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`
        WalkControl::Continue
    }
}

fn msg_to_instr_msg(msg: &ChatMessage) -> Option<InstructionMessage> {
    // Begin parts with content of message converted to Markdown
    let mut parts: Vec<MessagePart> = blocks_to_message_part(&msg.content)
        .iter()
        .cloned()
        .collect();

    // Add any attached files
    let mut files = msg
        .options
        .files
        .iter()
        .flatten()
        .filter_map(file_to_message_part)
        .collect();
    parts.append(&mut files);

    // Some models do not like empty message parts, or no message parts so ensure that
    // does not happen.
    if parts.is_empty() {
        parts.push(MessagePart::Text("Hello".into()));
    }

    Some(InstructionMessage {
        role: Some(msg.role),
        parts,
        ..Default::default()
    })
}

fn group_to_instr_msg(group: &ChatMessageGroup) -> Option<InstructionMessage> {
    // Convert the selected message, defaulting to the first messages, into an
    // instruction message
    group
        .messages
        .iter()
        .find(|msg| msg.options.is_selected.unwrap_or_default())
        .or_else(|| group.messages.first())
        .and_then(msg_to_instr_msg)
}

fn truncate(message: String, chars: usize) -> String {
    if message.len() > chars {
        let mut message: String = message.chars().take(chars).collect();
        message.push('…');
        message
    } else {
        message
    }
}
