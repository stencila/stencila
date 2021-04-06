use anyhow::Result;
use stencila::{
    cli::{print_error, run_command, Args, Command, GLOBAL_ARGS},
    config, interact, logging, plugins, upgrade,
};
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse args into a command
    let parsed_args = Args::from_iter_safe(args.clone());
    let Args {
        command,
        debug,
        info,
        warn,
        error,
        ..
    } = match parsed_args {
        Ok(args) => args,
        Err(error) => {
            if args.contains(&"-i".to_string()) || args.contains(&"--interact".to_string()) {
                // Parse the global options ourselves so that user can
                // pass an incomplete command prefix to interactive mode
                Args {
                    command: None,
                    debug: args.contains(&"--debug".to_string()),
                    info: args.contains(&"--info".to_string()),
                    warn: args.contains(&"--warn".to_string()),
                    error: args.contains(&"--error".to_string()),
                    interact: true,
                }
            } else {
                // Print the error `clap` help or usage message and exit
                eprintln!("{}", error);
                std::process::exit(exitcode::USAGE);
            }
        }
    };

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
    let mut plugins_store = plugins::Store::load()?;

    // Get the result of running the command
    let result = if let Some(command) = command {
        run_command(command, &config, &mut plugins_store).await
    } else {
        let prefix: Vec<String> = args
            .into_iter()
            // Remove executable name
            .skip(1)
            // Remove the global args which can not be applied to each interactive line
            .filter(|arg| !GLOBAL_ARGS.contains(&arg.as_str()))
            .collect();
        interact::run(prefix, &config, &mut plugins_store).await
    };

    // Join the upgrade thread and log any errors
    if let Some(upgrade_thread) = upgrade_thread {
        if let Err(_error) = upgrade_thread.join() {
            tracing::warn!("Error while attempting to join upgrade thread")
        }
    }

    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            print_error(error);
            std::process::exit(exitcode::SOFTWARE);
        }
    }
}
