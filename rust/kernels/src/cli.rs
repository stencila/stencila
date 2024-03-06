use cli_utils::{
    table::{self, Attribute, Cell, CellAlignment, Color},
    Code, ToStdout,
};
use kernel::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
        serde_yaml,
    },
    format::Format,
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

    Execute(Execute),
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

/// Execute some code in a kernel
///
/// Creates a temporary kernel instance, executes one or more lines of code,
/// and returns any decoded outputs and execution messages.
///
/// Mainly intended for quick testing of kernels during development.
#[derive(Debug, Args)]
#[clap(alias = "exec")]
struct Execute {
    /// The name of the kernel to execute code in
    name: String,

    /// The code to execute
    ///
    /// Escaped newline characters (i.e. "\n") in the code will be transformed into new lines
    /// before passing to the kernel.
    code: String,
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
            Command::Execute(Execute { name, code }) => {
                let mut kernels = Kernels::default();
                let instance = kernels.create_instance(Some(&name)).await?;

                let code = code.replace("\\n", "\n");
                let (outputs, messages) = instance.execute(&code).await?;

                // TODO: creates a `Map` output type that can be used to display sections with headers
                // instead of the following printlns

                println!("Outputs");
                Code::new(Format::Yaml, &serde_yaml::to_string(&outputs)?).to_stdout();

                println!("Messages");
                Code::new(Format::Yaml, &serde_yaml::to_string(&messages)?).to_stdout();
            }
        }

        Ok(())
    }
}
