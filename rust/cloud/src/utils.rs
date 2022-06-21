use std::{env, fs::read_to_string, io::Write, path::PathBuf};

use cli_utils::{
    clap::{self, Parser},
    result,
};
use common::{
    dirs,
    eyre::{bail, Result},
    once_cell::sync::Lazy,
    regex::Regex,
    serde_json, tracing,
};
use fs_utils::open_file_600;

use crate::types::{ApiToken, User};

/// The base URL for Stencila Cloud
pub(crate) const BASE_URL: &str = if cfg!(debug_assertions) {
    "http://localhost:3000"
} else {
    "https://stencila.fly.dev"
};

#[macro_export]
macro_rules! api {
    ($template:expr $(, $par:expr)*) => {
        [crate::utils::BASE_URL, "/api/v1/", &format!($template $(, $par)*)].concat()
    };
}

#[macro_export]
macro_rules! page {
    ($template:expr $(, $par:expr)*) => {
        [crate::utils::BASE_URL, "/", &format!($template $(, $par)*)].concat()
    };
}

// An option that is reused in several subcommands in this crate to
// allow the user to open the corresponding web page on Stencila Cloud
#[derive(Parser)]
pub(crate) struct WebArg {
    /// Open the corresponding web page on Stencila in your browser
    ///
    /// Use this option when you want to quickly jump to the web page
    /// on Stencila that offers the same, or similar, functionality to this
    /// command.
    #[clap(long = "web", short = 'w')]
    pub yes: bool,
}

impl WebArg {
    pub fn open(&self, url: impl AsRef<str>) -> cli_utils::Result {
        let url = url.as_ref();
        tracing::info!("Opening web page in browser: {}", url);
        webbrowser::open(url)?;
        result::nothing()
    }
}

pub(crate) static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$",
    )
    .expect("Unable to created regex")
});

/// Get the path used to store `token.json`, `user.json`, and other files
/// associated with this crate
pub(crate) fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| env::current_dir().unwrap())
        .join("stencila")
}

/// Get the path of `token.json`
pub(crate) fn token_path() -> PathBuf {
    config_dir().join("token.json")
}

/// Read the current Stencila access token
pub(crate) fn token_read() -> Result<String> {
    let path = token_path();
    if path.exists() {
        let json = read_to_string(token_path())?;
        let token: ApiToken = serde_json::from_str(&json)?;
        Ok(token.token)
    } else {
        bail!("You are not logged in; try using `stencila login` first");
    }
}

pub(crate) fn token_write(token: &ApiToken) -> Result<()> {
    let json = serde_json::to_string_pretty(&token)?;
    let mut file = open_file_600(token_path())?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Get the path of `user.json`
pub(crate) fn user_path() -> PathBuf {
    config_dir().join("user.json")
}

/// Read the current Stencila user
pub(crate) fn user_read() -> Result<User> {
    let path = user_path();
    if path.exists() {
        let json = read_to_string(user_path())?;
        let user: User = serde_json::from_str(&json)?;
        Ok(user)
    } else {
        bail!("You are not logged in; try using `stencila login` first");
    }
}

/// Write the current Stencila user
pub(crate) fn user_write(user: &User) -> Result<()> {
    let json = serde_json::to_string_pretty(&user)?;
    let mut file = open_file_600(user_path())?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
