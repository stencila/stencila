use crate::export::export;
use crate::import::import;
use anyhow::Result;

pub fn convert(input: &str, output: &str, from: Option<String>, to: Option<String>) -> Result<()> {
    let imported = import(input, from)?;
    export(imported, output, to)
}

#[cfg(feature = "watch")]
pub fn convert_watch(
    input: &str,
    output: &str,
    from: Option<String>,
    to: Option<String>,
    watch: &str,
) -> Result<()> {
    tracing::info!("Watching '{}' for changes", watch);

    use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
    use std::time::Duration;

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(watch, RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                tracing::debug!("{:?}", event);
                if let DebouncedEvent::Write(_) = event {
                    if let Err(error) = convert(input, output, from.clone(), to.clone()) {
                        tracing::error!("Convert error: {:?}", error)
                    }
                }
            }
            Err(error) => tracing::error!("Watch error: {:?}", error),
        }
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Convert document from one format to another")]
    pub struct Args {
        input: String,

        output: String,

        #[structopt(short, long)]
        from: Option<String>,

        #[structopt(short, long)]
        to: Option<String>,

        #[structopt(short, long)]
        watch: bool,
    }

    pub fn run(args: Args) -> Result<()> {
        let Args {
            input,
            output,
            from,
            to,
            watch,
        } = args;

        if watch {
            super::convert_watch(&input, &output, from, to, &input)
        } else {
            super::convert(&input, &output, from, to)
        }
    }
}
