use clap::{Parser, Subcommand};
use eyre::{Result, bail};

use stencila_ask::Answer;
use stencila_cli_utils::color_print::cstr;
use stencila_server::ServeOptions;
use stencila_version::STENCILA_VERSION;

use crate::{
    compile, convert, db, demo, execute, lint,
    logging::{LoggingFormat, LoggingLevel},
    merge, new, preview, pull, push, render, sync, uninstall, upgrade,
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
        default_value = "globset=warn,headless_chrome=warn,h2=info,hyper=info,hyper_util=info,ignore=warn,keyring=info,mio=info,notify=warn,ort=error,reqwest=info,rustls=info,sled=info,tokenizers:=info,tokio=info,tungstenite=info",
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
        env = "NO_COLOR",
        value_parser = parse_env_var_bool
    )]
    pub no_color: bool,

    /// Force color in outputs, even when piping to non-TTY devices
    #[arg(
        long,
        global = true,
        hide = true,
        conflicts_with = "no_color",
        env = "FORCE_COLOR",
        value_parser = parse_env_var_bool
    )]
    pub force_color: bool,
}

/// Parse boolean environment variables
///
/// This needs to explicitly handle "false" because that is the default value
/// passed here by clap. Note however, that NO_COLOR and FORCE_COLOR are
/// observed by presence/absence by most libraries so setting NO_COLOR=false
/// should be avoided because most tools will treat that as "no color present".
fn parse_env_var_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        // Any other non-empty value is treated as true
        _ => Ok(!value.is_empty()),
    }
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

    Init(stencila_document::cli::Init),
    Config(stencila_document::cli::Config),

    Status(stencila_document::cli::Status),
    Move(stencila_document::cli::Move),
    Track(stencila_document::cli::Track),
    Untrack(stencila_document::cli::Untrack),
    Clean(stencila_document::cli::Clean),
    Convert(convert::Cli),
    Merge(merge::Cli),
    Sync(sync::Cli),

    Compile(compile::Cli),
    Lint(lint::Cli),
    Execute(execute::Cli),
    Render(render::Cli),
    Query(stencila_document::cli::Query),

    Preview(preview::Cli),
    Publish(stencila_publish::Cli),
    Pull(pull::Cli),
    Push(push::Cli),
    Demo(demo::Demo),

    Db(db::Cli),

    Prompts(stencila_prompts::cli::Cli),
    Models(stencila_models::cli::Cli),
    Kernels(stencila_kernels::cli::Cli),
    Linters(stencila_linters::cli::Cli),
    Formats(stencila_codecs::cli::Cli),
    Themes(stencila_themes::cli::Cli),
    Secrets(stencila_secrets::cli::Cli),
    Tools(stencila_tools::cli::Cli),

    Serve(ServeOptions),
    Snap(stencila_snap::cli::Cli),
    /// Run the Language Server Protocol server
    Lsp,

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
            Command::New(new) => new.run().await,

            Command::Init(init) => init.run().await,
            Command::Config(config) => config.run().await,

            Command::Status(status) => status.run().await,
            Command::Move(mov) => mov.run().await,
            Command::Track(track) => track.run().await,
            Command::Untrack(untrack) => untrack.run().await,
            Command::Clean(clean) => clean.run().await,

            Command::Convert(convert) => convert.run().await,
            Command::Merge(merge) => merge.run().await,
            Command::Sync(sync) => sync.run().await,

            Command::Compile(compile) => compile.run().await,
            Command::Lint(lint) => lint.run().await,
            Command::Execute(execute) => execute.run().await,
            Command::Render(render) => render.run().await,
            Command::Query(query) => query.run().await,

            Command::Preview(preview) => preview.run().await,
            Command::Publish(publish) => publish.run().await,
            Command::Pull(pull) => pull.run().await,
            Command::Push(push) => push.run().await,
            Command::Demo(demo) => demo.run().await,

            Command::Db(db) => db.run().await,

            Command::Prompts(prompts) => prompts.run().await,
            Command::Models(models) => models.run().await,
            Command::Kernels(kernels) => kernels.run().await,
            Command::Linters(linters) => linters.run().await,
            Command::Formats(codecs) => codecs.run().await,
            Command::Themes(themes) => themes.run().await,
            Command::Secrets(secrets) => secrets.run().await,
            Command::Tools(tools) => tools.run().await,

            Command::Serve(options) => stencila_server::serve(options).await,
            Command::Snap(snap) => snap.run().await,

            Command::Cloud(cloud) => cloud.run().await,
            Command::Signin(signin) => signin.run().await,
            Command::Signout(signout) => signout.run().await,

            Command::Upgrade(upgrade) => upgrade.run().await,
            Command::Uninstall(uninstall) => uninstall.run().await,

            // Handled before this function
            Command::Lsp => bail!("The LSP command should already been run"),
        }
    }
}
