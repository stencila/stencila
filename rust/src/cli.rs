use crate::config;
use crate::decode;
use crate::open;
use crate::request;
use crate::serve;
use crate::upgrade;
use crate::validate;
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
}

#[derive(Debug, Display, StructOpt, EnumVariantNames)]
#[strum(serialize_all = "lowercase")]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
pub enum Command {
    Open(open::cli::Args),
    Decode(decode::cli::Args),
    Validate(validate::cli::Args),
    Serve(serve::cli::Args),
    Request(request::cli::Args),
    Upgrade(upgrade::cli::Args),
    Config(config::cli::Args),
}

pub async fn cli(args: Vec<String>) -> i32 {
    let Args { command } = Args::from_iter(args);

    let result = match command {
        Command::Open(command) => open::cli::open(command).await,
        Command::Decode(command) => decode::cli::decode(command),
        Command::Validate(command) => validate::cli::validate(command),
        Command::Serve(command) => serve::cli::serve(command).await,
        Command::Request(command) => request::cli::request(command).await,
        Command::Upgrade(command) => upgrade::cli::upgrade(command),
        Command::Config(command) => config::cli::config(command),
    };
    match result {
        Ok(_) => exitcode::OK,
        Err(error) => {
            // Write the error to the terminal
            // TODO Send this to a logger
            eprintln!("{}", error);
            exitcode::SOFTWARE
        }
    }
}
