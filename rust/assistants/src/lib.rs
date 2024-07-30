#![recursion_limit = "256"]

use std::fs::read_dir;

use app::{get_app_dir, DirType};
use codec_markdown_trait::to_markdown;
use codecs::{DecodeOptions, EncodeOptions, Format};
use common::{eyre::eyre, tokio::fs::read_to_string};
use rust_embed::RustEmbed;

use model::{
    common::{
        eyre::{bail, Result},
        futures::future::join_all,
        itertools::Itertools,
        tracing,
    },
    schema::{
        authorship, shortcuts::p, Article, Assistant, AudioObject, Author, AuthorRole, ImageObject,
        Inline, InstructionBlock, InstructionMessage, InstructionType, Link, Node, SuggestionBlock,
        SuggestionStatus, Timestamp, VideoObject,
    },
    ModelOutput, ModelOutputKind, ModelTask,
};

pub mod cli;

/// Get a list of available assistants in descending preference rank
pub async fn list() -> Vec<Assistant> {
    let futures = (0..=3).map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("builtin", list_builtin().await),
            1 => ("local", list_local().await),
            _ => return vec![],
        };

        match result {
            Ok(list) => list,
            Err(error) => {
                tracing::error!("While listing {provider} assistants: {error}");
                vec![]
            }
        }
    });

    join_all(futures).await.into_iter().flatten().collect_vec()
}

/// Builtin assistants
///
/// During development these are loaded directly from the `assistants/builtin`
/// directory at the root of the repository but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../assistants/builtin"]
struct Builtin;

/// List the builtin assistants.
pub async fn list_builtin() -> Result<Vec<Assistant>> {
    let mut assistants = vec![];

    for (filename, content) in
        Builtin::iter().filter_map(|name| Builtin::get(&name).map(|file| (name, file.data)))
    {
        let (.., ext) = filename.split_once(".").unwrap_or_default();

        // TODO: Remove this when all assistants ported
        if ext != "smd" {
            continue;
        }

        let content = String::from_utf8_lossy(&content);

        let node = codecs::from_str(
            &content,
            Some(DecodeOptions {
                format: Some(Format::from_name(ext)),
                ..Default::default()
            }),
        )
        .await?;

        if let Node::Assistant(assistant) = node {
            assistants.push(assistant)
        } else {
            bail!(
                "Expected node to be an assistant, got `{}`",
                node.to_string()
            )
        }
    }

    Ok(assistants)
}

/// List any local assistants
async fn list_local() -> Result<Vec<Assistant>> {
    let mut assistants = vec![];

    let dir = get_app_dir(DirType::Assistants, false)?;

    tracing::debug!(
        "Attempting to read assistants from `{}` (if it exists)",
        dir.display()
    );

    if !dir.exists() {
        return Ok(assistants);
    }

    for entry in read_dir(dir)?.flatten() {
        let path = entry.path();
        let Some(ext) = path.extension() else {
            continue;
        };
        let content = read_to_string(&path).await?;

        let node = codecs::from_str(
            &content,
            Some(DecodeOptions {
                format: Some(Format::from_name(&ext.to_string_lossy())),
                ..Default::default()
            }),
        )
        .await?;

        if let Node::Assistant(assistant) = node {
            assistants.push(assistant)
        } else {
            bail!(
                "Expected node to be an assistant, got `{}`",
                node.to_string()
            )
        }
    }

    Ok(assistants)
}

/// Get the most appropriate assistant for an instruction
pub async fn find(
    assignee: &Option<String>,
    instruction_type: &InstructionType,
    node_types: &Option<Vec<String>>,
) -> Result<Assistant> {
    let assistants = list().await;

    // If there is an assignee then get it
    if let Some(assignee) = assignee {
        let name = if assignee.contains('/') {
            assignee.to_string()
        } else {
            ["stencila/", assignee].concat()
        };

        return assistants
            .into_iter()
            .find(|assistant| assistant.name == name)
            .ok_or_else(|| eyre!("Unable to find assistant with name `{assignee}`"));
    }

    // Filter the assistants to those that support the instruction and node types
    let mut assistants = assistants
        .into_iter()
        .filter(|assistant| assistant.instruction_types.contains(instruction_type))
        .collect_vec();

    if assistants.is_empty() {
        bail!("No assistants suitable to instruction")
    } else {
        Ok(assistants.swap_remove(0))
    }
}

/**
 * Render and assistant's content to Markdown use as a system prompt
 */
pub async fn render(assistant: Assistant) -> Result<String> {
    codecs::to_string(
        &Node::Article(Article {
            content: assistant.content,
            ..Default::default()
        }),
        Some(EncodeOptions {
            format: Some(Format::Markdown),
            render: Some(true),
            ..Default::default()
        }),
    )
    .await
}

/**
 * Execute an `InstructionBlock`
 */
pub async fn execute_instruction_block(
    instructor: AuthorRole,
    prompter: AuthorRole,
    system_prompt: &str,
    instruction: &InstructionBlock,
) -> Result<SuggestionBlock> {
    // Create a vector of messages from the system message and instruction messages
    let mut messages = vec![InstructionMessage::system(
        system_prompt,
        Some(vec![Author::AuthorRole(prompter.clone())]),
    )];

    if let Some(message) = &instruction.message {
        messages.push(message.clone())
    }

    for suggestion in instruction.suggestions.iter().flatten() {
        // Note: this encodes suggestion content to Markdown. Using the
        // format used by the particular assistant e.g. HTML may be more appropriate
        let md = to_markdown(&suggestion.content);
        messages.push(InstructionMessage::assistant(
            md,
            suggestion.authors.clone(),
        ));

        if let Some(feedback) = &suggestion.feedback {
            messages.push(InstructionMessage::user(feedback, None));
        }
    }

    // Create a model task
    let task = ModelTask::new(
        instruction.instruction_type.clone(),
        instruction.model.as_deref().cloned(),
        messages,
    );

    // Perform the task
    let started = Timestamp::now();
    let ModelOutput {
        mut authors,
        kind,
        format,
        content,
    } = models::perform_task(task).await?;
    let ended = Timestamp::now();

    let blocks = match kind {
        ModelOutputKind::Text => {
            // Decode the model output into blocks
            let node = codecs::from_str(
                &content,
                Some(DecodeOptions {
                    format: format
                        .is_unknown()
                        .then_some(Format::Markdown)
                        .or(Some(format)),
                    ..Default::default()
                }),
            )
            .await?;

            let Node::Article(Article { content, .. }) = node else {
                bail!("Expected content to be decoded to an article")
            };

            content
        }
        ModelOutputKind::Url => {
            let content_url = content;
            let media_type = Some(format.media_type());

            let node = if format.is_audio() {
                Inline::AudioObject(AudioObject {
                    content_url,
                    media_type,
                    ..Default::default()
                })
            } else if format.is_image() {
                Inline::ImageObject(ImageObject {
                    content_url,
                    media_type,
                    ..Default::default()
                })
            } else if format.is_video() {
                Inline::VideoObject(VideoObject {
                    content_url,
                    media_type,
                    ..Default::default()
                })
            } else {
                Inline::Link(Link {
                    target: content_url,
                    ..Default::default()
                })
            };

            vec![p([node])]
        }
    };

    // TODO: check that blocks are the correct type

    let mut suggestion = SuggestionBlock::new(blocks);

    // Record execution time for the suggestion
    let duration = ended
        .duration(&started)
        .expect("should use compatible timestamps");
    suggestion.execution_duration = Some(duration);
    suggestion.execution_ended = Some(ended);
    suggestion.suggestion_status = Some(SuggestionStatus::Proposed);

    // Apply authorship to the suggestion.
    authors.append(&mut vec![instructor, prompter]);
    authorship(&mut suggestion, authors)?;

    Ok(suggestion)
}
