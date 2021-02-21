use crate::decode;
use crate::encode;
use crate::nodes::Node;
use crate::open;
use crate::request;
use crate::serve;
use crate::upgrade;
use crate::validate;
use anyhow::Context;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Stencila command line tool",
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
struct Args {
    #[structopt(subcommand)]
    command: Command,

    /// Where to write the output to
    #[structopt(global = true, short, long, default_value = "stdout")]
    output: String,

    /// Format to encode the output as
    #[structopt(global = true, short, long, default_value = "derived")]
    to: String,
}

#[derive(Debug, StructOpt)]
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
}

pub async fn cli(args: Vec<String>) -> i32 {
    let Args {
        command,
        output,
        to,
    } = Args::from_iter(args);

    let node = match command {
        Command::Open(command) => open::cli::open(command).await,
        Command::Decode(command) => decode::cli::decode(command),
        Command::Validate(command) => validate::cli::validate(command),
        Command::Serve(command) => serve::cli::serve(command).await,
        Command::Request(command) => request::cli::request(command).await,
        Command::Upgrade(command) => {
            upgrade::cli::upgrade(command).unwrap();
            Ok(Node::Null)
        }
    };
    match node {
        Ok(node) => {
            match node {
                Node::Null => (),
                _ => {
                    let format = match to.as_str() {
                        "derived" => match output.as_str() {
                            "stdout" => "cli",
                            _ => Path::new(&output)
                                .extension()
                                .and_then(OsStr::to_str)
                                .unwrap(),
                        },
                        _ => to.as_str(),
                    };

                    let encoded = encode::encode(node, format.to_string())
                        .context("Encoding to output")
                        .unwrap();

                    match output.as_str() {
                        "stdout" => {
                            let stdout: &mut dyn std::io::Write = &mut std::io::stdout();
                            writeln!(stdout, "{}", encoded)
                                .context("Writing output to stdout")
                                .unwrap();
                        }
                        _ => {
                            fs::write(output, encoded)
                                .context("Writing output to file")
                                .unwrap();
                        }
                    }
                }
            }
            exitcode::OK
        }
        Err(error) => {
            // Write the error to the terminal
            // TODO Send this to a logger
            eprintln!("{}", error);
            exitcode::SOFTWARE
        }
    }
}
