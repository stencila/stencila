use app::DirType;
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::{bail, Result},
    tracing,
};
use server::{self, ServeOptions};
use version::STENCILA_VERSION;

use crate::{
    compile, convert, execute,
    logging::{LoggingFormat, LoggingLevel},
    new, preview, render, sync, uninstall, upgrade,
};

/// CLI subcommands and global options
#[derive(Debug, Parser)]
#[command(name = "stencila", author, version = STENCILA_VERSION, about, long_about, styles = Cli::styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Display debug level logging and detailed error reports
    ///
    /// Equivalent to using `--log-level=debug`, `--log-format=pretty`, and `--error-details=all`
    #[arg(
        long,
        global = true,
        conflicts_with = "trace",
        conflicts_with = "log_level",
        conflicts_with = "log_format",
        conflicts_with = "error_details"
    )]
    pub debug: bool,

    /// Display trace level logging and detailed error reports
    ///
    /// Equivalent to using `--log-level=trace`, `--log-format=pretty`, and `--error-details=all`
    #[arg(
        long,
        global = true,
        conflicts_with = "debug",
        conflicts_with = "log_level",
        conflicts_with = "log_format",
        conflicts_with = "error_details"
    )]
    pub trace: bool,

    /// The minimum log level to output
    #[arg(long, default_value = "info", global = true)]
    pub log_level: LoggingLevel,

    /// A filter for log entries
    ///
    /// Allows more fine-grained control over which log entries are shown.
    /// To additionally see lower level entries for a specific crates use
    /// syntax such as `tokio=debug`.
    #[arg(
        long,
        default_value = "globset=warn,hyper=info,hyper_util=info,ignore=warn,mio=info,notify=warn,ort=error,reqwest=info,sled=info,tokio=info,tungstenite=info",
        global = true
    )]
    pub log_filter: String,

    /// The log format to use
    ///
    /// When `auto`, uses `simple` for terminals and `json`
    /// for non-TTY devices.
    #[arg(long, default_value = "auto", global = true)]
    pub log_format: LoggingFormat,

    /// The details to include in error reports
    ///
    /// `auto`, `all`, or a comma separated list including `location`, `span`, or `env`.
    #[arg(long, default_value = "auto", global = true)]
    pub error_details: String,

    /// Output a link to more easily report an issue
    #[arg(long, global = true)]
    pub error_link: bool,
}

impl Cli {
    pub fn styles() -> clap::builder::Styles {
        use clap::builder::styling::*;
        Styles::styled()
            .header(AnsiColor::Blue.on_default().bold())
            .usage(AnsiColor::Cyan.on_default())
            .literal(AnsiColor::Cyan.on_default())
            .valid(AnsiColor::Green.on_default())
            .invalid(AnsiColor::Yellow.on_default())
            .error(AnsiColor::Red.on_default().bold())
            .placeholder(AnsiColor::Green.on_default())
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    New(new::Cli),

    Track(document::cli::Track),
    Untrack(document::cli::Untrack),
    Status(document::cli::Status),

    Convert(convert::Cli),
    Sync(sync::Cli),

    Compile(compile::Cli),
    Execute(execute::Cli),
    Render(render::Cli),

    Preview(preview::Cli),
    Publish(publish::Cli),

    Serve(ServeOptions),
    /// Run the Language Server Protocol server
    Lsp,

    Prompts(prompts::cli::Cli),
    Models(models::cli::Cli),
    Kernels(kernels::cli::Cli),
    Codecs(codecs::cli::Cli),
    Plugins(plugins::cli::Cli),
    Secrets(secrets::cli::Cli),

    Upgrade(upgrade::Cli),
    Uninstall(uninstall::Cli),

    Config(ConfigOptions),
}

#[derive(Debug, Args)]
pub struct ConfigOptions {
    #[arg(long, default_value = "config")]
    dir: DirType,

    #[arg(long)]
    ensure: bool,
}

impl Cli {
    /// Run the CLI command
    ///
    /// This function mainly exists to have a top level, instrumented function
    /// to call after error reporting and logging have been setup. This is
    /// useful because then CLI arguments are captured in span traces.
    #[tracing::instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        tracing::trace!("Running CLI command");

        match self.command {
            Command::New(new) => new.run().await?,

            Command::Track(track) => track.run().await?,
            Command::Untrack(untrack) => untrack.run().await?,
            Command::Status(status) => status.run().await?,

            Command::Convert(convert) => convert.run().await?,
            Command::Sync(sync) => sync.run().await?,

            Command::Compile(compile) => compile.run().await?,
            Command::Execute(execute) => execute.run().await?,
            Command::Render(render) => render.run().await?,

            Command::Preview(preview) => preview.run().await?,
            Command::Publish(publish) => publish.run().await?,

            Command::Serve(options) => server::serve(options).await?,

            Command::Prompts(prompts) => prompts.run().await?,
            Command::Models(models) => models.run().await?,
            Command::Kernels(kernels) => kernels.run().await?,
            Command::Codecs(codecs) => codecs.run().await?,
            Command::Plugins(plugins) => plugins.run().await?,
            Command::Secrets(secrets) => secrets.run().await?,

            Command::Upgrade(upgrade) => upgrade.run().await?,
            Command::Uninstall(uninstall) => uninstall.run()?,

            Command::Config(options) => {
                // TODO: Make options.dir an option, and if it not there, show all folders.
                let dir = app::get_app_dir(options.dir, options.ensure)?;
                println!("{}", dir.display());
            }

            // Handled before this function
            Command::Lsp => bail!("The LSP command should already been run"),
        }

        Ok(())
    }
}
