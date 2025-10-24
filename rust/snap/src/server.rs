//! Server discovery and URL resolution

use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use eyre::{OptionExt, Result, bail, eyre};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use stencila_dirs::{DirType, get_app_dir};

/// Server runtime information (matches rust/server/src/server.rs)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    /// Process ID of the server
    pub pid: u32,

    /// Port the server is listening on
    pub port: u16,

    /// Server authentication token
    pub token: Option<String>,

    /// Directory being served (absolute path)
    pub directory: PathBuf,

    /// Unix timestamp when server started
    pub started_at: u64,
}

/// Handle to a running server with shutdown control
pub struct ServerHandle {
    /// Server runtime information
    pub info: ServerInfo,

    /// Sender to trigger graceful shutdown (None for existing servers)
    shutdown_sender: Option<oneshot::Sender<()>>,

    /// Handle to the server task (None for existing servers)
    task_handle: Option<tokio::task::JoinHandle<Result<()>>>,
}

impl ServerHandle {
    /// Create a handle for a newly started server
    fn new(
        info: ServerInfo,
        shutdown_tx: oneshot::Sender<()>,
        task_handle: tokio::task::JoinHandle<Result<()>>,
    ) -> Self {
        Self {
            info,
            shutdown_sender: Some(shutdown_tx),
            task_handle: Some(task_handle),
        }
    }

    /// Create a handle for an existing server (discovered, not started by us)
    fn for_existing(info: ServerInfo) -> Self {
        Self {
            info,
            shutdown_sender: None,
            task_handle: None,
        }
    }

    /// Whether the server was started by this process
    pub fn is_in_process(&self) -> bool {
        self.shutdown_sender.is_some()
    }

    /// Trigger graceful shutdown and wait for completion
    ///
    /// Returns Ok(()) if shutdown was successful, or an error if:
    /// - This is a handle to an existing server (not started by us)
    /// - The shutdown signal couldn't be sent
    /// - The server task failed
    pub async fn shutdown(mut self) -> Result<()> {
        let shutdown_sender = self
            .shutdown_sender
            .take()
            .ok_or_eyre("Cannot shutdown existing server (not started by this handle)")?;

        let task_handle = self
            .task_handle
            .take()
            .ok_or_eyre("No task handle available")?;

        // Send shutdown signal
        shutdown_sender
            .send(())
            .map_err(|_| eyre!("Failed to send shutdown signal"))?;

        // Wait for server to finish
        task_handle.await??;

        Ok(())
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // Send shutdown signal when handle is dropped
        // Note: We don't wait for completion here as Drop can't be async
        if let Some(shutdown_tx) = self.shutdown_sender.take() {
            let _ = shutdown_tx.send(());
        }
    }
}

impl ServerInfo {
    /// Discover an active server for the given path, starting one if none exists
    ///
    /// Searches the runtime directory for server info files, filters by:
    /// - Valid PID (process still running)
    /// - Matching directory (path must be within server's served directory)
    /// - Sorted by most recent (started_at)
    ///
    /// If no suitable server is found, starts a new server in the background.
    ///
    /// Returns a `ServerHandle` which provides server information and shutdown control.
    /// For existing servers (not started by this call), the handle will not support
    /// graceful shutdown as we don't have control over those processes.
    #[tracing::instrument]
    pub async fn discover(path: Option<&Path>) -> Result<ServerHandle> {
        // Try to find an existing server
        if let Ok(server_info) = Self::find_existing(path) {
            return Ok(ServerHandle::for_existing(server_info));
        }

        // No suitable server found, start a new one
        Self::start_new(path).await
    }

    /// Find an existing running server
    #[tracing::instrument]
    fn find_existing(path: Option<&Path>) -> Result<Self> {
        let servers_dir = get_app_dir(DirType::Servers, false)?;

        if !servers_dir.exists() {
            bail!("No running servers found (no servers directory)");
        }

        // Read all server info files
        let entries = fs::read_dir(&servers_dir)?;
        let mut servers = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Try to parse server info
            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(info) = serde_json::from_str::<ServerInfo>(&content)
            {
                // Check if PID is still valid
                if is_process_running(info.pid) {
                    servers.push(info);
                }
            }
        }

        if servers.is_empty() {
            bail!("No running servers found");
        }

        // Filter by directory if path provided
        if let Some(target_path) = path {
            let target_canonical = target_path.canonicalize()?;
            servers.retain(|server| target_canonical.starts_with(&server.directory));

            if servers.is_empty() {
                bail!(
                    "No server found serving directory containing {}",
                    target_path.display()
                );
            }
        }

        // Sort by most recent
        servers.sort_by_key(|s| std::cmp::Reverse(s.started_at));

        Ok(servers[0].clone())
    }

    /// Start a new server in the background
    #[tracing::instrument]
    async fn start_new(path: Option<&Path>) -> Result<ServerHandle> {
        use stencila_server::{ServeOptions, get_server_token};

        // Determine the directory to serve
        let dir = if let Some(path) = path {
            let path = path.canonicalize()?;
            if path.is_dir() {
                path
            } else {
                path.parent()
                    .ok_or_eyre("File has no parent")?
                    .to_path_buf()
            }
        } else {
            current_dir()?
        };

        // Get (or generate) an access token
        let server_token = get_server_token();

        tracing::debug!("Starting server in dir: {}", dir.display());

        // Create shutdown channel for graceful shutdown control
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        // Spawn server in background
        let options = ServeOptions {
            dir,
            server_token: Some(server_token.clone()),
            no_startup_message: true,
            shutdown_receiver: Some(shutdown_receiver),
            ..Default::default()
        };
        let task_handle = tokio::spawn(async move {
            if let Err(error) = stencila_server::serve(options).await {
                tracing::error!("Background server failed: {error}");
                return Err(error);
            }
            Ok(())
        });

        // Wait for server info file to be written
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        let info = loop {
            if start.elapsed() > timeout {
                bail!("Timeout waiting for server to start");
            }

            // Try to find the newly started server
            if let Ok(server) = Self::find_existing(path) {
                break server;
            }

            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        Ok(ServerHandle::new(info, shutdown_sender, task_handle))
    }

    /// Resolve a path to a URL with authentication
    ///
    /// Converts a file path (relative or absolute) to a server URL with auth token
    #[tracing::instrument]
    pub fn resolve_url(&self, path: Option<PathBuf>) -> Result<String> {
        // Default to current directory if no path specified
        let path = path.unwrap_or_else(|| PathBuf::from("."));
        let canonical = path.canonicalize()?;

        // Check if path is within server directory
        if !canonical.starts_with(&self.directory) {
            bail!(
                "Path {} is not within server directory {}",
                canonical.display(),
                self.directory.display()
            );
        }

        // Get relative path from server root
        let rel_path = canonical.strip_prefix(&self.directory)?;

        // Convert path separators to forward slashes for URL
        let url_path = rel_path
            .to_str()
            .ok_or_else(|| eyre::eyre!("Path contains invalid UTF-8"))?
            .replace('\\', "/");

        // Construct URL with auth token
        if let Some(token) = &self.token {
            Ok(format!(
                "http://127.0.0.1:{}/~login?sst={}&next=/{}",
                self.port, token, url_path
            ))
        } else {
            Ok(format!("http://127.0.0.1:{}/{}", self.port, url_path))
        }
    }
}

/// Check if a process with the given PID is running
#[cfg(target_family = "unix")]
fn is_process_running(pid: u32) -> bool {
    let proc_path = format!("/proc/{}", pid);
    Path::new(&proc_path).exists()
}

/// Check if a process with the given PID is running
#[cfg(target_family = "windows")]
fn is_process_running(pid: u32) -> bool {
    use std::process::Command;

    // Use tasklist to check if process exists
    Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

/// Fallback for other platforms
#[cfg(not(any(target_family = "unix", target_family = "windows")))]
fn is_process_running(_pid: u32) -> bool {
    // Conservative approach: assume process might be running
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_process_running() {
        // Test with current process PID (should be running)
        let current_pid = std::process::id();
        assert!(is_process_running(current_pid));

        // Test with unlikely PID (probably not running)
        assert!(!is_process_running(999999));
    }
}
