use crate::{result, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Run {
    /// Run the command
    async fn run(&self) -> Result;

    /// Run the command and print it to the console
    async fn print(&self) {
        let result = self.run().await;

        // TODO
        // Use current display format or fallback to configured preferences
        //let formats = if let Some(display) = display {
        //    vec![display]
        //} else {
        //    formats.into()
        //};
        let formats = &["".to_string()];

        if let Err(error) = match result {
            Ok(value) => result::print::value(value, formats),
            Err(error) => result::print::error(error),
        } {
            eprintln!("Error printing result: {}", error)
        }
    }
}
