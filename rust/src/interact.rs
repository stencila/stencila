use crate::{config, convert, open, plugins, serve, upgrade, util::dirs};
use anyhow::{bail, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Stencila command line tool",
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub struct Line {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
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

/// Run the interactive REPL
#[cfg(feature = "interact")]
#[tracing::instrument]
pub async fn run(config: &config::Config) -> Result<()> {
    use rustyline::{error::ReadlineError, Editor};

    let history_file = dirs::config(true)?.join("history.txt");

    let mut rl = Editor::<()>::new();
    if rl.load_history(&history_file).is_err() {
        tracing::debug!("No previous history found")
    }

    let mut config = config.clone();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let clap = Line::clap();
                let line = line.split_whitespace().collect::<Vec<&str>>();
                match clap.get_matches_from_safe(line) {
                    Ok(matches) => {
                        let Line { command } = Line::from_clap(&matches);
                        if let Err(error) = match command {
                            Command::Open(args) => open::cli::run(args).await,
                            Command::Convert(args) => convert::cli::run(args),
                            Command::Serve(args) => serve::cli::run(args, &config.serve).await,
                            Command::Plugins(args) => {
                                plugins::cli::run(args, &config.plugins).await
                            }
                            Command::Upgrade(args) => upgrade::cli::run(args, &config.upgrade),
                            Command::Config(args) => match config::cli::run(args, &config) {
                                Ok(config_changed) => {
                                    // Update the configuration (may have been changed by `set` and `reset`)
                                    config = config_changed;
                                    Ok(())
                                }
                                Err(err) => Err(err),
                            },
                        } {
                            eprintln!("{}", error)
                        }
                    }
                    Err(error) => {
                        if error.kind == structopt::clap::ErrorKind::VersionDisplayed {
                            print!("{}", error)
                        } else if error.kind == structopt::clap::ErrorKind::HelpDisplayed
                            || error.kind == structopt::clap::ErrorKind::MissingArgumentOrSubcommand
                        {
                            // Remove the unnecessary command / version line at the start
                            let lines = format!("{}\n", error)
                                .to_string()
                                .lines()
                                .skip(1)
                                .map(str::to_string)
                                .collect::<Vec<String>>()
                                .join("\n");
                            print!("{}", lines)
                        } else {
                            tracing::debug!("{:?}", error.kind);
                            eprintln!("{}", error)
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                tracing::info!("Ctrl-C pressed, interrupting current command");
                // TODO
            }
            Err(ReadlineError::Eof) => {
                tracing::info!("Ctrl-D pressed, ending session");
                break;
            }
            Err(error) => bail!(error),
        }
    }
    rl.save_history(&history_file)?;

    Ok(())
}
