use crate::{
    nodes::Node,
    serve::{generate_key, login_url, serve},
};
use anyhow::Result;

/// Serve JSON-RPC requests at a URL
///
/// # Arguments
///
/// - `url`: The file system path or URL to the stencil
///
/// # Examples
///
/// Open the stencil in the current working directory,
///
/// ```
/// use stencila::open::open;
/// open(None);
/// ```
pub async fn open(url: Option<String>) -> Result<()> {
    // URL defaults to the
    let url = url.unwrap_or_else(|| "".to_string());

    // Read the stencil from the URL

    // Store the stencil in an in memory ap of stencils

    // Determine the local path to the stencil

    // Generate a key and a login URL
    let key = generate_key();
    let login = login_url(&key, Some(url))?;

    // Open browser
    webbrowser::open(login.as_str())?;

    // Serve
    serve(None, Some(key)).await
}

/// CLI options for the `serve` command
#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;
    #[derive(Debug, StructOpt)]
    #[structopt(about = "Open a stencil")]
    pub struct Args {
        /// The file path or URL of the stencil to open
        #[structopt(default_value = ".")]
        url: String,
    }

    pub async fn open(args: Args) -> Result<()> {
        let Args { url } = args;

        super::open(Some(url)).await?;

        Ok(())
    }
}
