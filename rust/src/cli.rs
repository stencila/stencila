use crate::{config, convert, open, plugins, serve, upgrade};
use anyhow::Result;
use regex::Regex;
use structopt::StructOpt;
use strum::{Display, EnumVariantNames};
#[derive(Debug, StructOpt)]
#[structopt(
    about = "Stencila command line tool",
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Option<Command>,

    /// Show debug level log events (and above)
    #[structopt(long, global = true, conflicts_with_all = &["info", "warn", "error"])]
    pub debug: bool,

    /// Show info level log events (and above; default)
    #[structopt(long, global = true, conflicts_with_all = &["debug", "warn", "error"])]
    pub info: bool,

    /// Show warning level log events (and above)
    #[structopt(long, global = true, conflicts_with_all = &["debug", "info", "error"])]
    pub warn: bool,

    /// Show error level log entries only
    #[structopt(long, global = true, conflicts_with_all = &["debug", "info", "warn"])]
    pub error: bool,

    /// Enter interactive mode (with any command and options as the prefix)
    #[structopt(short, long, global = true)]
    pub interact: bool,
}

pub const GLOBAL_ARGS: [&str; 6] = ["--debug", "--info", "--warn", "--error", "--interact", "-i"];

#[derive(Debug, Display, StructOpt, EnumVariantNames)]
#[strum(serialize_all = "lowercase")]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
pub enum Command {
    Open(open::cli::Args),
    Convert(convert::cli::Args),
    Serve(serve::cli::Args),
    Plugins(plugins::cli::Args),
    Config(config::cli::Args),
    Upgrade(upgrade::cli::Args),
}

/// Run a command
pub async fn run_command(command: Command, config: &config::Config) -> Result<()> {
    match command {
        Command::Open(args) => open::cli::run(args).await,
        Command::Convert(args) => convert::cli::run(args),
        Command::Serve(args) => serve::cli::run(args, &config.serve).await,
        Command::Plugins(args) => plugins::cli::run(args, &config.plugins).await,
        Command::Upgrade(args) => upgrade::cli::run(args, &config.upgrade),
        Command::Config(args) => config::cli::run(args, &config).map(|_| ()),
    }
}

/// Print an error
pub fn print_error(error: anyhow::Error) {
    // Remove any error label already in error string
    let re = Regex::new(r"\s*error\s*:?").unwrap();
    let error = error.to_string();
    let error = if let Some(captures) = re.captures(error.as_str()) {
        error.replace(&captures[0], "").trim().into()
    } else {
        error
    };
    eprintln!("ERROR: {}", error);
}

/// Parse a vector of command line arguments into parameters of a method call
pub fn parse_params(params: Vec<String>) -> serde_json::Value {
    let re = Regex::new(r"(\w+)(:?=)(.+)").unwrap();
    let mut object = serde_json::json!({});
    for param in params {
        if let Some(captures) = re.captures(param.as_str()) {
            let (name, kind, value) = (&captures[1], &captures[2], &captures[3]);
            if kind == ":=" {
                object[name] = match serde_json::from_str(value) {
                    Ok(value) => value,
                    Err(_) => serde_json::Value::String(value.to_string()),
                };
            } else {
                object[name] = serde_json::Value::from(value);
            }
        }
    }
    object
}
