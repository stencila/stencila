use std::{collections::HashMap, fs, path::PathBuf};

use kernel::common::{
    defaults::Defaults,
    eyre::Result,
    glob,
    serde::{Deserialize, Serialize},
    serde_json,
    serde_with::skip_serializing_none,
    tracing,
};

use crate::dirs::runtime_dirs;

/// A Jupyter server
///
/// Used to access information about currently running kernels so that they
/// can be associated with notebook files and connected to if necessary.
#[skip_serializing_none]
#[derive(Debug, Defaults, Deserialize, Serialize)]
#[serde(default, crate = "kernel::common::serde")]
pub struct JupyterServer {
    pub(crate) base_url: String,
    pub(crate) hostname: String,
    pub(crate) notebook_dir: PathBuf,
    pub(crate) password: bool,
    pub(crate) pid: u32,
    pub(crate) port: u32,
    pub(crate) secure: bool,
    pub(crate) sock: String,
    pub(crate) token: String,
    pub(crate) url: String,
}

impl JupyterServer {
    /// Get a list of running Jupyter servers
    ///
    /// Scans the Jupyter runtime directory for `nbserver-*.json` files and
    /// checks that they are running by requesting from the URL with the token.
    /// This avoids issues with "zombie" `nbserver-*.json` files.
    pub async fn running() -> Result<HashMap<String, JupyterServer>> {
        let pattern = runtime_dirs()
            .first()
            .expect("Should always be at least one runtime directory")
            .join("nbserver-*.json")
            .to_string_lossy()
            .to_string();

        let files = glob::glob(&pattern)?.flatten();

        let client = reqwest::Client::new();

        let mut map = HashMap::new();
        for entry in files {
            let json = fs::read_to_string(entry)?;
            let server: JupyterServer = serde_json::from_str(&json)?;

            let url = format!("{}api/status?token={}", server.url, server.token);
            match client.get(url).send().await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::FORBIDDEN {
                        tracing::debug!("Unable to authenticate with Jupyter server; skipping");
                        continue;
                    }
                }
                Err(..) => {
                    tracing::debug!("Unable to send request to Jupyter server; skipping");
                    continue;
                }
            };

            map.insert(server.url.clone(), server);
        }

        Ok(map)
    }
}
