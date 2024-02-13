use cli_utils::table::{self, Attribute, Cell, Color};
use kernel::{
    common::{
        clap::{self, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
    },
    KernelAvailability, KernelForks, KernelInterrupt, KernelKill, KernelTerminate,
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
                table.set_header([
                    "Id",
                    "Availability",
                    "Languages",
                    "Fork",
                    "Interrupt",
                    "Terminate",
                    "Kill",
                ]);
                for kernel in list().await {
                    use KernelAvailability::*;

                    let availability = kernel.availability();
                    let langs = kernel
                        .supports_languages()
                        .iter()
                        .map(|format| format.name())
                        .join(", ");
                    let forks = kernel.supports_forks();
                    let interrupt = kernel.supports_interrupt();
                    let terminate = kernel.supports_terminate();
                    let kill = kernel.supports_kill();

                    table.add_row([
                        Cell::new(kernel.id()).add_attribute(Attribute::Bold),
                        match availability {
                            Available => Cell::new(availability).fg(Color::Green),
                            Installable => Cell::new(availability).fg(Color::DarkYellow),
                            Unavailable => Cell::new(availability).fg(Color::Grey),
                        },
                        Cell::new(langs),
                        match forks {
                            KernelForks::Yes => Cell::new(forks).fg(Color::Green),
                            KernelForks::No => Cell::new(forks).fg(Color::DarkGrey),
                        },
                        match interrupt {
                            KernelInterrupt::Yes => Cell::new(interrupt).fg(Color::Green),
                            KernelInterrupt::No => Cell::new(interrupt).fg(Color::DarkGrey),
                        },
                        match terminate {
                            KernelTerminate::Yes => Cell::new(terminate).fg(Color::Green),
                            KernelTerminate::No => Cell::new(terminate).fg(Color::DarkGrey),
                        },
                        match kill {
                            KernelKill::Yes => Cell::new(kill).fg(Color::Green),
                            KernelKill::No => Cell::new(kill).fg(Color::DarkGrey),
                        },
                    ]);
                }
                println!("{table}");
            }
        }

        Ok(())
    }
}
