use ask::Answer;
use cli_utils::color_print::cstr;
use common::{
    clap::{self, Parser, Subcommand},
    eyre::{Result, bail},
    tracing,
};
use server::{self, ServeOptions};
use version::STENCILA_VERSION;

use crate::{
    compile, convert, demo, execute, lint,
    logging::{LoggingFormat, LoggingLevel},
    merge, new, preview, render, sync, uninstall, upgrade,
};

/// CLI subcommands and global options
#[derive(Debug, Parser)]
#[command(
    name = "stencila",
    author,
    version = STENCILA_VERSION,
    about,
    long_about,
    disable_help_flag = true, // Grouped into global options below
    styles = Cli::styles(),
    after_long_help = CLI_AFTER_LONG_HELP
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Print help: `-h` for brief help, `--help` for more details.
    #[arg(
        short, long,
        action = clap::ArgAction::Help,
        global = true,
        help_heading = "Global Options",
        display_order = 100
    )]
    help: Option<bool>,

    /// Assume the answer `yes` to any interactive prompts
    ///
    /// The unlisted options `--no` and `--cancel` (and corresponding env vars)
    /// are also available.
    #[arg(
        long,
        global = true,
        help_heading = "Global Options",
        display_order = 110,
        conflicts_with = "no",
        conflicts_with = "cancel",
        env = "ASSUME_YES"
    )]
    pub yes: bool,

    /// Assume the answer `no` to any interactive prompts
    #[arg(
        long,
        global = true,
        hide = true,
        conflicts_with = "yes",
        conflicts_with = "cancel",
        env = "ASSUME_NO"
    )]
    pub no: bool,

    /// Assume the answer `cancel` to any interactive prompts
    #[arg(
        long,
        global = true,
        hide = true,
        conflicts_with = "yes",
        conflicts_with = "no",
        env = "ASSUME_CANCEL"
    )]
    pub cancel: bool,

    /// Display debug level logging and detailed error reports
    ///
    /// For trace level logging, use the unlisted --trace option. See
    /// documentation for other unlisted logging options --log-level,
    /// --log-format, log-filter.
    #[arg(
        long,
        global = true,
        help_heading = "Global Options",
        display_order = 120,
        conflicts_with = "trace",
        conflicts_with = "log_level",
        conflicts_with = "log_format",
        conflicts_with = "error_details"
    )]
    pub debug: bool,

    /// Display trace level logging and detailed error reports
    #[arg(
        long,
        global = true,
        hide = true,
        conflicts_with = "debug",
        conflicts_with = "log_level",
        conflicts_with = "log_format",
        conflicts_with = "error_details"
    )]
    pub trace: bool,

    /// The minimum log level to output
    #[arg(long, default_value = "info", global = true, hide = true)]
    pub log_level: LoggingLevel,

    /// A filter for log entries
    ///
    /// Allows more fine-grained control over which log entries are shown.
    /// To additionally see lower level entries for a specific crates use
    /// syntax such as `tokio=debug`.
    #[arg(
        long,
        default_value = "globset=warn,headless_chrome=warn,hyper=info,hyper_util=info,ignore=warn,keyring=info,mio=info,notify=warn,ort=error,reqwest=info,rustls=info,sled=info,tokenizers:=info,tokio=info,tungstenite=info",
        global = true,
        hide = true
    )]
    pub log_filter: String,

    /// The log format to use
    ///
    /// When `auto`, uses `simple` for terminals and `json`
    /// for non-TTY devices.
    #[arg(long, default_value = "auto", global = true, hide = true)]
    pub log_format: LoggingFormat,

    /// The details to include in error reports
    ///
    /// `none`, `auto`, `all`, or a comma separated list including `location`, `span`, or `env`.
    #[arg(long, default_value = "auto", global = true, hide = true)]
    pub error_details: String,

    /// Output a link to more easily report an issue
    #[arg(long, global = true, hide = true)]
    pub error_link: bool,

    /// Do not color any output
    #[arg(
        long,
        global = true,
        help_heading = "Global Options",
        display_order = 140,
        conflicts_with = "force_color",
        env = "NO_COLOR"
    )]
    pub no_color: bool,

    /// Force color in outputs, even when piping to non-TTY devices
    #[arg(
        long,
        global = true,
        hide = true,
        conflicts_with = "no_color",
        env = "FORCE_COLOR"
    )]
    pub force_color: bool,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Get help on all available commands</dim>
  <b>stencila</> <c>--help</>

  <dim># Create a new document</dim>
  <b>stencila new</> <g>article.md</>

  <dim># Convert a document to another format</dim>
  <b>stencila convert</> <g>input.md</> <g>output.pdf</>

  <dim># Check available formats</dim>
  <b>stencila formats list</>

  <dim># Execute a document</dim>
  <b>stencila execute</> <g>notebook.myst</>

  <dim># Preview a document with hot reloading</dim>
  <b>stencila preview</> <g>document.md</>
"
);

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

    pub fn assume_answer(&self) -> Option<Answer> {
        if self.yes {
            Some(Answer::Yes)
        } else if self.no {
            Some(Answer::No)
        } else if self.cancel {
            Some(Answer::Cancel)
        } else {
            None
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    New(new::Cli),

    Init(document::cli::Init),
    Config(document::cli::Config),

    Status(document::cli::Status),
    Add(document::cli::Add),
    Remove(document::cli::Remove),
    Move(document::cli::Move),
    Track(document::cli::Track),
    Untrack(document::cli::Untrack),
    Clean(document::cli::Clean),

    Rebuild(document::cli::Rebuild),
    Query(document::cli::Query),

    Convert(convert::Cli),
    Merge(merge::Cli),
    Sync(sync::Cli),

    Compile(compile::Cli),
    Lint(lint::Cli),
    Execute(execute::Cli),
    Render(render::Cli),

    Preview(preview::Cli),
    Publish(publish::Cli),
    Demo(demo::Demo),

    Serve(ServeOptions),
    /// Run the Language Server Protocol server
    Lsp,

    Prompts(prompts::cli::Cli),
    Models(models::cli::Cli),
    Kernels(kernels::cli::Cli),
    Formats(codecs::cli::Cli),
    Plugins(plugins::cli::Cli),
    Secrets(secrets::cli::Cli),
    Tools(tools::cli::Cli),

    Cloud(crate::cloud::Cli),
    Signin(crate::cloud::Signin),
    Signout(crate::cloud::Signout),

    Upgrade(upgrade::Cli),
    Uninstall(uninstall::Cli),
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

            Command::Init(init) => init.run().await?,
            Command::Config(config) => config.run().await?,

            Command::Status(status) => status.run().await?,
            Command::Add(add) => add.run().await?,
            Command::Remove(remove) => remove.run().await?,
            Command::Move(mov) => mov.run().await?,
            Command::Track(track) => track.run().await?,
            Command::Untrack(untrack) => untrack.run().await?,
            Command::Clean(clean) => clean.run().await?,

            Command::Rebuild(rebuild) => rebuild.run().await?,
            Command::Query(query) => query.run().await?,

            Command::Convert(convert) => convert.run().await?,
            Command::Merge(merge) => merge.run().await?,
            Command::Sync(sync) => sync.run().await?,

            Command::Compile(compile) => compile.run().await?,
            Command::Lint(lint) => lint.run().await?,
            Command::Execute(execute) => execute.run().await?,
            Command::Render(render) => render.run().await?,

            Command::Preview(preview) => preview.run().await?,
            Command::Publish(publish) => publish.run().await?,
            Command::Demo(demo) => demo.run().await?,

            Command::Serve(options) => server::serve(options).await?,

            Command::Prompts(prompts) => prompts.run().await?,
            Command::Models(models) => models.run().await?,
            Command::Kernels(kernels) => kernels.run().await?,
            Command::Formats(codecs) => codecs.run().await?,
            Command::Plugins(plugins) => plugins.run().await?,
            Command::Secrets(secrets) => secrets.run().await?,
            Command::Tools(tools) => tools.run().await?,

            Command::Cloud(cloud) => cloud.run().await?,
            Command::Signin(signin) => signin.run().await?,
            Command::Signout(signout) => signout.run().await?,

            Command::Upgrade(upgrade) => upgrade.run().await?,
            Command::Uninstall(uninstall) => uninstall.run().await?,

            // Handled before this function
            Command::Lsp => bail!("The LSP command should already been run"),
        }

        Ok(())
    }
}
