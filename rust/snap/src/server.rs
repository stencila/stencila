//! Server discovery and URL resolution
//!
//! This module discovers running Stencila servers by reading their info files
//! from disk. It does not depend on `stencila-server` — the server mode field
//! is a plain string matched against well-known values. This avoids a cyclic
//! dependency (snap → server → attractor → agents → snap) and allows the
//! `stencila-agents` crate to use snap as a library.
//!
//! If no running server is found, `discover()` returns an error asking the
//! user to start one. The server crate serializes `ServerMode` with serde's
//! default enum representation, so the values on the wire are
//! `"DocumentPreview"` and `"SitePreview"`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, bail};
use serde::{Deserialize, Serialize};

use stencila_dirs::{DirType, get_app_dir};

/// Well-known server mode value for site preview servers
const MODE_SITE_PREVIEW: &str = "SitePreview";
/// Well-known server mode value for document preview servers
const MODE_DOCUMENT_PREVIEW: &str = "DocumentPreview";

/// Server runtime information (matches rust/server/src/server.rs)
///
/// The `mode` field is a plain string to avoid importing from `stencila-server`.
/// See the module-level docs for the rationale.
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

    /// The mode the server is running in (e.g. "DocumentPreview", "SitePreview")
    #[serde(default)]
    pub mode: String,
}

impl ServerInfo {
    /// Discover an active server for the given path
    ///
    /// When `prefer_site` is true, site preview servers are preferred over
    /// document servers (and vice versa). Falls back to any available server
    /// if the preferred type is not found.
    ///
    /// Returns an error if no running server is found — callers should ensure
    /// a server is running (e.g. via `stencila serve` or `stencila site preview`)
    /// before invoking snap.
    #[tracing::instrument]
    pub async fn discover(path: Option<&Path>, prefer_site: bool) -> Result<Self> {
        let servers_dir = get_app_dir(DirType::Servers, false)?;

        if !servers_dir.exists() {
            bail!(
                "No running Stencila server found (no servers directory). \
                 Start one with `stencila serve` or `stencila site preview`."
            );
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
            bail!(
                "No running Stencila server found. \
                 Start one with `stencila serve` or `stencila site preview`."
            );
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

        // Sort by preference: preferred mode first, then most recent
        let preferred_mode = if prefer_site {
            MODE_SITE_PREVIEW
        } else {
            MODE_DOCUMENT_PREVIEW
        };
        servers.sort_by(|a, b| {
            let a_preferred = a.mode == preferred_mode;
            let b_preferred = b.mode == preferred_mode;
            b_preferred
                .cmp(&a_preferred)
                .then_with(|| b.started_at.cmp(&a.started_at))
        });

        Ok(servers[0].clone())
    }

    /// Resolve a file path to a URL with authentication
    ///
    /// Converts a file path (relative or absolute) to a server URL with auth token.
    /// Passes the auth token as a query parameter directly on the target URL
    /// (rather than going through /~login redirect) so headless Chrome lands
    /// on the actual page immediately.
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

        // Construct URL with auth token as query parameter
        if let Some(token) = &self.token {
            Ok(format!(
                "http://127.0.0.1:{}/{}?sst={}",
                self.port, url_path, token
            ))
        } else {
            Ok(format!("http://127.0.0.1:{}/{}", self.port, url_path))
        }
    }

    /// Resolve a site route to a URL with authentication
    ///
    /// Takes a route like "/docs/guide/" and constructs the full server URL.
    /// Passes the auth token as a query parameter directly on the target URL
    /// (rather than going through /~login redirect) so headless Chrome lands
    /// on the actual page immediately.
    pub fn resolve_route(&self, route: &str) -> String {
        if let Some(token) = &self.token {
            let separator = if route.contains('?') { '&' } else { '?' };
            format!(
                "http://127.0.0.1:{}{route}{separator}sst={token}",
                self.port
            )
        } else {
            format!("http://127.0.0.1:{}{}", self.port, route)
        }
    }
}

/// Check if a process with the given PID is running
#[cfg(target_os = "linux")]
fn is_process_running(pid: u32) -> bool {
    let proc_path = format!("/proc/{}", pid);
    Path::new(&proc_path).exists()
}

/// Check if a process with the given PID is running
#[cfg(all(target_family = "unix", not(target_os = "linux")))]
#[allow(unsafe_code)]
fn is_process_running(pid: u32) -> bool {
    // On macOS and BSDs, /proc doesn't exist. Use kill(pid, 0) which
    // checks process existence without sending a signal. Returns 0 if
    // the process exists, or -1 with EPERM if it exists but we lack
    // permission to signal it — either way, the process is running.
    let ret = unsafe { libc::kill(pid as i32, 0) };
    if ret == 0 {
        return true;
    }
    // EPERM means the process exists but we can't signal it
    std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// Check if a process with the given PID is running
#[cfg(target_family = "windows")]
fn is_process_running(pid: u32) -> bool {
    use std::process::Command;

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

    #[test]
    fn test_resolve_route() {
        let info = ServerInfo {
            pid: 1234,
            port: 9000,
            token: Some("sst_testtoken".to_string()),
            directory: PathBuf::from("/tmp/test"),
            started_at: 0,
            mode: MODE_SITE_PREVIEW.to_string(),
        };

        assert_eq!(
            info.resolve_route("/docs/guide/"),
            "http://127.0.0.1:9000/docs/guide/?sst=sst_testtoken"
        );

        // Route with existing query string
        assert_eq!(
            info.resolve_route("/search?q=test"),
            "http://127.0.0.1:9000/search?q=test&sst=sst_testtoken"
        );

        let info_no_token = ServerInfo {
            token: None,
            ..info
        };
        assert_eq!(
            info_no_token.resolve_route("/docs/guide/"),
            "http://127.0.0.1:9000/docs/guide/"
        );
    }

    #[test]
    fn test_server_mode_serde() {
        // Test that mode defaults to empty string when missing from JSON
        let json = r#"{"pid":1,"port":9000,"token":null,"directory":"/tmp","started_at":0}"#;
        let info: ServerInfo = serde_json::from_str(json).expect("should parse without mode");
        assert_eq!(info.mode, "");

        // Test round-trip with explicit mode
        let json_with_mode = serde_json::to_string(&ServerInfo {
            pid: 1,
            port: 9000,
            token: None,
            directory: PathBuf::from("/tmp"),
            started_at: 0,
            mode: MODE_SITE_PREVIEW.to_string(),
        })
        .expect("should serialize");
        let info: ServerInfo =
            serde_json::from_str(&json_with_mode).expect("should parse with mode");
        assert_eq!(info.mode, MODE_SITE_PREVIEW);
    }
}
