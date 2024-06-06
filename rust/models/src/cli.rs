use cli_utils::table::{self, Attribute, Cell, CellAlignment, Color};
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
    },
    ModelAvailability, ModelType,
};

/// Manage models
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List {}.run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
        }

        Ok(())
    }
}

/// List the assistant available
#[derive(Debug, Args)]
struct List;

impl List {
    async fn run(self) -> Result<()> {
        let mut table = table::new();
        table.set_header([
            "Name",
            "Provider",
            "Version",
            "Context len.",
            "Inputs",
            "Outputs",
        ]);

        for assistant in super::list().await {
            use ModelAvailability::*;
            let availability = assistant.availability();

            let inputs = assistant
                .supported_inputs()
                .iter()
                .map(|input| input.to_string())
                .join(", ");

            let outputs = assistant
                .supported_outputs()
                .iter()
                .map(|output| output.to_string())
                .join(", ");

            table.add_row([
                Cell::new(assistant.name()).add_attribute(Attribute::Bold),
                match assistant.r#type() {
                    ModelType::Builtin => Cell::new("builtin").fg(Color::Green),
                    ModelType::Local => Cell::new("local").fg(Color::Cyan),
                    ModelType::Remote => Cell::new("remote").fg(Color::Magenta),
                    ModelType::Plugin(name) => {
                        Cell::new(format!("plugin \"{name}\"")).fg(Color::Blue)
                    }
                },
                match availability {
                    Available => Cell::new(assistant.version()),
                    _ => match availability {
                        Available => Cell::new(availability).fg(Color::Green),
                        Disabled => Cell::new(availability).fg(Color::DarkBlue),
                        Installable => Cell::new(availability).fg(Color::Cyan),
                        Unavailable => Cell::new(availability).fg(Color::Grey),
                    },
                },
                match assistant.context_length() {
                    0 => Cell::new(""),
                    _ => Cell::new(assistant.context_length()).set_alignment(CellAlignment::Right),
                },
                Cell::new(inputs),
                Cell::new(outputs),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}
