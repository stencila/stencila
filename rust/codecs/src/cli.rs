use cli_utils::table::{self, Attribute, Cell};
use codec::common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
    itertools::Itertools,
};

/// Manage format conversion codecs
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

/// List the codecs available
#[derive(Debug, Args)]
struct List;

impl List {
    async fn run(self) -> Result<()> {
        let mut table = table::new();
        table.set_header(["Name", "From", "To"]);

        for codec in super::list() {
            let from = codec
                .supports_from_formats()
                .keys()
                .map(|format| format.to_string())
                .join(", ");
            let to = codec
                .supports_to_formats()
                .keys()
                .map(|format| format.to_string())
                .join(", ");

            table.add_row([
                Cell::new(codec.name()).add_attribute(Attribute::Bold),
                Cell::new(from),
                Cell::new(to),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}
