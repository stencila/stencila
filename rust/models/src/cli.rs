use cli_utils::{
    color_print::cstr,
    tabulated::{Attribute, Cell, CellAlignment, Color, Tabulated},
    AsFormat, Code, ToStdout,
};
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
        serde_yaml,
    },
    format::Format,
    schema::{InstructionMessage, ModelParameters},
    ModelAvailability, ModelSpecification, ModelTask, ModelType,
};

use crate::select;

/// Manage generative models
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># List all available models</dim>
  <blue>></blue> stencila models

  <dim># List models as JSON</dim>
  <blue>></blue> stencila models list --as json

  <dim># Test a model with a prompt</dim>
  <blue>></blue> stencila models run \"Explain photosynthesis\"

  <dim># Test a specific model</dim>
  <blue>></blue> stencila models run \"Write a poem\" --model gpt-4o

  <dim># Dry run to see task construction</dim>
  <blue>></blue> stencila models run \"Hello\" --dry-run

<bold><blue>Model Types</blue></bold>
  • <blue>builtin</blue> - Built into Stencila
  • <blue>local</blue> - Running locally (e.g. Ollama)
  • <blue>remote</blue> - Cloud-based APIs
  • <blue>router</blue> - Routes to other models
  • <blue>plugin</blue> - Provided by plugins
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

/// List the models available
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># List all models in table format</dim>
  <blue>></blue> stencila models list

  <dim># Output models as YAML</dim>
  <blue>></blue> stencila models list --as yaml
"
);

impl List {
    async fn run(self) -> Result<()> {
        let list = super::list().await;

        if let Some(format) = self.r#as {
            let list = list
                .into_iter()
                .map(|model| ModelSpecification::from(model.as_ref()))
                .collect_vec();

            Code::new_from(format.into(), &list)?.to_stdout();

            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header([
            "Id",
            "Type",
            "Availability",
            "Provider",
            "Name",
            "Version",
            "Quality",
            "Cost",
            "Speed",
        ]);

        for model in list {
            use ModelAvailability::*;
            use ModelType::*;

            let availability = model.availability();

            let no_score = || (String::new(), Color::Reset);
            let score = |score: u32| {
                (
                    score.to_string(),
                    match score {
                        0..=20 => Color::Red,
                        21..=40 => Color::Magenta,
                        41..=60 => Color::Yellow,
                        61..=80 => Color::Cyan,
                        81..=100 => Color::Green,
                        _ => Color::White,
                    },
                )
            };
            let quality = model.quality_score().map_or_else(no_score, score);
            let cost = model.cost_score().map_or_else(no_score, score);
            let speed = model.speed_score().map_or_else(no_score, score);
            let right = CellAlignment::Right;

            table.add_row([
                Cell::new(model.id()).add_attribute(Attribute::Bold),
                match model.r#type() {
                    Builtin => Cell::new("builtin").fg(Color::DarkBlue),
                    Local => Cell::new("local").fg(Color::Blue),
                    Router => Cell::new("router").fg(Color::Green),
                    Proxied => Cell::new("proxied").fg(Color::Cyan),
                    Remote => Cell::new("remote").fg(Color::Magenta),
                    Plugin(name) => Cell::new(format!("plugin \"{name}\"")).fg(Color::DarkCyan),
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
                Cell::new(quality.0).fg(quality.1).set_alignment(right),
                Cell::new(cost.0).fg(cost.1).set_alignment(right),
                Cell::new(speed.0).fg(speed.1).set_alignment(right),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Run a model task
///
/// Mainly intended for testing of model selection and routing.
/// Displays the task sent to the model and the generated output
/// returned from it.
#[derive(Debug, Args)]
#[clap(alias = "execute")]
#[command(after_long_help = RUN_AFTER_LONG_HELP)]
struct Run {
    prompt: String,

    /// The id pattern to specify the model to use
    #[arg(long, short)]
    model: Option<String>,

    /// Perform a dry run
    #[arg(long)]
    dry_run: bool,
}

pub static RUN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># Run with automatic model selection</dim>
  <blue>></blue> stencila models run \"Explain quantum computing\"

  <dim># Run with a specific model</dim>
  <blue>></blue> stencila models run \"Write a haiku\" --model gpt-3.5-turbo

  <dim># Run a dry run to see task construction</dim>
  <blue>></blue> stencila models run \"Hello world\" --dry-run

  <dim># Use the execute alias</dim>
  <blue>></blue> stencila models execute \"Summarize this text\"

<bold><blue>Note</blue></bold>
  This command is primarily for testing model routing and selection.
"
);

impl Run {
    async fn run(self) -> Result<()> {
        let message = InstructionMessage::from(self.prompt);

        let model = if self.model.is_some() {
            Some(ModelParameters {
                model_ids: self.model.map(|model| vec![model]),
                ..Default::default()
            })
        } else {
            None
        };

        let task = ModelTask {
            messages: vec![message],
            model_parameters: model,
            dry_run: self.dry_run,
            ..Default::default()
        };

        Code::new(Format::Markdown, "# Constructed task\n").to_stdout();
        Code::new(Format::Yaml, &serde_yaml::to_string(&task)?).to_stdout();

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

        let output = model.perform_task(&task).await?;

        Code::new(Format::Markdown, "# Generated output\n").to_stdout();
        Code::new(Format::Yaml, &serde_yaml::to_string(&output)?).to_stdout();

        Ok(())
    }
}
