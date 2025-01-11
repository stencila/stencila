use common::{
    futures::{stream::FuturesUnordered, StreamExt},
    itertools::Itertools,
    tokio,
};
use models::ModelTask;
use schema::{
    authorship,
    shortcuts::{adm, p, t},
    AdmonitionType, Author, AuthorRole, AuthorRoleName, Block, Chat, ChatMessage, ChatMessageGroup,
    ChatMessageOptions, ExecutionBounds, InstructionMessage, MessagePart, MessageRole,
    ModelParameters, SoftwareApplication,
};

use crate::{
    interrupt_impl,
    model_utils::{
        blocks_to_message_part, blocks_to_system_message, file_to_message_part,
        model_task_to_blocks_and_authors,
    },
    prelude::*,
};

impl Executable for Chat {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Chat {node_id}");

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

        // Check if this chat is to be executed i.e. node ids contain this chat,
        // or there are no node ids and this is not an embedded chat (`is_temporary` is None),
        // or force_all is on.
        let is_pending = executor.options.force_all
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
            return WalkControl::Continue;
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
        // then use it as the first message
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

        // Construct a model task from all the messages in this chat
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
                self.prompt.instruction_type.clone().unwrap_or_default(),
                ModelParameters {
                    model_ids: Some(vec![model_id]),
                    ..*self.model_parameters.clone()
                },
                instruction_messages.clone(),
            );
            futures.push(async move {
                let started = Timestamp::now();
                let result = model_task_to_blocks_and_authors(task).await;
                let ended = Timestamp::now();
                (message_id, started, ended, result)
            })
        }

        // Wait for each future to complete and patch content
        let execution_bounds = self.execution_bounds.clone().unwrap_or_default();
        while let Some((message_id, started, ended, result)) = futures.next().await {
            tracing::trace!("Model message finished {message_id}");

            let (content, messages) = match result {
                Ok((mut content, mut authors)) => {
                    // Apply model and user authorship to blocks
                    if let Some(role) = &user_author_role {
                        authors.push(role.clone());
                    }
                    if let Err(error) = authorship(&mut content, authors) {
                        tracing::error!("While applying authorship to content: {error}");
                    }

                    // Execute if not skipped
                    if !matches!(execution_bounds, ExecutionBounds::Skip) {
                        let mut fork = executor.fork_for_all();
                        let mut content = content.clone();
                        let execute = async move {
                            if let Err(error) = fork.compile_prepare_execute(&mut content).await {
                                tracing::error!("While executing content: {error}");
                            }
                        };
                        tokio::spawn(execute);
                    }

                    (content, None)
                }
                Err(error) => (
                    vec![adm(
                        AdmonitionType::Failure,
                        None::<String>,
                        [p([t(error.to_string())])],
                    )],
                    Some(vec![error_to_execution_message(
                        "While running model",
                        error,
                    )]),
                ),
            };

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);

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

        // Patch the chat, including appending a new, empty user message
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
        role: Some(msg.role.clone()),
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
