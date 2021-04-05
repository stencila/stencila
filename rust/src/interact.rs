use crate::{
    cli::{print_error, Command},
    config, convert, open, plugins, serve, upgrade,
    util::dirs,
};
use anyhow::{anyhow, bail, Result};
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

/// Run the interactive REPL
#[cfg(feature = "interact")]
#[tracing::instrument]
pub async fn run(prefix: &Vec<String>, config: &config::Config) -> Result<()> {
    use rustyline::{error::ReadlineError, Editor};

    let history_file = dirs::config(true)?.join("history.txt");

    let mut rl = Editor::<()>::new();
    if rl.load_history(&history_file).is_err() {
        tracing::debug!("No previous history found")
    }

    let mut prefix = prefix.clone();
    let mut config = config.clone();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);

                let mut args = line
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<String>>();

                if let Some(first) = line.trim_start().chars().nth(0) {
                    if first == '~' {
                        println!("Command prefix is {:?}", prefix);
                        continue;
                    } else if first == '<' {
                        prefix = args[1..].into();
                        println!("Set command prefix to {:?}", prefix);
                        continue;
                    } else if first == '>' {
                        prefix.clear();
                        println!("Cleared command prefix");
                        continue;
                    } else if first == '?' {
                        args[0] = "help".into();
                    }
                };

                let args = [prefix.as_slice(), args.as_slice()].concat();
                match Line::clap().get_matches_from_safe(args) {
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
                            print_error(error)
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
                            print_error(anyhow!(error))
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
