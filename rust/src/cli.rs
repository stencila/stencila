use crate::{config, convert, interact, logging, open, plugins, serve, upgrade};
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
    /// Show debug level log events (and above)
    #[structopt(long, conflicts_with_all = &["info", "warn", "error"])]
    pub debug: bool,

    /// Show info level log events (and above; default)
    #[structopt(long, conflicts_with_all = &["debug", "warn", "error"])]
    pub info: bool,

    /// Show warning level log events (and above)
    #[structopt(long, conflicts_with_all = &["debug", "info", "error"])]
    pub warn: bool,

    /// Show error level log entries only
    #[structopt(long, conflicts_with_all = &["debug", "info", "warn"])]
    pub error: bool,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}

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

pub async fn cli(args: Vec<String>) -> Result<i32> {
    // Parse args into a command
    let Args {
        command,
        debug,
        info,
        warn,
        error,
    } = Args::from_iter(args);

    // Determine the log level to use on stderr
    let level = if debug {
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

    // Create a preliminary logging subscriber to be able to log any issues
    // when reading the config.
    let prelim_subscriber_guard = logging::prelim();
    let config = config::read()?;
    drop(prelim_subscriber_guard);

    // To ensure all log events get written to file, take guards here, so that
    // non blocking writers do not get dropped until the end of this function.
    // See https://tracing.rs/tracing_appender/non_blocking/struct.workerguard
    let _logging_guards = logging::init(Some(level), &config.logging)?;

    // If not explicitly upgrading then run an upgrade check in the background
    let upgrade_thread = if let Some(Command::Upgrade(_)) = command {
        None
    } else {
        Some(upgrade::upgrade_auto(&config.upgrade))
    };

    // Load plugins
    plugins::read_plugins()?;

    let result = if command.is_none() {
        interact::run()
    } else {
        match command.unwrap() {
            Command::Open(args) => open::cli::run(args).await,
            Command::Convert(args) => convert::cli::run(args),
            Command::Serve(args) => serve::cli::run(args, &config.serve).await,
            Command::Plugins(args) => plugins::cli::run(args, &config.plugins).await,
            Command::Upgrade(args) => upgrade::cli::run(args, &config.upgrade),
            Command::Config(args) => config::cli::run(args, &config),
        }
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
