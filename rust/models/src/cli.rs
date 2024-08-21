use cli_utils::table::{self, Attribute, Cell, Color};
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
    },
    ModelAvailability, ModelType,
};

/// Manage generative models
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

/// List the models available
#[derive(Debug, Args)]
struct List;

impl List {
    async fn run(self) -> Result<()> {
        let mut table = table::new();
        table.set_header([
            "Id", "Type", "Provider", "Name", "Version", "Inputs", "Outputs",
        ]);

        for model in super::list().await {
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

            table.add_row([
                Cell::new(model.id()).add_attribute(Attribute::Bold),
                match model.r#type() {
                    ModelType::Builtin => Cell::new("builtin").fg(Color::Blue),
                    ModelType::Local => Cell::new("local").fg(Color::Yellow),
                    ModelType::Router => Cell::new("router").fg(Color::Green),
                    ModelType::Proxy => Cell::new("proxy").fg(Color::Cyan),
                    ModelType::Remote => Cell::new("remote").fg(Color::Magenta),
                    ModelType::Plugin(name) => {
                        Cell::new(format!("plugin \"{name}\"")).fg(Color::DarkCyan)
                    }
                },
                Cell::new(model.provider()),
                Cell::new(model.name()),
                match availability {
                    Available => Cell::new(model.version()),
                    _ => match availability {
                        Available => Cell::new(availability).fg(Color::Green),
                        Disabled => Cell::new(availability).fg(Color::DarkBlue),
                        Installable => Cell::new(availability).fg(Color::Cyan),
                        Unavailable => Cell::new(availability).fg(Color::Grey),
                    },
                },
                Cell::new(inputs),
                Cell::new(outputs),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}
