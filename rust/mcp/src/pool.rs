//! MCP server connection pool.
//!
//! [`ConnectionPool`] manages a set of MCP server connections with lazy
//! initialization, automatic reconnection, and coordinated shutdown.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tokio::sync::RwLock;

use crate::{McpError, McpResult, config::McpServerConfig, server::LiveMcpServer};

/// Hard timeout for graceful pool shutdown.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// A pool of MCP server connections.
///
/// Servers are connected lazily on first access. If a previously connected
/// server has disconnected, [`get`](Self::get) will attempt to reconnect it.
///
/// Use [`start_shutdown`](Self::start_shutdown) for fire-and-forget cleanup
/// from synchronous `close()` methods.
pub struct ConnectionPool {
    /// Server configurations indexed by ID.
    configs: HashMap<String, McpServerConfig>,

    /// Connected servers (lazily populated).
    servers: RwLock<HashMap<String, Arc<LiveMcpServer>>>,

    /// Whether shutdown has been initiated.
    shutting_down: AtomicBool,
}

impl ConnectionPool {
    /// Create a new pool from discovered server configurations.
    ///
    /// Only enabled servers are included. No connections are established
    /// until [`get`](Self::get) or [`connect_all`](Self::connect_all) is called.
    #[must_use]
    pub fn new(configs: Vec<McpServerConfig>) -> Self {
        let configs = configs
            .into_iter()
            .filter(|c| c.enabled)
            .map(|c| (c.id.clone(), c))
            .collect();

        Self {
            configs,
            servers: RwLock::new(HashMap::new()),
            shutting_down: AtomicBool::new(false),
        }
    }

    /// The IDs of all configured (enabled) servers.
    #[must_use]
    pub fn server_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.configs.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Get a connected server by ID, connecting lazily if needed.
    ///
    /// If the server was previously connected but has disconnected,
    /// a reconnection is attempted.
    ///
    /// # Errors
    ///
    /// Returns [`McpError::ServerNotFound`] if no config exists for the ID
    /// or the pool is shutting down. Returns connection errors if the server
    /// cannot be reached.
    pub async fn get(&self, server_id: &str) -> McpResult<Arc<LiveMcpServer>> {
        if self.shutting_down.load(Ordering::SeqCst) {
            return Err(McpError::ServerNotFound {
                server_id: server_id.into(),
            });
        }

        // Fast path: already connected
        {
            let servers = self.servers.read().await;
            if let Some(server) = servers.get(server_id)
                && server.is_connected()
            {
                return Ok(Arc::clone(server));
            }
            // Disconnected — fall through to reconnect
        }

        self.connect_server(server_id).await
    }

    /// Connect all configured servers.
    ///
    /// Returns a list of `(server_id, error)` for servers that failed to
    /// connect. Successfully connected servers are available via [`get`](Self::get).
    pub async fn connect_all(&self) -> Vec<(String, McpError)> {
        let ids = self.server_ids();
        let mut errors = Vec::new();

        for id in ids {
            if let Err(e) = self.connect_server(&id).await {
                errors.push((id, e));
            }
        }

        errors
    }

    /// All currently connected servers.
    pub async fn connected_servers(&self) -> Vec<Arc<LiveMcpServer>> {
        let servers = self.servers.read().await;
        servers
            .values()
            .filter(|s| s.is_connected())
            .map(Arc::clone)
            .collect()
    }

    /// Number of configured (enabled) servers.
    #[must_use]
    pub fn config_count(&self) -> usize {
        self.configs.len()
    }

    /// Number of currently connected servers.
    pub async fn connected_count(&self) -> usize {
        let servers = self.servers.read().await;
        servers.values().filter(|s| s.is_connected()).count()
    }

    /// Start asynchronous shutdown of all connections.
    ///
    /// Returns immediately. A background task gracefully shuts down all
    /// servers, with a hard timeout of 5 seconds before force-killing.
    ///
    /// Safe to call multiple times — subsequent calls are no-ops.
    pub fn start_shutdown(self: &Arc<Self>) {
        if self.shutting_down.swap(true, Ordering::SeqCst) {
            return; // Already shutting down
        }

        let pool = Arc::clone(self);
        tokio::spawn(async move {
            pool.shutdown_all().await;
        });
    }

    /// Connect (or reconnect) a single server by ID.
    ///
    /// Uses double-checked locking: after the (expensive) connect completes,
    /// re-checks under the write lock whether another task already connected
    /// the same server, and whether shutdown was initiated in the meantime.
    async fn connect_server(&self, server_id: &str) -> McpResult<Arc<LiveMcpServer>> {
        let config = self
            .configs
            .get(server_id)
            .ok_or_else(|| McpError::ServerNotFound {
                server_id: server_id.into(),
            })?;

        tracing::debug!("Connecting to MCP server `{server_id}`");

        // Connect outside any lock (expensive: spawns process, handshake).
        let server = LiveMcpServer::connect(config.clone()).await?;
        let server = Arc::new(server);

        // Re-check under write lock before inserting.
        let mut servers = self.servers.write().await;

        // Guard: if shutdown started while we were connecting, clean up
        // the new connection so it doesn't leak.
        if self.shutting_down.load(Ordering::SeqCst) {
            drop(servers);
            let _ = server.shutdown().await;
            return Err(McpError::ServerNotFound {
                server_id: server_id.into(),
            });
        }

        // Guard: another task may have connected this server concurrently.
        // If so, shut down our duplicate and return the existing one.
        if let Some(existing) = servers.get(server_id)
            && existing.is_connected()
        {
            let duplicate = Arc::clone(&server);
            drop(servers);
            tokio::spawn(async move {
                let _ = duplicate.shutdown().await;
            });
            tracing::debug!(
                "MCP server `{server_id}` already connected by another task, discarding duplicate"
            );
            // Re-read to return the existing handle
            let servers = self.servers.read().await;
            return servers.get(server_id).map(Arc::clone).ok_or_else(|| {
                McpError::ServerNotFound {
                    server_id: server_id.into(),
                }
            });
        }

        servers.insert(server_id.to_string(), Arc::clone(&server));

        tracing::debug!("Connected to MCP server `{server_id}`");

        Ok(server)
    }

    /// Shut down all connected servers with a hard timeout.
    async fn shutdown_all(&self) {
        let servers: Vec<(String, Arc<LiveMcpServer>)> = {
            let mut map = self.servers.write().await;
            map.drain().collect()
        };

        if servers.is_empty() {
            return;
        }

        tracing::debug!("Shutting down {} MCP server(s)", servers.len());

        let mut set = tokio::task::JoinSet::new();
        for (id, server) in servers {
            set.spawn(async move {
                if let Err(e) = server.shutdown().await {
                    tracing::warn!("Error shutting down MCP server `{id}`: {e}");
                }
            });
        }

        // Wait with hard timeout
        let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, async {
            while set.join_next().await.is_some() {}
        })
        .await;

        tracing::debug!("MCP server pool shutdown complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::McpServer as _;
    use crate::config::TransportConfig;

    /// Helper: create a config for a Python mock MCP server.
    fn mock_config(id: &str, enabled: bool) -> McpServerConfig {
        let script = r#"
import sys, json
for line in sys.stdin:
    req = json.loads(line.strip())
    if req.get("id") is None:
        continue
    if req["method"] == "initialize":
        msg = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "serverInfo": {"name": "pool-mock", "version": "1.0"}
        }})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
    elif req["method"] == "tools/list":
        msg = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {
            "tools": [{"name": "echo", "description": "Echo input"}]
        }})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
    elif req["method"] == "tools/call":
        text = json.dumps(req["params"].get("arguments", {}))
        msg = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {
            "content": [{"type": "text", "text": text}],
            "isError": False
        }})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
"#;
        McpServerConfig {
            id: id.into(),
            name: None,
            transport: TransportConfig::Stdio {
                command: "python3".into(),
                args: vec!["-c".into(), script.into()],
            },
            env: HashMap::new(),
            enabled,
            source: None,
        }
    }

    #[test]
    fn new_filters_disabled_servers() {
        let configs = vec![
            mock_config("enabled-1", true),
            mock_config("disabled-1", false),
            mock_config("enabled-2", true),
        ];

        let pool = ConnectionPool::new(configs);
        assert_eq!(pool.config_count(), 2);

        let ids = pool.server_ids();
        assert!(ids.contains(&"enabled-1".to_string()));
        assert!(ids.contains(&"enabled-2".to_string()));
        assert!(!ids.contains(&"disabled-1".to_string()));
    }

    #[tokio::test]
    async fn lazy_connect_on_get() -> McpResult<()> {
        let pool = ConnectionPool::new(vec![mock_config("server-a", true)]);

        assert_eq!(pool.connected_count().await, 0);

        let server = pool.get("server-a").await?;
        assert_eq!(server.server_id(), "server-a");
        assert_eq!(pool.connected_count().await, 1);

        // Second get returns the same connection
        let server2 = pool.get("server-a").await?;
        assert_eq!(server2.server_id(), "server-a");
        assert_eq!(pool.connected_count().await, 1);

        // Cleanup
        let pool = Arc::new(pool);
        pool.start_shutdown();
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }

    #[tokio::test]
    async fn get_nonexistent_server_returns_not_found() {
        let pool = ConnectionPool::new(vec![]);

        let Err(err) = pool.get("nonexistent").await else {
            panic!("expected ServerNotFound error");
        };
        assert!(
            matches!(err, McpError::ServerNotFound { .. }),
            "expected ServerNotFound, got {err:?}"
        );
    }

    #[tokio::test]
    async fn connect_all_reports_errors() {
        let configs = vec![
            mock_config("good", true),
            McpServerConfig {
                id: "bad".into(),
                name: None,
                transport: TransportConfig::Stdio {
                    command: "nonexistent-binary-xyz".into(),
                    args: vec![],
                },
                env: HashMap::new(),
                enabled: true,
                source: None,
            },
        ];

        let pool = ConnectionPool::new(configs);
        let errors = pool.connect_all().await;

        // "good" should connect, "bad" should fail
        assert_eq!(pool.connected_count().await, 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "bad");

        // Cleanup
        let pool = Arc::new(pool);
        pool.start_shutdown();
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    #[tokio::test]
    async fn connected_servers_returns_only_active() -> McpResult<()> {
        let pool = ConnectionPool::new(vec![mock_config("s1", true), mock_config("s2", true)]);

        // Nothing connected yet
        assert!(pool.connected_servers().await.is_empty());

        // Connect one
        let _ = pool.get("s1").await?;
        let active = pool.connected_servers().await;
        assert_eq!(active.len(), 1);

        // Connect both
        let _ = pool.get("s2").await?;
        let active = pool.connected_servers().await;
        assert_eq!(active.len(), 2);

        // Cleanup
        let pool = Arc::new(pool);
        pool.start_shutdown();
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }

    #[tokio::test]
    async fn start_shutdown_prevents_new_connections() -> McpResult<()> {
        let pool = Arc::new(ConnectionPool::new(vec![mock_config("srv", true)]));

        pool.start_shutdown();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let Err(err) = pool.get("srv").await else {
            panic!("expected ServerNotFound after shutdown");
        };
        assert!(
            matches!(err, McpError::ServerNotFound { .. }),
            "expected ServerNotFound after shutdown, got {err:?}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn start_shutdown_is_idempotent() {
        let pool = Arc::new(ConnectionPool::new(vec![]));

        pool.start_shutdown();
        pool.start_shutdown(); // Should not panic
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn reconnect_on_disconnect() -> McpResult<()> {
        // Server that exits after handling initialize + one tools/list
        let script = r#"
import sys, json
count = 0
for line in sys.stdin:
    req = json.loads(line.strip())
    if req.get("id") is None:
        continue
    if req["method"] == "initialize":
        msg = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "serverInfo": {"name": "short-lived"}
        }})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
    elif req["method"] == "tools/list":
        count += 1
        msg = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {
            "tools": [{"name": f"tool_v{count}"}]
        }})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
        if count >= 1:
            sys.exit(0)
"#;
        let config = McpServerConfig {
            id: "reconnect-test".into(),
            name: None,
            transport: TransportConfig::Stdio {
                command: "python3".into(),
                args: vec!["-c".into(), script.into()],
            },
            env: HashMap::new(),
            enabled: true,
            source: None,
        };

        let pool = ConnectionPool::new(vec![config]);

        // First connection
        let server = pool.get("reconnect-test").await?;
        let tools = server.tools().await?;
        assert_eq!(tools[0].name, "tool_v1");

        // Wait for the server to exit
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Server should be disconnected now — get() should reconnect
        let server2 = pool.get("reconnect-test").await?;
        assert!(server2.is_connected());
        let tools2 = server2.tools().await?;
        assert_eq!(tools2[0].name, "tool_v1"); // New process starts with count=0 again

        // Cleanup
        let pool = Arc::new(pool);
        pool.start_shutdown();
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }
}
