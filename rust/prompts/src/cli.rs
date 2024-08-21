use cli_utils::table::{self, Attribute, Cell};
use common::itertools::Itertools;
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
    },
    schema::{Prompt, StringOrNumber},
};

/// Manage prompts
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

/// List the prompts available
#[derive(Debug, Args)]
struct List;

impl List {
    async fn run(self) -> Result<()> {
        let mut table = table::new();
        table.set_header([
            "Id",
            "Name",
            "Version",
            "Instructions",
            "Node types",
            "Description",
        ]);

        for prompt in super::list().await {
            let Prompt {
                id,
                name,
                version,
                instruction_types,
                node_types,
                description,
                ..
            } = prompt.inner;

            let version = match version {
                StringOrNumber::String(version) => version,
                StringOrNumber::Number(version) => version.to_string(),
            };

            table.add_row([
                Cell::new(id.unwrap_or_default()).add_attribute(Attribute::Bold),
                Cell::new(name),
                Cell::new(version),
                Cell::new(
                    instruction_types
                        .iter()
                        .map(|typ| typ.to_string())
                        .join(", "),
                ),
                Cell::new(node_types.join(", ")),
                Cell::new(description.as_str()),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}
