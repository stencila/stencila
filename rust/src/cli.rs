use crate::{config, convert, logging, open, plugins, request, serve, upgrade, validate};
use anyhow::Result;
use regex::Regex;
use structopt::StructOpt;
use strum::{Display, EnumVariantNames};

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Stencila command line tool",
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Command,

    /// Emit tracing level (and above) log entries
    #[structopt(long, conflicts_with_all = &["debug", "info", "warn", "error"])]
    pub trace: bool,

    /// Emit debug level (and above) log entries
    #[structopt(long, conflicts_with_all = &["trace", "info", "warn", "error"])]
    pub debug: bool,

    /// Emit info level (and above) log entries
    #[structopt(long, conflicts_with_all = &["trace", "debug", "warn", "error"])]
    pub info: bool,

    /// Emit waning level (and above) log entries
    #[structopt(long, conflicts_with_all = &["trace", "debug", "info", "error"])]
    pub warn: bool,

    /// Emit error level log entries only
    #[structopt(long, conflicts_with_all = &["trace", "debug", "info", "warn"])]
    pub error: bool,
}

#[derive(Debug, Display, StructOpt, EnumVariantNames)]
#[strum(serialize_all = "lowercase")]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
pub enum Command {
    Open(open::cli::Args),
    Convert(convert::cli::Args),
    Validate(validate::cli::Args),
    Serve(serve::cli::Args),
    Request(request::cli::Args),
    Upgrade(upgrade::cli::Args),
    Plugins(plugins::cli::Args),
    Config(config::cli::Args),
}

pub async fn cli(args: Vec<String>) -> Result<i32> {
    // Parse args into a command
    let Args {
        command,
        trace,
        debug,
        info,
        warn,
        error,
    } = Args::from_iter(args);

    // Determine the log level to use on stderr
    let level = if trace {
        logging::Level::Trace
    } else if debug {
        logging::Level::Debug
    } else if info {
        logging::Level::Info
    } else if warn {
        logging::Level::Warn
    } else if error {
        logging::Level::Error
    } else {
        logging::Level::Info
    };

    // To ensure all log events get written to file, take guards here, so it does
    // not get dropped until the end of this function.
    // See https://tracing.rs/tracing_appender/non_blocking/struct.workerguard
    let _logging_guards = logging::init(Some(level))?;

    // If not explicitly upgrading then run an upgrade check in the background
    let upgrade_thread = if let Command::Upgrade(_) = command {
        None
    } else {
        Some(upgrade::upgrade_auto())
    };

    // Load plugins
    plugins::read_plugins()?;

    // Run the command
    let result = match command {
        Command::Open(command) => open::cli::open(command).await,
        Command::Convert(command) => convert::cli::convert(command),
        Command::Validate(command) => validate::cli::validate(command),
        Command::Serve(command) => serve::cli::serve(command).await,
        Command::Request(command) => request::cli::request(command).await,
        Command::Upgrade(command) => upgrade::cli::upgrade(command),
        Command::Plugins(command) => plugins::cli::plugins(command).await,
        Command::Config(command) => config::cli::config(command),
    };

    // Join the upgrade thread and log any errors
    if let Some(upgrade_thread) = upgrade_thread {
        if let Err(_error) = upgrade_thread.join() {
            tracing::warn!("Error while attempting to join upgrade thread")
        }
    }

    match result {
        Ok(_) => Ok(exitcode::OK),
        Err(error) => {
            tracing::error!("{}", error);
            Ok(exitcode::SOFTWARE)
        }
    }
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
