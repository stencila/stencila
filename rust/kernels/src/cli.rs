use cli_utils::table::{self, Attribute, Cell, CellAlignment, Color};
use kernel::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
    },
    schema::StringOrNumber,
    KernelAvailability, KernelForks, KernelInterrupt, KernelKill, KernelTerminate,
};

use crate::{list, Kernels};

/// Manage execution kernels
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// A command to perform with kernels
#[derive(Debug, Subcommand, Default)]
enum Command {
    /// List the kernels available
    #[default]
    List,

    /// Get information about a kernel
    ///
    /// Mainly used to check the version of the kernel runtime and
    /// operating system for debugging purpose.
    Info(Info),

    /// List packages available to a kernel
    ///
    /// Mainly used to check libraries available to a kernel
    /// for debugging purpose.
    Packages(Packages),
}

#[derive(Debug, Args)]
struct Info {
    /// The name of the kernel to get information for
    name: String,
}

#[derive(Debug, Args)]
struct Packages {
    /// The name of the kernel to list packages for
    name: String,

    /// A filter on the name of the kernel
    ///
    /// Only packages whose name contains this string will be included
    /// (case insensitive)
    filter: Option<String>,
}

impl Cli {
    // Run the CLI
    pub async fn run(self) -> Result<()> {
        match self.command.unwrap_or_default() {
            Command::List => {
                let mut table = table::new();
                table.set_header([
                    "Name",
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
                        Cell::new(kernel.name()).add_attribute(Attribute::Bold),
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
            Command::Info(Info { name }) => {
                let mut kernels = Kernels::default();
                let instance = kernels.create_instance(Some(&name)).await?;

                let info = instance.info().await?;
                println!(
                    "Name: {}\nVersion: {}\nOperating system: {}\n",
                    info.name,
                    info.options.software_version.as_deref().unwrap_or("?"),
                    info.options.operating_system.as_deref().unwrap_or("?"),
                );
            }
            Command::Packages(Packages { name, filter }) => {
                let mut kernels = Kernels::default();
                let instance = kernels.create_instance(Some(&name)).await?;

                let packages = instance.packages().await?;
                let packages = packages
                    .into_iter()
                    .filter(|package| {
                        if let Some(filter) = &filter {
                            package.name.to_lowercase().contains(&filter.to_lowercase())
                        } else {
                            true
                        }
                    })
                    .sorted_by(|a, b| a.name.cmp(&b.name));

                let mut table = table::new();
                table.set_header(["Package", "Version"]);

                for package in packages {
                    let version = match package.options.version.unwrap_or_default() {
                        StringOrNumber::String(version) => version,
                        StringOrNumber::Number(version) => version.to_string(),
                    };
                    table.add_row([
                        Cell::new(package.name).add_attribute(Attribute::Bold),
                        Cell::new(version).set_alignment(CellAlignment::Right),
                    ]);
                }

                println!("{table}");
            }
        }

        Ok(())
    }
}
