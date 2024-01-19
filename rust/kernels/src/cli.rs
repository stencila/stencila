use cli_utils::table::{self, Attribute, Cell, Color};
use kernel::{
    common::{
        clap::{self, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
    },
    KernelAvailability, KernelForking,
};

use crate::list;

/// Execution kernels
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// A command to perform with kernels
#[derive(Subcommand, Default)]
enum Command {
    /// List kernels available (default)
    #[default]
    List,
}

impl Cli {
    // Run the CLI
    pub async fn run(self) -> Result<()> {
        match self.command.unwrap_or_default() {
            Command::List => {
                let mut table = table::new();
                table.set_header(["Id", "Availability", "Languages", "Forking"]);
                for kernel in list().await {
                    use KernelAvailability::*;

                    let availability = kernel.availability();
                    let forking = kernel.supports_forking();

                    table.add_row([
                        Cell::new(kernel.id()).add_attribute(Attribute::Bold),
                        match availability {
                            Available => Cell::new(availability).fg(Color::Green),
                            Installable => Cell::new(availability).fg(Color::DarkYellow),
                            Unavailable => Cell::new(availability).fg(Color::Grey),
                        },
                        Cell::new(
                            kernel
                                .supports_languages()
                                .iter()
                                .map(|format| format.name())
                                .join(", "),
                        ),
                        match forking {
                            KernelForking::Yes => Cell::new(forking).fg(Color::Green),
                            KernelForking::No => Cell::new(forking).fg(Color::DarkGrey),
                        },
                    ]);
                }
                println!("{table}");
            }
        }

        Ok(())
    }
}
