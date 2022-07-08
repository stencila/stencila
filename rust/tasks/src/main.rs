//! Mini CLI for testing this crate at the command line without compiling the whole `stencila` binary.
//!
//! Differs from the usual "mini cli" in that it falls back to attempting to run a task

#[cfg(feature = "cli")]
use common::{eyre::Result, tokio};

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> Result<()> {
    use cli_utils::{clap::Parser, tracing_subscriber, Run};
    use tasks::cli::{Command, Run_};

    tracing_subscriber::fmt().pretty().init();

    let args: Vec<String> = std::env::args().collect();
    let parsed_args = Command::try_parse_from(args.clone());
    let value_formats = ["md".to_string(), "yaml".to_string(), "json".to_string()];
    let error_format = "";
    match parsed_args {
        Ok(cmd) => cmd.print(&value_formats, error_format).await,
        Err(_) => {
            Run_::parse_from(args)
                .print(&value_formats, error_format)
                .await
        }
    };
    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {}
