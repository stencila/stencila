#![recursion_limit = "256"]

use std::{
    cmp::Ordering,
    io::Cursor,
    path::{Path, PathBuf},
};

use app::{get_app_dir, DirType};
use codec_markdown_trait::to_markdown;
use codecs::{DecodeOptions, EncodeOptions, Format};
use common::{
    chrono::Utc,
    derive_more::{Deref, DerefMut},
    eyre::{bail, eyre, OptionExt, Result},
    futures::future::{join_all, try_join_all},
    glob::glob,
    itertools::Itertools,
    regex::Regex,
    reqwest::Client,
    tar::Archive,
    tokio::fs::{create_dir_all, read_to_string, remove_dir_all, write},
    tracing,
};
use flate2::read::GzDecoder;
use images::ensure_http_or_data_uri;
use rust_embed::RustEmbed;

use model::{
    schema::{
        authorship, shortcuts::p, Article, AudioObject, Author, AuthorRole, Block,
        CompilationMessage, ExecutionMessage, ImageObject, Inline, InstructionBlock,
        InstructionMessage, InstructionType, Link, MessageLevel, MessagePart, Node, Prompt,
        SuggestionBlock, SuggestionStatus, Timestamp, VideoObject,
    },
    ModelOutput, ModelOutputKind, ModelTask,
};

pub mod cli;

// Re-export
pub use prompt;

/// An instance of a prompt
///
/// A wrapper around an [`Prompt`] used to cache derived properties
/// such as regexes / embeddings
#[derive(Clone, Deref, DerefMut)]
pub struct PromptInstance {
    #[deref]
    #[deref_mut]
    inner: Prompt,

    /// Home directory of the prompt
    ///
    /// Used mainly to resolve relative paths used for the source of
    /// `IncludeBlocks` used within instructions.
    home: PathBuf,

    /// Compiled regexes for the prompt's instruction regexes
    instruction_regexes: Vec<Regex>,
}

impl PromptInstance {
    fn new(inner: Prompt, home: PathBuf) -> Result<Self> {
        let instruction_regexes = inner
            .instruction_patterns
            .iter()
            .flatten()
            .map(|pattern| Regex::new(pattern))
            .try_collect()?;

        let home = home.canonicalize()?;

        Ok(Self {
            inner,
            home,
            instruction_regexes,
        })
    }

    // Get the home of the prompt
    pub fn home(&self) -> PathBuf {
        self.home.clone()
    }
}

/// Get a list of available prompts
///
/// Cached if not in debug mode
#[cfg_attr(not(debug_assertions), cached::proc_macro::cached(time = 3600))]
pub async fn list() -> Vec<PromptInstance> {
    let futures = (0..=3).map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("builtin", list_builtin().await),
            1 => ("local", list_local().await),
            _ => return vec![],
        };

        match result {
            Ok(list) => list,
            Err(error) => {
                tracing::error!("While listing {provider} prompts: {error}");
                vec![]
            }
        }
    });

    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .sorted_by(|a, b| {
            match a
                .instruction_types
                .first()
                .cmp(&b.instruction_types.first())
            {
                Ordering::Equal => a.id.cmp(&b.id),
                cmp => cmp,
            }
        })
        .collect_vec()
}

/// Get a prompt by id
pub async fn get(id: &str, instruction_type: &InstructionType) -> Result<PromptInstance> {
    let id = if id.contains('/') {
        id.to_string()
    } else {
        let instruction_type = [&instruction_type.to_string().to_lowercase(), "/"].concat();
        if !id.starts_with(&instruction_type) {
            ["stencila/", &instruction_type, id].concat()
        } else {
            ["stencila/", id].concat()
        }
    };

    list()
        .await
        .into_iter()
        .find(|prompt| prompt.id.as_ref() == Some(&id))
        .ok_or_else(|| eyre!("Unable to find prompt with id `{id}`"))
}

/// Builtin prompts
///
/// During development these are loaded directly from the `prompts`
/// directory at the root of the repository but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../prompts"]
struct Builtin;

/// List the builtin prompts.
pub async fn list_builtin() -> Result<Vec<PromptInstance>> {
    // If in debug mode, just use the prompts dir in the repo
    if cfg!(debug_assertions) {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../prompts");
        return list_dir(&dir).await;
    }

    let dir = initialize_builtin(false).await?;
    list_dir(&dir).await
}

/// List any local prompts
async fn list_local() -> Result<Vec<PromptInstance>> {
    let dir = get_app_dir(DirType::Prompts, false)?.join("local");

    if dir.exists() {
        list_dir(&dir).await
    } else {
        Ok(Vec::new())
    }
}

/// List prompts in a directory
///
/// Lists all files (including in subdirectories) with one of the supported formats
/// (currently only `.smd`) except "partials".
async fn list_dir(dir: &Path) -> Result<Vec<PromptInstance>> {
    tracing::trace!("Attempting to read prompts from `{}`", dir.display());

    let mut prompts = vec![];
    for path in glob(&format!("{}/**/*.smd", dir.display()))?.flatten() {
        if path.components().any(|c| c.as_os_str() == "partials") {
            continue;
        }

        let (Some(filename), Some(ext)) = (path.file_name(), path.extension()) else {
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

        if let Node::Prompt(prompt) = node {
            let home = path
                .parent()
                .ok_or_eyre("prompt not in a dir")?
                .to_path_buf();
            prompts.push(PromptInstance::new(prompt, home)?)
        } else {
            bail!(
                "Expected `{}` to be an `Prompt`, got a `{}`",
                filename.to_string_lossy(),
                node.to_string()
            )
        }
    }

    Ok(prompts)
}

/// Initialize the builtin prompts directory
///
/// Saves the embedded prompts to disk
async fn initialize_builtin(force: bool) -> Result<PathBuf> {
    let dir = get_app_dir(DirType::Prompts, false)?.join("builtin");
    if !dir.exists() {
        create_dir_all(&dir).await?;
    }

    let initialized_at = dir.join("initialized-at.txt");
    let updated_at = dir.join("updated-at.txt");

    // If the built-ins have not yet been initialized or updated then write them into
    // the directory. This needs to be done, rather than loading directly from memory
    // (as we used to do) so that inclusions work correctly.
    if force || (!initialized_at.exists() && !updated_at.exists()) {
        let futures = Builtin::iter()
            .filter_map(|name| Builtin::get(&name).map(|file| (name, file.data)))
            .map(|(filename, content)| {
                let dir = dir.clone();
                async move {
                    let path = dir.join(filename.to_string());
                    if let Some(parent) = path.parent() {
                        create_dir_all(parent).await?;
                    }
                    write(path, content).await
                }
            });
        try_join_all(futures).await?;

        // Write timestamp
        write(initialized_at, Utc::now().to_rfc3339()).await?;
    }

    Ok(dir)
}

/// Reset the builtin prompts directory
async fn reset_builtin() -> Result<()> {
    let dir = get_app_dir(DirType::Prompts, false)?.join("builtin");
    if dir.exists() {
        remove_dir_all(&dir).await?;
    }

    initialize_builtin(true).await?;

    Ok(())
}

/// Update the builtin prompts directory
///
/// So that users can get the latest version of prompts, without installing a new version of
/// the binary.Fetches a tarball of the prompts and extracts it into the `prompts/builtin` directory.
async fn update_builtin() -> Result<()> {
    let dir = get_app_dir(DirType::Prompts, false)?.join("builtin");
    if !dir.exists() {
        create_dir_all(&dir).await?;
    }

    tracing::info!("Updating builtin prompts");

    // Fetch the repo tar ball
    let tar_gz = Client::new()
        .get("https://github.com/stencila/stencila/archive/main.tar.gz")
        .send()
        .await?
        .bytes()
        .await?;
    let tar = GzDecoder::new(Cursor::new(tar_gz));

    // Extract just the builtin prompts
    let mut archive = Archive::new(tar);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if let Ok(relative_path) = path.strip_prefix("stencila-main") {
            if let Ok(name) = relative_path.strip_prefix("prompts/") {
                entry.unpack(&dir.join(name))?;
            }
        }
    }

    // Write timestamp
    let updated_at = dir.join("updated-at.txt");
    write(updated_at, Utc::now().to_rfc3339()).await?;

    Ok(())
}

/// Select the most appropriate prompt for an instruction
pub async fn select(
    instruction_type: &InstructionType,
    message: &Option<InstructionMessage>,
    prompt: &Option<String>,
    _node_types: &Option<Vec<String>>,
) -> Result<PromptInstance> {
    let prompts = list().await;

    // If there is a prompt then get it
    if let Some(prompt) = prompt {
        return get(prompt, instruction_type).await;
    }

    // Filter the prompts to those that support the instruction type
    let prompts = prompts
        .into_iter()
        .filter(|prompt| prompt.instruction_types.contains(instruction_type));

    // Get the text of the message to match prompts against
    let message_text = match message {
        Some(message) => message
            .parts
            .iter()
            .filter_map(|part| match part {
                MessagePart::Text(text) => Some(text.value.string.clone()),
                _ => None,
            })
            .join(""),
        None => String::new(),
    };

    // Count the number of characters in the instruction message that are matched by
    // each of the patterns in each of the candidates
    let prompt = prompts
        .map(|prompt| {
            let matches = prompt
                .instruction_regexes
                .iter()
                .flat_map(|regex| regex.find_iter(&message_text).map(|found| found.len()))
                .sum::<usize>();
            (prompt, matches)
        })
        .sorted_by(|(.., a), (.., b)| a.cmp(b).reverse())
        .map(|(prompt, ..)| prompt)
        .next()
        .take();

    prompt.ok_or_eyre("No prompts found for instruction")
}

/// Render and prompt's content to Markdown to use as a system prompt
pub async fn render(prompt: PromptInstance) -> Result<String> {
    codecs::to_string(
        &Node::Article(Article {
            content: prompt.content.clone(),
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

/// Execute an [`InstructionBlock`]
pub async fn execute_instruction_block(
    mut instructors: Vec<AuthorRole>,
    prompter: AuthorRole,
    system_prompt: &str,
    instruction: &InstructionBlock,
    dry_run: bool,
) -> Result<SuggestionBlock> {
    // Create a vector of messages beginning with the system message
    let mut messages = vec![InstructionMessage::system(
        system_prompt,
        Some(vec![Author::AuthorRole(prompter.clone())]),
    )];

    // Add a user message for the instruction
    if let Some(message) = instruction.message.clone() {
        let parts = message
            .parts
            .into_iter()
            .map(|part| {
                // Ensure that any images in the message are fully resolved
                Ok(match part {
                    MessagePart::ImageObject(image) => MessagePart::ImageObject(ImageObject {
                        content_url: ensure_http_or_data_uri(&image.content_url)?,
                        ..image
                    }),
                    _ => part,
                })
            })
            .collect::<Result<_>>()?;
        messages.push(InstructionMessage { parts, ..message })
    }

    // If the instruction type is `Fix` and the first block in the content
    // (usually there is only one!) has errors or warnings then add a message for those.
    fn comp_msgs(messages: &Option<Vec<CompilationMessage>>) -> Option<String> {
        let messages = messages
            .iter()
            .flatten()
            .filter_map(|message| {
                matches!(
                    message.level,
                    MessageLevel::Warning | MessageLevel::Error | MessageLevel::Exception
                )
                .then_some(message.formatted())
            })
            .join("\n\n");

        (!messages.is_empty()).then_some(messages)
    }
    fn exec_msgs(messages: &Option<Vec<ExecutionMessage>>) -> Option<String> {
        let messages = messages
            .iter()
            .flatten()
            .filter_map(|message| {
                matches!(
                    message.level,
                    MessageLevel::Warning | MessageLevel::Error | MessageLevel::Exception
                )
                .then_some(message.formatted())
            })
            .join("\n\n");

        (!messages.is_empty()).then_some(messages)
    }
    if instruction.instruction_type == InstructionType::Fix {
        if let Some(message) = instruction
            .content
            .iter()
            .flatten()
            .next()
            .and_then(|block| match block {
                Block::CodeChunk(node) => comp_msgs(&node.options.compilation_messages)
                    .or(exec_msgs(&node.options.execution_messages)),
                Block::MathBlock(node) => comp_msgs(&node.options.compilation_messages),
                _ => None,
            })
        {
            messages.push(InstructionMessage::user(message, None))
        }
    }

    // Add pairs of assistant/user messages for each suggestion
    for suggestion in instruction.suggestions.iter().flatten() {
        // Note: this encodes suggestion content to Markdown. Using the
        // format used by the particular prompt e.g. HTML may be more appropriate
        messages.push(InstructionMessage::assistant(
            to_markdown(&suggestion.content),
            suggestion.authors.clone(),
        ));

        // If there is feedback on the suggestion use that, otherwise generate feedback
        // to follow the suggestion.
        let feedback = if let Some(feedback) = &suggestion.feedback {
            feedback
        } else if let Some(status) = &suggestion.suggestion_status {
            match status {
                SuggestionStatus::Accepted => {
                    "This is suggestion is acceptable, but please try again."
                }
                SuggestionStatus::Rejected => {
                    "This is wrong or otherwise not acceptable, please try again."
                }
            }
        } else {
            "Please try again."
        };
        messages.push(InstructionMessage::user(feedback, None));
    }

    tracing::trace!("Model task messages:\n\n{messages:#?}");

    // Create a model task
    let mut task = ModelTask::new(
        instruction.instruction_type.clone(),
        instruction.model.as_deref().cloned(),
        messages,
    );
    task.dry_run = dry_run;

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

    // Apply authorship to the suggestion.
    authors.append(&mut instructors);
    authors.push(prompter);
    authorship(&mut suggestion, authors)?;

    Ok(suggestion)
}
