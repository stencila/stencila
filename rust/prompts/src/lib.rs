#![recursion_limit = "256"]

use std::{
    cmp::Ordering,
    io::Cursor,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use app::{get_app_dir, DirType};
use codec_markdown_trait::to_markdown;
use codecs::{DecodeOptions, Format};
use common::{
    derive_more::{Deref, DerefMut},
    eyre::{bail, eyre, OptionExt, Result},
    futures::future::{join_all, try_join_all},
    glob::glob,
    itertools::Itertools,
    regex::Regex,
    reqwest::Client,
    serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer},
    serde_json,
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
        SuggestionBlock, SuggestionStatus, Timestamp, UnsignedIntegerOrString, VideoObject,
    },
    ModelOutput, ModelOutputKind, ModelTask,
};

pub mod cli;

// Re-export
pub use prompt;
use semver::Version;
use version::stencila_version;

/// An instance of a prompt
///
/// A wrapper around an [`Prompt`] used to cache derived properties
/// such as regexes / embeddings
#[derive(Default, Clone, Deref, DerefMut, Deserialize)]
#[serde(default, crate = "common::serde")]
pub struct PromptInstance {
    #[deref]
    #[deref_mut]
    #[serde(flatten)]
    inner: Prompt,

    /// Path of the prompt
    ///
    /// Used to be able to open the prompt in editors.
    path: PathBuf,

    /// Home directory of the prompt
    ///
    /// Used mainly to resolve relative paths used for the source of
    /// `IncludeBlocks` used within instructions.
    #[serde(skip)]
    home: PathBuf,

    /// The `node_count` of the prompt parsed from a number or string into a [`RangeInclusive`]
    #[serde(skip)]
    node_count_range: Option<RangeInclusive<usize>>,

    /// Compiled regexes for the prompt's instruction regexes
    #[serde(skip)]
    instruction_regexes: Vec<Regex>,

    /// The generality of the prompt
    ///
    /// Based on the number of instruction types and node types the prompt supports.
    /// Used to rank prompts with higher specificity (i.e. lower generality) when
    /// inferring which prompts to use for chats and commands.                         
    #[serde(skip)]
    generality: usize,
}

/// Custom serialization to flatten and avoid unnecessarily serializing content of prompt
impl Serialize for PromptInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PromptInstance", 2)?;
        state.serialize_field("id", &self.inner.id)?;
        state.serialize_field("version", &self.inner.version)?;
        state.serialize_field("name", &self.inner.name)?;
        state.serialize_field("description", &self.inner.description)?;
        state.serialize_field("instructionTypes", &self.inner.instruction_types)?;
        state.serialize_field("instructionPatterns", &self.inner.instruction_patterns)?;
        state.serialize_field("nodeTypes", &self.inner.node_types)?;
        state.serialize_field("path", &self.path)?;
        state.end()
    }
}

impl PromptInstance {
    fn new(inner: Prompt, path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;

        let home = path
            .parent()
            .ok_or_eyre("prompt not in a dir")?
            .to_path_buf();

        let node_count_range = if let Some(node_count) = &inner.node_count {
            let range = match node_count {
                UnsignedIntegerOrString::UnsignedInteger(count) => {
                    let count = *count as usize;
                    count..=count
                }
                UnsignedIntegerOrString::String(range) => {
                    let mut parts = range.split(['+', '-']);
                    let lower = match parts.next() {
                        Some(lower) => usize::from_str(lower)?,
                        None => 0,
                    };
                    let upper = match parts.next() {
                        Some(upper) => usize::from_str(upper)?,
                        None => usize::MAX,
                    };
                    lower..=upper
                }
            };
            Some(range)
        } else {
            None
        };

        let instruction_regexes = inner
            .instruction_patterns
            .iter()
            .flatten()
            .map(|pattern| Regex::new(pattern))
            .try_collect()?;

        let generality = inner.instruction_types.len().min(1)
            * inner
                .node_types
                .as_ref()
                .map(|node_types| node_types.len())
                .unwrap_or(10);

        Ok(Self {
            inner,
            path,
            home,
            node_count_range,
            instruction_regexes,
            generality,
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
                Ordering::Equal => match a.generality.cmp(&b.generality) {
                    Ordering::Equal => a.id.cmp(&b.id),
                    cmp => cmp,
                },
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

/// Infer which prompt to use based on instruction type, node types, node count and/or hint
pub async fn infer(
    instruction_type: &Option<InstructionType>,
    node_types: &Option<Vec<String>>,
    hint: &Option<&str>,
) -> Option<PromptInstance> {
    let prompts = list().await.into_iter();

    // Filter the prompts by instruction type
    let prompts = prompts.filter(|prompt| match instruction_type {
        Some(instruction_type) => prompt.instruction_types.contains(instruction_type),
        None => true,
    });

    // Filter the prompts by node types: all node types must be in the prompts node types
    let prompts = prompts.filter(|prompt| match (node_types, &prompt.node_types) {
        (Some(node_types), Some(prompt_node_types)) => node_types
            .iter()
            .all(|node_type| prompt_node_types.contains(node_type)),
        _ => true,
    });

    // Filter the prompts by the number of nodes: the count must be in the prompts node count range
    let prompts = prompts.filter(|prompt| match (node_types, &prompt.node_count_range) {
        (Some(node_types), Some(range)) => range.contains(&node_types.len()),
        _ => true,
    });

    if let Some(hint) = hint {
        // If there is a hint, count the number of characters in the instruction message that
        // are matched by each of the patterns in each of the candidates
        let counts = prompts
            .map(|prompt| {
                let matches = prompt
                    .instruction_regexes
                    .iter()
                    .flat_map(|regex| regex.find_iter(hint).map(|found| found.len()))
                    .sum::<usize>();
                (prompt, matches)
            })
            .sorted_by(|(prompt_a, matches_a), (prompt_b, matches_b)| {
                // Sort by descending matches, and ascending generality
                match matches_a.cmp(matches_b).reverse() {
                    Ordering::Equal => prompt_a.generality.cmp(&prompt_b.generality),
                    order => order,
                }
            });

        // Get the prompt with the highest matches / lowest generality
        counts.map(|(prompt, ..)| prompt).next().take()
    } else {
        // Get the prompt with the lowest generality
        prompts
            .sorted_by(|prompt_a, prompt_b| prompt_a.generality.cmp(&prompt_b.generality))
            .next()
            .take()
    }
}

/// Attempt to shorten a prompt id, by removing "stencila/" and instruction type prefixes if possible
pub fn shorten(id: &str, instruction_type: &Option<InstructionType>) -> String {
    if let Some(rest) = id.strip_prefix("stencila/") {
        if let Some(instruction_type) = instruction_type {
            let prefix = [&instruction_type.to_string().to_lowercase(), "/"].concat();
            if let Some(name) = rest.strip_prefix(&prefix) {
                return name.into();
            } else {
                return rest.into();
            }
        }
        return rest.into();
    }

    id.into()
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
            prompts.push(PromptInstance::new(prompt, path)?)
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

/// Details recorded about the current builtin prompt library
#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct BuiltinDetails {
    version: Version,
    timestamp: u64,
    updated: bool,
}

impl BuiltinDetails {
    fn current() -> Self {
        Self {
            version: stencila_version(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs(),
            updated: false,
        }
    }
}

/// Initialize the builtin prompts directory
///
/// Saves the embedded prompts to disk
async fn initialize_builtin(force: bool) -> Result<PathBuf> {
    let dir = get_app_dir(DirType::Prompts, false)?.join("builtin");

    // Read in details, if any
    let details_path = dir.join("details.json");
    let details: Option<BuiltinDetails> = if details_path.exists() {
        read_to_string(&details_path)
            .await
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
    } else {
        None
    };

    // Determine if outdated
    let outdated = details
        .map(|details| details.version < stencila_version())
        .unwrap_or(true);

    // If the built-ins have not yet been initialized or updated then write them into
    // the directory. This needs to be done, rather than loading directly from memory
    // (as we used to do) so that inclusions work correctly.
    if force || outdated {
        tracing::debug!("Initializing builtin prompts");

        if dir.exists() {
            remove_dir_all(&dir).await?;
        }
        if !dir.exists() {
            create_dir_all(&dir).await?;
        }

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

        // Write details
        write(
            details_path,
            serde_json::to_string_pretty(&BuiltinDetails::current())?,
        )
        .await?;
    }

    Ok(dir)
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
                entry.unpack(dir.join(name))?;
            }
        }
    }

    // Write details
    let details_path = dir.join("details.json");
    write(
        details_path,
        serde_json::to_string_pretty(&BuiltinDetails {
            updated: true,
            ..BuiltinDetails::current()
        })?,
    )
    .await?;

    Ok(())
}

/// Select the most appropriate prompt for an instruction
pub async fn select(
    instruction_type: &InstructionType,
    message: &InstructionMessage,
    prompt: &Option<String>,
    _node_types: &Option<Vec<String>>,
) -> Result<PromptInstance> {
    let prompts = list().await;

    // If there is a prompt specified then get it
    if let Some(prompt) = prompt {
        return get(prompt, instruction_type).await;
    }

    // Filter the prompts to those that support the instruction type
    let prompts = prompts
        .into_iter()
        .filter(|prompt| prompt.instruction_types.contains(instruction_type));

    // Get the text of the message to match prompts against
    let message_text = message
        .parts
        .iter()
        .filter_map(|part| match part {
            MessagePart::Text(text) => Some(text.value.string.clone()),
            _ => None,
        })
        .join("");

    // Count the number of characters in the instruction message that are matched by
    // each of the patterns in each of the candidates
    let counts = prompts
        .filter_map(|prompt| {
            let matches = prompt
                .instruction_regexes
                .iter()
                .flat_map(|regex| regex.find_iter(&message_text).map(|found| found.len()))
                .sum::<usize>();
            // Let through those with any matches or that have no regexes (i.e. defaults)
            (matches > 0 || prompt.instruction_regexes.is_empty()).then_some((prompt, matches))
        })
        .sorted_by(|(.., a), (.., b)| a.cmp(b).reverse());

    // Get the prompt with the highest matches (or no regexes)
    let prompt = counts.map(|(prompt, ..)| prompt).next().take();

    prompt.ok_or_eyre("No prompts found for instruction")
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
    let message = instruction.message.clone();
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
    messages.push(InstructionMessage { parts, ..message });

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
                SuggestionStatus::Original => "This is the original.",
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
        *instruction.model_parameters.clone(),
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
