use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use itertools::Itertools;
use serde_yaml;
use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    tabulated::{Attribute, Cell, Color, Tabulated},
};

use stencila_model::{
    ModelAvailability, ModelMessage, ModelSpecification, ModelTask, ModelType,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{File, ImageObject, MessagePart, ModelParameters, Text},
    stencila_schema_json::{JsonSchemaVariant, json_schema},
};

use crate::select;

/// Manage and interact with generative AI models
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available models</dim>
  <b>stencila models</b>

  <dim># List models as JSON</dim>
  <b>stencila models list</b> <c>--as</c> <g>json</g>

  <dim># Test a model with a prompt</dim>
  <b>stencila models run</b> <y>\"Explain photosynthesis\"</y>

  <dim># Test a specific model</dim>
  <b>stencila models run</b> <y>\"Write a poem\"</y> <c>--model</c> <g>gpt-4o</g>

  <dim># Run with multiple text arguments</dim>
  <b>stencila models run</b> <y>\"Analyze this:\"</y> <y>\"Some data here\"</y>

  <dim># Mix text and file arguments</dim>
  <b>stencila models run</b> <y>\"Summarize this file:\"</y> <g>document.txt</g>

  <dim># Multiple files and text</dim>
  <b>stencila models run</b> <y>\"Compare these files:\"</y> <g>file1.txt</g> <g>file2.txt</g>

  <dim># Dry run to see task construction</dim>
  <b>stencila models run</b> <y>\"Hello\"</y> <c>--dry-run</c>

<bold><b>Model Types</b></bold>
  • <g>builtin</g> - Built into Stencila
  • <g>local</g> - Running locally (e.g. Ollama)
  • <g>remote</g> - Cloud-based APIs
  • <g>router</g> - Routes to other models
  • <g>proxied</g> - Proxied through another service
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Run(Run),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Run(run) => run.run().await?,
        }

        Ok(())
    }
}

/// List available models with their status and capabilities
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Filter models by ID prefix (e.g., "ollama/gemma")
    prefix: Option<String>,

    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all models in table format</dim>
  <b>stencila models list</b>

  <dim># Filter models by ID prefix</dim>
  <b>stencila models list</b> <g>google/gemini</g>

  <dim># Output models as YAML</dim>
  <b>stencila models list</b> <c>--as</c> <g>yaml</g>
"
);

impl List {
    async fn run(self) -> Result<()> {
        let mut list = super::list().await;

        // Filter by prefix if provided
        if let Some(prefix) = &self.prefix {
            list.retain(|model| model.id().starts_with(prefix));
        }

        if let Some(format) = self.r#as {
            let list = list
                .into_iter()
                .map(|model| ModelSpecification::from(model.as_ref()))
                .collect_vec();

            Code::new_from(format.into(), &list)?.to_stdout();

            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Id", "Type", "Availability", "Provider", "Name", "Version"]);

        for model in list {
            use ModelAvailability::*;
            use ModelType::*;

            let availability = model.availability();

            table.add_row([
                Cell::new(model.id()).add_attribute(Attribute::Bold),
                match model.r#type() {
                    Builtin => Cell::new("builtin").fg(Color::DarkBlue),
                    Local => Cell::new("local").fg(Color::Blue),
                    Router => Cell::new("router").fg(Color::Green),
                    Proxied => Cell::new("proxied").fg(Color::Cyan),
                    Remote => Cell::new("remote").fg(Color::Magenta),
                },
                match availability {
                    Available => Cell::new(availability).fg(Color::Green),
                    Disabled => Cell::new(availability).fg(Color::DarkYellow),
                    RequiresKey => Cell::new(availability).fg(Color::Yellow),
                    Installable => Cell::new(availability).fg(Color::Cyan),
                    Unavailable => Cell::new(availability).fg(Color::Grey),
                },
                Cell::new(model.provider()),
                Cell::new(model.name()),
                Cell::new(model.version()),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Execute a task using a generative AI model
///
/// Primarily intended for testing model selection and routing. This command
/// constructs a task from the provided inputs, selects an appropriate model,
/// and displays both the constructed task and the generated output.
#[derive(Debug, Args)]
#[clap(alias = "execute")]
#[command(after_long_help = RUN_AFTER_LONG_HELP)]
struct Run {
    /// Text prompts and/or file paths (automatically detected)
    args: Vec<String>,

    /// Model id or pattern to select a specific model (e.g., "gpt-4o", "ollama/")
    #[arg(long, short)]
    model: Option<String>,

    /// Output format for generated content (json, markdown, yaml, etc.)
    #[arg(long, short)]
    format: Option<String>,

    /// JSON schema name for structured output validation (e.g., "math-block-tex")
    #[arg(long, short)]
    schema: Option<JsonSchemaVariant>,

    /// System message to set context or behavior for the model
    #[arg(long)]
    system: Option<String>,

    /// Write generated output to the specified file instead of stdout
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// Show task construction and model selection without executing
    #[arg(long)]
    dry_run: bool,
}

pub static RUN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Run with automatic model selection</dim>
  <b>stencila models run</b> <y>\"Explain quantum computing\"</y>

  <dim># Run with a specific model</dim>
  <b>stencila models run</b> <y>\"Write a haiku\"</y> <c>--model</c> <g>gpt-3.5-turbo</g>

  <dim># Multiple text arguments</dim>
  <b>stencila models run</b> <y>\"Analyze this data:\"</y> <y>\"temperature: 23°C, humidity: 65%\"</y>

  <dim># Mix text and file paths (files detected automatically)</dim>
  <b>stencila models run</b> <y>\"Summarize:\"</y> <g>report.txt</g>

  <dim># Multiple files and text</dim>
  <b>stencila models run</b> <y>\"Compare these:\"</y> <g>version1.py</g> <g>version2.py</g>

  <dim># Run a dry run to see task construction</dim>
  <b>stencila models run</b> <y>\"Hello world\"</y> <c>--dry-run</c>

  <dim># Use the execute alias</dim>
  <b>stencila models execute</b> <y>\"Summarize this text\"</y>

<bold><b>Note</b></bold>
  Arguments are automatically detected as file paths (if they exist) or treated as
  text content. Images are detected by file extension. This command is primarily
  for testing model routing and selection.
"
);

impl Run {
    async fn run(self) -> Result<()> {
        let mut messages: Vec<ModelMessage> = Vec::new();

        if let Some(system) = self.system {
            messages.push(ModelMessage::system(vec![MessagePart::Text(Text::from(
                system,
            ))]));
        }

        let mut parts = Vec::new();

        for arg in &self.args {
            let path = PathBuf::from(arg);
            if path.exists() && path.is_file() {
                let format = Format::from_path(&path);
                if format.is_image() {
                    parts.push(MessagePart::ImageObject(ImageObject::new(arg.to_string())));
                } else {
                    parts.push(MessagePart::File(File::read(&path)?));
                }
            } else {
                parts.push(MessagePart::Text(Text::from(arg)));
            }
        }

        messages.push(ModelMessage::user(parts));

        let schema = self.schema.map(json_schema);

        let format = Some(match self.format {
            Some(format) => {
                let mut format = Format::from_name(&format);
                if self.schema.is_some() && format != Format::Json {
                    tracing::warn!("Schema specified, so ignoring non-JSON format");
                    format = Format::Json;
                }
                format
            }
            None => {
                if self.schema.is_some() {
                    Format::Json
                } else {
                    Format::Markdown
                }
            }
        });

        let model = if self.model.is_some() {
            Some(ModelParameters {
                model_ids: self.model.map(|model| vec![model]),
                ..Default::default()
            })
        } else {
            None
        };

        let task = ModelTask {
            format,
            schema,
            messages,
            model_parameters: model,
            dry_run: self.dry_run,
            ..Default::default()
        };

        Code::new(Format::Markdown, "# Constructed task\n").to_stdout();

        // To avoid printing image and file contents iterate over the task
        // and replace `ImageObject.content_url` and `File.content` with "<redacted>"
        // in messages parts
        let mut redacted = task.clone();
        for message in &mut redacted.messages {
            for part in &mut message.parts {
                match part {
                    MessagePart::ImageObject(image) => {
                        if !image.content_url.is_empty() {
                            image.content_url = "<redacted>".to_string();
                        }
                    }
                    MessagePart::File(file) => {
                        if file.content.is_some() {
                            file.content = Some("<redacted>".to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        Code::new(Format::Yaml, &serde_yaml::to_string(&redacted)?).to_stdout();

        let model = select(&task).await?;

        Code::new(Format::Markdown, "# Selected model\n").to_stdout();
        Code::new(
            Format::Yaml,
            &format!(
                "id: {}\nprovider: {}\nname: {}\nversion: {}\ntype: {}",
                model.id(),
                model.provider(),
                model.name(),
                model.version(),
                model.r#type()
            ),
        )
        .to_stdout();

        let mut output = model.perform_task(&task).await?;

        if let Some(path) = self.output {
            write(&path, output.content)?;

            // Write any attachments to a sibling folder `xxxx.attachments`
            if !output.attachments.is_empty() {
                let attachments_dir = path.with_extension("attachments");
                create_dir_all(&attachments_dir)?;

                for attachment in output.attachments {
                    attachment.write(&attachments_dir.join(&attachment.name))?;
                }
            }
        } else {
            Code::new(Format::Markdown, "# Generated output\n").to_stdout();

            // As above, redact the content of any attachments
            for attachment in &mut output.attachments {
                if attachment.content.is_some() {
                    attachment.content = Some("<redacted>".to_string());
                }
            }

            Code::new(Format::Yaml, &serde_yaml::to_string(&output)?).to_stdout();
        }

        Ok(())
    }
}
