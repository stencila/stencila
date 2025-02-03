use std::path::PathBuf;

use cli_utils::{
    table::{self, Attribute, Cell, CellAlignment, Color},
    AsFormat, Code, ToStdout,
};
use kernel::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::{OptionExt, Result},
        itertools::Itertools,
        serde_yaml,
        tokio::fs::read_to_string,
    },
    format::Format,
    schema::StringOrNumber,
    KernelAvailability, KernelForks, KernelInterrupt, KernelKill, KernelLinting,
    KernelLintingOptions, KernelLintingOutput, KernelProvider, KernelSpecification,
    KernelTerminate, KernelType,
};

use crate::Kernels;

/// Manage execution kernels
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Info(Info),
    Packages(Packages),
    Execute(Execute),
    Evaluate(Evaluate),
    Lint(Lint),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return List::default().run().await;
        };

        match command {
            Command::List(list) => list.run().await,
            Command::Info(info) => info.run().await,
            Command::Packages(pkgs) => pkgs.run().await,
            Command::Execute(exec) => exec.run().await,
            Command::Evaluate(eval) => eval.run().await,
            Command::Lint(lint) => lint.run().await,
        }
    }
}

/// List the kernels available
#[derive(Debug, Default, Args)]
struct List {
    /// Only list kernels of a particular type
    #[arg(short, long)]
    r#type: Option<KernelType>,

    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl List {
    async fn run(self) -> Result<()> {
        let list = super::list().await;

        let list = if let Some(type_) = self.r#type {
            list.into_iter()
                .filter(|kernel| kernel.r#type() == type_)
                .collect()
        } else {
            list
        };

        if let Some(format) = self.r#as {
            let list = list
                .into_iter()
                .map(|kernel| KernelSpecification::from(kernel.as_ref()))
                .collect_vec();

            Code::new_from(format.into(), &list)?.to_stdout();

            return Ok(());
        }

        let mut table = table::new();
        table.set_header([
            "Name",
            "Type",
            "Provider",
            "Availability",
            "Languages",
            "Lint",
            "Fork",
            "Interrupt",
            "Terminate",
            "Kill",
        ]);

        for kernel in list {
            use KernelAvailability::*;

            let r#type = kernel.r#type();
            let provider = kernel.provider();
            let availability = kernel.availability();
            let langs = kernel
                .supports_languages()
                .iter()
                .map(|format| format.name())
                .join(", ");
            let lint = kernel.supports_linting();
            let forks = kernel.supports_forks();
            let interrupt = kernel.supports_interrupt();
            let terminate = kernel.supports_terminate();
            let kill = kernel.supports_kill();

            table.add_row([
                Cell::new(kernel.name()).add_attribute(Attribute::Bold),
                match r#type {
                    KernelType::Diagrams => Cell::new("diagrams").fg(Color::DarkYellow),
                    KernelType::Math => Cell::new("math").fg(Color::Blue),
                    KernelType::Programming => Cell::new("programming").fg(Color::Green),
                    KernelType::Styling => Cell::new("styling").fg(Color::Magenta),
                    KernelType::Templating => Cell::new("templating").fg(Color::Cyan),
                },
                match provider {
                    KernelProvider::Builtin => Cell::new("builtin").fg(Color::Green),
                    KernelProvider::Environment => Cell::new("environ").fg(Color::Cyan),
                    KernelProvider::Plugin(name) => {
                        Cell::new(format!("plugin \"{name}\"")).fg(Color::Blue)
                    }
                },
                match availability {
                    Available => Cell::new(availability).fg(Color::Green),
                    Disabled => Cell::new(availability).fg(Color::DarkBlue),
                    Installable => Cell::new(availability).fg(Color::Cyan),
                    Unavailable => Cell::new(availability).fg(Color::Grey),
                },
                Cell::new(langs),
                match lint {
                    KernelLinting::No => Cell::new(lint).fg(Color::DarkGrey),
                    KernelLinting::Format => Cell::new(lint).fg(Color::Yellow),
                    KernelLinting::Check => Cell::new(lint).fg(Color::Magenta),
                    KernelLinting::Fix => Cell::new(lint).fg(Color::Blue),
                    KernelLinting::FormatCheck => Cell::new(lint).fg(Color::Cyan),
                    KernelLinting::FormatFix => Cell::new(lint).fg(Color::Green),
                },
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

        table.to_stdout();

        Ok(())
    }
}

/// Get information about a kernel
///
/// Mainly used to check the version of the kernel runtime and
/// operating system for debugging purpose.
#[derive(Debug, Args)]
struct Info {
    /// The name of the kernel to get information for
    name: String,
}

impl Info {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        let mut kernels = Kernels::new_here();
        let instance = kernels.create_instance(Some(&self.name)).await?;

        let info = instance.lock().await.info().await?;
        println!(
            "Name: {}\nVersion: {}\nOperating system: {}\n",
            info.name,
            info.options.software_version.as_deref().unwrap_or("?"),
            info.options.operating_system.as_deref().unwrap_or("?"),
        );

        Ok(())
    }
}

/// List packages available to a kernel
///
/// Mainly used to check libraries available to a kernel
/// for debugging purpose.
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

impl Packages {
    async fn run(self) -> Result<()> {
        let mut kernels = Kernels::new_here();
        let instance = kernels.create_instance(Some(&self.name)).await?;

        let packages = instance.lock().await.packages().await?;
        let packages = packages
            .into_iter()
            .filter(|package| {
                if let Some(filter) = &self.filter {
                    package.name.to_lowercase().contains(&filter.to_lowercase())
                } else {
                    true
                }
            })
            .sorted_by(|a, b| a.name.cmp(&b.name));

        let mut table = table::new();
        table.set_header(["Package", "Version"]);

        for package in packages {
            let version = match package.version.unwrap_or_default() {
                StringOrNumber::String(version) => version,
                StringOrNumber::Number(version) => version.to_string(),
            };
            table.add_row([
                Cell::new(package.name).add_attribute(Attribute::Bold),
                Cell::new(version).set_alignment(CellAlignment::Right),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Execute code in a kernel
///
/// Creates a temporary kernel instance, executes one or more lines of code,
/// and returns any outputs and execution messages.
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

impl Execute {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        let mut kernels = Kernels::new_here();
        let instance = kernels.create_instance(Some(&self.name)).await?;

        let code = self.code.replace("\\n", "\n");
        let (outputs, messages) = instance.lock().await.execute(&code).await?;

        // TODO: creates a `Map` output type that can be used to display sections with headers
        // instead of the following printlns

        println!("Outputs");
        Code::new(Format::Yaml, &serde_yaml::to_string(&outputs)?).to_stdout();

        println!("Messages");
        Code::new(Format::Yaml, &serde_yaml::to_string(&messages)?).to_stdout();

        Ok(())
    }
}

/// Evaluate a code expression in a kernel
///
/// Creates a temporary kernel instance, evaluates the expression in it,
/// and returns the output and any execution messages.
///
/// Mainly intended for quick testing of kernels during development.
#[derive(Debug, Args)]
#[clap(alias = "eval")]
struct Evaluate {
    /// The name of the kernel to evaluate code in
    name: String,

    /// The code expression to evaluate
    code: String,
}

impl Evaluate {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        let mut kernels = Kernels::new_here();
        let instance = kernels.create_instance(Some(&self.name)).await?;

        let (output, messages) = instance.lock().await.evaluate(&self.code).await?;

        // TODO: creates a `Map` output type that can be used to display sections with headers
        // instead of the following printlns

        println!("Output");
        Code::new(Format::Yaml, &serde_yaml::to_string(&output)?).to_stdout();

        println!("Messages");
        Code::new(Format::Yaml, &serde_yaml::to_string(&messages)?).to_stdout();

        Ok(())
    }
}

/// Lint code using the linting tool/s associated with a kernel
///
/// Note that this does not affect the file. It only prints how it
/// would be formatted/fixed and any diagnostics.
///
/// Mainly intended for testing of linting by kernels during
/// development of Stencila.
#[derive(Debug, Args)]
struct Lint {
    /// The file to lint
    file: PathBuf,

    /// Format the code
    #[arg(long)]
    format: bool,

    /// Fix warnings and errors where possible
    #[arg(long)]
    fix: bool,
}

impl Lint {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        let format = Format::from_path(&self.file);
        let code = read_to_string(&self.file).await?;
        let dir = self.file.parent().ok_or_eyre("file is not in a dir")?;

        let KernelLintingOutput {
            code,
            output,
            messages,
            authors,
        } = crate::lint(
            &code,
            dir,
            &format,
            KernelLintingOptions {
                fix: self.fix,
                format: self.format,
            },
        )
        .await?;

        if let Some(code) = code {
            println!("Formatted and/or fixed code:\n");
            Code::new(format.clone(), &code).to_stdout();
        }

        if let Some(output) = output {
            println!("Diagnostic output:\n");
            Code::new(format, &output).to_stdout();
        }

        if let Some(messages) = messages {
            println!("Diagnostic messages:\n");
            Code::new(Format::Yaml, &serde_yaml::to_string(&messages)?).to_stdout();
        }

        if let Some(authors) = authors {
            println!("Contributors:\n");
            Code::new(Format::Yaml, &serde_yaml::to_string(&authors)?).to_stdout();
        }

        Ok(())
    }
}
