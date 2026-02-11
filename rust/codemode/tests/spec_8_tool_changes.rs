//! Tests for §8: Tool list changes.
//!
//! Validates that dirty servers are refreshed before snapshot build (§8.1)
//! and that in-flight executions see a frozen tool set (§8.2).

mod common;

use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use stencila_codemode::{
    CodemodeError, DirtyServerTracker, McpContent, McpServer, McpToolInfo, McpToolResult, Sandbox,
};

// ---------------------------------------------------------------------------
// DynamicMockServer — tools change on refresh, tracks refresh calls
// ---------------------------------------------------------------------------

/// A mock server whose tool list can be swapped at runtime.
///
/// - `tools` is behind a `Mutex` so `refresh_tools()` can update it.
/// - `refresh_count` tracks how many times `refresh_tools()` was called.
/// - `pending_tools` holds the next tool list to install on refresh.
struct DynamicMockServer {
    id: String,
    name: String,
    tools: Mutex<Vec<McpToolInfo>>,
    pending_tools: Mutex<Option<Vec<McpToolInfo>>>,
    refresh_count: AtomicU32,
    list_changed: bool,
}

impl DynamicMockServer {
    fn new(id: &str, name: &str, tools: Vec<McpToolInfo>, list_changed: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tools: Mutex::new(tools),
            pending_tools: Mutex::new(None),
            refresh_count: AtomicU32::new(0),
            list_changed,
        }
    }

    /// Stage a new tool list that will be applied on the next `refresh_tools()` call.
    fn stage_tools(&self, tools: Vec<McpToolInfo>) {
        let mut pending = self.pending_tools.lock().expect("pending_tools lock");
        *pending = Some(tools);
    }

    /// How many times `refresh_tools()` has been called.
    fn refresh_count(&self) -> u32 {
        self.refresh_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl McpServer for DynamicMockServer {
    fn server_id(&self) -> &str {
        &self.id
    }

    fn server_name(&self) -> &str {
        &self.name
    }

    async fn tools(&self) -> Result<Vec<McpToolInfo>, CodemodeError> {
        let tools = self.tools.lock().expect("tools lock");
        Ok(tools.clone())
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        _input: serde_json::Value,
    ) -> Result<McpToolResult, CodemodeError> {
        Ok(McpToolResult {
            content: vec![McpContent::Text {
                text: format!("Called {tool_name}"),
            }],
            structured_content: None,
            is_error: false,
        })
    }

    fn supports_list_changed(&self) -> bool {
        self.list_changed
    }

    async fn refresh_tools(&self) -> Result<(), CodemodeError> {
        self.refresh_count.fetch_add(1, Ordering::SeqCst);
        let mut pending = self.pending_tools.lock().expect("pending_tools lock");
        if let Some(new_tools) = pending.take() {
            let mut tools = self.tools.lock().expect("tools lock");
            *tools = new_tools;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// JS code that lists tool names for a given server via discovery.
fn list_tool_names_js(server_id: &str) -> String {
    format!(
        r#"
        import {{ listTools }} from "@codemode/discovery";
        const tools = await listTools("{server_id}", {{ detail: "name" }});
        globalThis.__codemode_result__ = tools.map(t => t.toolName);
    "#
    )
}

/// Build a `HashSet` containing a single dirty server ID.
fn dirty_set(server_id: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    set.insert(server_id.to_string());
    set
}

// ---------------------------------------------------------------------------
// §8.1 — Refresh timing
// ---------------------------------------------------------------------------

/// §8.1: Dirty servers that support listChanged are refreshed before snapshot build.
/// After refresh, the new tools are visible in the sandbox.
#[tokio::test]
async fn dirty_server_tools_refreshed_before_import() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(DynamicMockServer::new(
        "dynamic",
        "Dynamic Server",
        vec![common::simple_tool("alpha", "First tool")],
        true, // supports listChanged
    ));

    // First invocation sees "alpha"
    let sandbox = Sandbox::new(None, &[server.clone() as Arc<dyn McpServer>]).await?;
    let resp = sandbox.execute(&list_tool_names_js("dynamic")).await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!(["alpha"]));

    // Stage new tools and mark dirty
    server.stage_tools(vec![
        common::simple_tool("alpha", "First tool"),
        common::simple_tool("beta", "Second tool"),
    ]);

    // Second invocation with dirty set → refresh_tools() called → sees both tools
    let dirty = dirty_set("dynamic");
    let sandbox =
        Sandbox::with_dirty_servers(None, &[server.clone() as Arc<dyn McpServer>], &dirty).await?;
    let resp = sandbox.execute(&list_tool_names_js("dynamic")).await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!(["alpha", "beta"]));
    assert_eq!(server.refresh_count(), 1);

    Ok(())
}

/// §8.1: refresh_tools() is NOT called when the server is not in the dirty set.
#[tokio::test]
async fn clean_server_not_refreshed() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(DynamicMockServer::new(
        "dynamic",
        "Dynamic Server",
        vec![common::simple_tool("alpha", "First tool")],
        true,
    ));

    // Stage new tools but do NOT mark dirty
    server.stage_tools(vec![common::simple_tool("beta", "New tool")]);

    let sandbox = Sandbox::new(None, &[server.clone() as Arc<dyn McpServer>]).await?;
    let resp = sandbox.execute(&list_tool_names_js("dynamic")).await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    // Still sees "alpha" because refresh was not called
    assert_eq!(resp.result, serde_json::json!(["alpha"]));
    assert_eq!(server.refresh_count(), 0);

    Ok(())
}

/// §8.1: Servers that do NOT support listChanged are never refreshed, even if dirty.
#[tokio::test]
async fn server_without_list_changed_not_refreshed() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(DynamicMockServer::new(
        "static-server",
        "Static Server",
        vec![common::simple_tool("alpha", "First tool")],
        false, // does NOT support listChanged
    ));

    server.stage_tools(vec![common::simple_tool("beta", "New tool")]);

    let dirty = dirty_set("static-server");
    let sandbox =
        Sandbox::with_dirty_servers(None, &[server.clone() as Arc<dyn McpServer>], &dirty).await?;
    let resp = sandbox.execute(&list_tool_names_js("static-server")).await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!(["alpha"]));
    assert_eq!(server.refresh_count(), 0);

    Ok(())
}

// ---------------------------------------------------------------------------
// §8.2 — In-flight executions
// ---------------------------------------------------------------------------

/// §8.2: Tool snapshot is frozen at sandbox creation — changes after creation
/// are not visible to that sandbox's execution.
#[tokio::test]
async fn inflight_execution_sees_frozen_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(DynamicMockServer::new(
        "dynamic",
        "Dynamic Server",
        vec![common::simple_tool("alpha", "First tool")],
        true,
    ));

    // Create the sandbox (snapshot frozen here)
    let sandbox = Sandbox::new(None, &[server.clone() as Arc<dyn McpServer>]).await?;

    // After sandbox creation, stage new tools (simulating tools.listChanged mid-flight)
    server.stage_tools(vec![
        common::simple_tool("alpha", "First tool"),
        common::simple_tool("gamma", "Third tool"),
    ]);
    // Even manually applying the refresh won't affect the already-created sandbox
    server.refresh_tools().await?;

    // Execute — should still see only "alpha" (frozen snapshot)
    let resp = sandbox.execute(&list_tool_names_js("dynamic")).await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!(["alpha"]));

    Ok(())
}

/// §8.2: The updated tool set becomes visible on the next invocation.
#[tokio::test]
async fn updated_tools_visible_on_next_invocation() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(DynamicMockServer::new(
        "dynamic",
        "Dynamic Server",
        vec![common::simple_tool("v1_tool", "Version 1")],
        true,
    ));

    // Invocation 1
    let sandbox = Sandbox::new(None, &[server.clone() as Arc<dyn McpServer>]).await?;
    let resp = sandbox.execute(&list_tool_names_js("dynamic")).await;
    assert_eq!(resp.result, serde_json::json!(["v1_tool"]));

    // Server announces change: stage new tools
    server.stage_tools(vec![common::simple_tool("v2_tool", "Version 2")]);

    // Invocation 2 (with dirty)
    let dirty = dirty_set("dynamic");
    let sandbox =
        Sandbox::with_dirty_servers(None, &[server.clone() as Arc<dyn McpServer>], &dirty).await?;
    let resp = sandbox.execute(&list_tool_names_js("dynamic")).await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!(["v2_tool"]));

    Ok(())
}

/// §8.2: Calling a tool that was added via refresh works in the new sandbox.
#[tokio::test]
async fn refreshed_tool_is_callable() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(DynamicMockServer::new(
        "dynamic",
        "Dynamic Server",
        vec![common::simple_tool("alpha", "First tool")],
        true,
    ));

    // Stage new tool
    server.stage_tools(vec![
        common::simple_tool("alpha", "First tool"),
        common::simple_tool("beta", "Second tool"),
    ]);

    let dirty = dirty_set("dynamic");
    let sandbox =
        Sandbox::with_dirty_servers(None, &[server.clone() as Arc<dyn McpServer>], &dirty).await?;
    let resp = sandbox
        .execute(
            r#"
        import { beta } from "@codemode/servers/dynamic";
        const result = await beta();
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!("Called beta"));

    Ok(())
}

// ---------------------------------------------------------------------------
// DirtyServerTracker unit tests
// ---------------------------------------------------------------------------

/// DirtyServerTracker: mark_changed adds server, take_dirty returns and clears.
#[test]
fn dirty_tracker_mark_and_take() {
    let mut tracker = DirtyServerTracker::new();
    assert!(!tracker.has_dirty());

    tracker.mark_changed("server-a");
    tracker.mark_changed("server-b");
    assert!(tracker.has_dirty());

    let dirty = tracker.take_dirty();
    assert!(dirty.contains("server-a"));
    assert!(dirty.contains("server-b"));
    assert_eq!(dirty.len(), 2);

    // After take, tracker is empty
    assert!(!tracker.has_dirty());
    let empty = tracker.take_dirty();
    assert!(empty.is_empty());
}

/// DirtyServerTracker: duplicate marks are idempotent.
#[test]
fn dirty_tracker_duplicate_marks_idempotent() {
    let mut tracker = DirtyServerTracker::new();
    tracker.mark_changed("server-a");
    tracker.mark_changed("server-a");

    let dirty = tracker.take_dirty();
    assert_eq!(dirty.len(), 1);
}

/// DirtyServerTracker: dirty() borrows without consuming; clear() resets.
#[test]
fn dirty_tracker_borrow_and_clear() {
    let mut tracker = DirtyServerTracker::new();
    tracker.mark_changed("server-a");

    // dirty() borrows — marks preserved
    assert!(tracker.dirty().contains("server-a"));
    assert!(tracker.has_dirty());

    // clear() resets
    tracker.clear();
    assert!(!tracker.has_dirty());
    assert!(tracker.dirty().is_empty());
}

/// DirtyServerTracker integrates with Sandbox::with_dirty_servers using dirty() borrow.
#[tokio::test]
async fn dirty_tracker_integration() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(DynamicMockServer::new(
        "tracked",
        "Tracked Server",
        vec![common::simple_tool("original", "Original tool")],
        true,
    ));

    // Stage new tools and use tracker
    server.stage_tools(vec![common::simple_tool("updated", "Updated tool")]);

    let mut tracker = DirtyServerTracker::new();
    tracker.mark_changed("tracked");

    // Use dirty() (borrow) so marks survive if with_dirty_servers fails
    let sandbox = Sandbox::with_dirty_servers(
        None,
        &[server.clone() as Arc<dyn McpServer>],
        tracker.dirty(),
    )
    .await?;

    // Clear after successful creation
    tracker.clear();

    let resp = sandbox.execute(&list_tool_names_js("tracked")).await;
    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!(["updated"]));
    assert_eq!(server.refresh_count(), 1);

    // Tracker is now empty — no unnecessary refreshes
    assert!(!tracker.has_dirty());

    Ok(())
}
