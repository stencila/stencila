use cli_utils::table::{self, Attribute, Cell, Color};
use codec::common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
};
use codec::status::Status;

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
        table.set_header(["Name", "Input formats", "Output formats", "Status"]);

        for codec in super::list() {
            let supports_input = codec.supports_from_path();
            let supports_output = codec.supports_to_path();
            let status = codec.status();
            table.add_row([
                Cell::new(codec.name()).add_attribute(Attribute::Bold),
                match supports_input {
                    true => Cell::new("Yes").fg(Color::Green),
                    false => Cell::new("No"),
                },
                match supports_output {
                    true => Cell::new("Yes").fg(Color::Green),
                    false => Cell::new("No"),
                },
                match status {
                    Status::Planned => Cell::new(status).fg(Color::Red),
                    Status::UnderDevelopment => Cell::new(status).fg(Color::Yellow),
                    Status::Stable => Cell::new(status).fg(Color::Green),
                    Status::Beta => Cell::new(status).fg(Color::DarkBlue),
                    Status::Alpha => Cell::new(status).fg(Color::Magenta),
                    Status::Experimental => Cell::new(status).fg(Color::DarkRed),
                },
            ]);
        }

        println!("{table}");

        Ok(())
    }
}
