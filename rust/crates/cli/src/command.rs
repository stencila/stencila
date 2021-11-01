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
                // Use the result format or fallback to preferences passed in
                let formats = if let Some(format) = &value.format {
                    vec![format.clone()]
                } else {
                    formats.into()
                };

                if let Err(error) = result::print::value(value, &formats) {
                    result::print::error(error)
                }
            }
            Err(error) => result::print::error(error),
        }
    }
}
