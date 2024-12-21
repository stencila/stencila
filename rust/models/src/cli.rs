use cli_utils::{
    table::{self, Attribute, Cell, Color},
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
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Execute(Execute),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Execute(execute) => execute.run().await?,
        }

        Ok(())
    }
}

/// List the models available
#[derive(Default, Debug, Args)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

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

        let mut table = table::new();
        table.set_header([
            "Id",
            "Type",
            "Availability",
            "Provider",
            "Name",
            "Version",
            "I/O",
        ]);

        for model in list {
            use ModelAvailability::*;
            let availability = model.availability();

            let inputs = model
                .supported_inputs()
                .iter()
                .map(|input| input.to_string())
                .join(", ");

            let outputs = model
                .supported_outputs()
                .iter()
                .map(|output| output.to_string())
                .join(", ");

            let io = [&inputs, "/", &outputs].concat();

            table.add_row([
                Cell::new(model.id()).add_attribute(Attribute::Bold),
                match model.r#type() {
                    ModelType::Builtin => Cell::new("builtin").fg(Color::DarkBlue),
                    ModelType::Local => Cell::new("local").fg(Color::Blue),
                    ModelType::Router => Cell::new("router").fg(Color::Green),
                    ModelType::Proxied => Cell::new("proxied").fg(Color::Cyan),
                    ModelType::Remote => Cell::new("remote").fg(Color::Magenta),
                    ModelType::Plugin(name) => {
                        Cell::new(format!("plugin \"{name}\"")).fg(Color::DarkCyan)
                    }
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
                Cell::new(io),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}

/// Execute a model task
///
/// Mainly intended for testing of model selection and routing.
#[derive(Debug, Args)]
struct Execute {
    prompt: String,

    /// The id pattern to specify the model to use
    #[arg(long, short)]
    model: Option<String>,

    /// Perform a dry run
    #[arg(long)]
    dry_run: bool,
}

impl Execute {
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
