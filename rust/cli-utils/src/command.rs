use crate::{result, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Run {
    /// Run the command
    async fn run(&self) -> Result;

    /// Run the command and print it to the console
    async fn print(&self, formats: &[String]) {
        match self.run().await {
            Ok(value) => {
                if let Err(error) = result::print::value(value, formats) {
                    result::print::error(error)
                }
            }
            Err(error) => result::print::error(error),
        }
    }
}

/// Mini CLI for testing crates at the command line without compiling the whole `stencila` binary.
#[macro_export]
macro_rules! mini_main {
    ($command:ident) => {
        #[tokio::main]
        async fn main() {
            use cli_utils::{tracing_subscriber, Run};
            use structopt::StructOpt;

            tracing_subscriber::fmt().pretty().init();

            $command::from_args()
                .print(&["md".to_string(), "yaml".to_string(), "json".to_string()])
                .await
        }
    };
}
