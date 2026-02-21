mod common;

use std::sync::Arc;

use stencila_codemode::{McpServer, Sandbox};

use common::{MockServer, sandbox_with_servers, simple_tool, tool_with_schema};

/// Helper: create a pair of mock servers for multi-server tests.
fn two_servers() -> Vec<Arc<dyn McpServer>> {
    vec![
        Arc::new(MockServer::with_description(
            "files",
            "File Server",
            "A file management server",
            "1.2.0",
            vec![
                simple_tool("readFile", "Read a file from disk"),
                tool_with_schema("search", "Search for files by query"),
            ],
        )),
        Arc::new(MockServer::new(
            "database",
            "Database Server",
            vec![
                simple_tool("query", "Run a database query"),
                simple_tool("insert", "Insert a record"),
            ],
        )),
    ]
}

// ============================================================
// §12.1 — specVersion
// ============================================================

#[tokio::test]
async fn spec_version_is_semver_string() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { specVersion } from "@codemode/discovery";
        globalThis.__codemode_result__ = specVersion;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let version = response
        .result
        .as_str()
        .expect("specVersion should be a string");
    // Should be a semver string like "0.1.0"
    assert!(
        version.split('.').count() == 3,
        "specVersion should be semver, got: {version}"
    );
}

// ============================================================
// §4.1 — listServers()
// ============================================================

#[tokio::test]
async fn list_servers_returns_all_servers() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { listServers } from "@codemode/discovery";
        const servers = await listServers();
        globalThis.__codemode_result__ = servers;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert!(result.is_array(), "should return an array");
    let arr = result.as_array().expect("array");
    assert_eq!(arr.len(), 2);

    // Check first server
    assert_eq!(arr[0]["serverId"], "files");
    assert_eq!(arr[0]["serverName"], "File Server");

    // Check second server
    assert_eq!(arr[1]["serverId"], "database");
    assert_eq!(arr[1]["serverName"], "Database Server");
}

#[tokio::test]
async fn list_servers_empty_when_no_servers() {
    let sandbox = Sandbox::new(None, &[]).await.expect("sandbox");
    let response = sandbox
        .execute(
            r#"
        import { listServers } from "@codemode/discovery";
        const servers = await listServers();
        globalThis.__codemode_result__ = servers.length;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, serde_json::json!(0));
}

// ============================================================
// §4.1 — describeServer()
// ============================================================

#[tokio::test]
async fn describe_server_returns_full_info() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { describeServer } from "@codemode/discovery";
        const desc = await describeServer("files");
        globalThis.__codemode_result__ = desc;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert_eq!(result["serverId"], "files");
    assert_eq!(result["serverName"], "File Server");
    assert_eq!(result["description"], "A file management server");
    assert_eq!(result["version"], "1.2.0");
}

#[tokio::test]
async fn describe_server_unknown_throws() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { describeServer } from "@codemode/discovery";
        import { ServerNotFoundError, CodemodeError } from "@codemode/errors";
        try {
            await describeServer("nonexistent");
            globalThis.__codemode_result__ = "should not reach";
        } catch (e) {
            globalThis.__codemode_result__ = {
                message: e.message,
                name: e.name,
                isServerNotFoundError: e instanceof ServerNotFoundError,
                isCodemodeError: e instanceof CodemodeError,
                serverId: e.serverId,
                hint: e.hint,
            };
        }
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert_eq!(result["isServerNotFoundError"], true);
    assert_eq!(result["isCodemodeError"], true);
    assert_eq!(result["name"], "ServerNotFoundError");
    assert_eq!(result["serverId"], "nonexistent");
    assert!(
        result["hint"]
            .as_str()
            .unwrap_or("")
            .contains("listServers")
    );
}

// ============================================================
// §4.1 / §4.3 — listTools() with detail levels
// ============================================================

#[tokio::test]
async fn list_tools_description_detail() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { listTools } from "@codemode/discovery";
        const tools = await listTools("files");
        globalThis.__codemode_result__ = tools;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    let arr = result.as_array().expect("array");
    assert_eq!(arr.len(), 2);

    // Default detail is "description" — should have description but no inputSchema
    assert_eq!(arr[0]["toolName"], "readFile");
    assert_eq!(arr[0]["exportName"], "readFile");
    assert_eq!(arr[0]["description"], "Read a file from disk");
    assert!(arr[0].get("inputSchema").is_none() || arr[0]["inputSchema"].is_null());
}

#[tokio::test]
async fn list_tools_name_detail() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { listTools } from "@codemode/discovery";
        const tools = await listTools("files", { detail: "name" });
        globalThis.__codemode_result__ = tools;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert_eq!(arr[0]["toolName"], "readFile");
    assert_eq!(arr[0]["exportName"], "readFile");
    // Name detail should omit description
    assert!(arr[0].get("description").is_none() || arr[0]["description"].is_null());
}

#[tokio::test]
async fn list_tools_full_detail() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { listTools } from "@codemode/discovery";
        const tools = await listTools("files", { detail: "full" });
        globalThis.__codemode_result__ = tools;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");

    // "search" tool should have inputSchema at full detail
    let search_tool = arr
        .iter()
        .find(|t| t["toolName"] == "search")
        .expect("search tool");
    assert!(
        search_tool["inputSchema"].is_object(),
        "full detail should include inputSchema"
    );
    assert!(
        search_tool["annotations"].is_object(),
        "full detail should include annotations"
    );
}

#[tokio::test]
async fn list_tools_unknown_server_throws() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { listTools } from "@codemode/discovery";
        import { ServerNotFoundError } from "@codemode/errors";
        try {
            await listTools("nonexistent");
            globalThis.__codemode_result__ = "should not reach";
        } catch (e) {
            globalThis.__codemode_result__ = {
                isServerNotFoundError: e instanceof ServerNotFoundError,
                name: e.name,
                serverId: e.serverId,
            };
        }
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert_eq!(result["isServerNotFoundError"], true);
    assert_eq!(result["name"], "ServerNotFoundError");
    assert_eq!(result["serverId"], "nonexistent");
}

// ============================================================
// §4.1 — getTool()
// ============================================================

#[tokio::test]
async fn get_tool_returns_full_definition() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { getTool } from "@codemode/discovery";
        const tool = await getTool("files", "search");
        globalThis.__codemode_result__ = tool;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert_eq!(result["toolName"], "search");
    assert_eq!(result["exportName"], "search");
    assert_eq!(result["description"], "Search for files by query");
    assert!(result["inputSchema"].is_object());
}

#[tokio::test]
async fn get_tool_unknown_throws() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { getTool } from "@codemode/discovery";
        import { ToolNotFoundError } from "@codemode/errors";
        try {
            await getTool("files", "nonexistent");
            globalThis.__codemode_result__ = "should not reach";
        } catch (e) {
            globalThis.__codemode_result__ = {
                isToolNotFoundError: e instanceof ToolNotFoundError,
                name: e.name,
                serverId: e.serverId,
                toolName: e.toolName,
                hint: e.hint,
            };
        }
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert_eq!(result["isToolNotFoundError"], true);
    assert_eq!(result["name"], "ToolNotFoundError");
    assert_eq!(result["serverId"], "files");
    assert_eq!(result["toolName"], "nonexistent");
    assert!(result["hint"].as_str().unwrap_or("").contains("listTools"));
}

#[tokio::test]
async fn get_tool_unknown_server_throws_server_not_found() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { getTool } from "@codemode/discovery";
        import { ServerNotFoundError } from "@codemode/errors";
        try {
            await getTool("nonexistent", "ping");
            globalThis.__codemode_result__ = "should not reach";
        } catch (e) {
            globalThis.__codemode_result__ = {
                isServerNotFoundError: e instanceof ServerNotFoundError,
                name: e.name,
                serverId: e.serverId,
            };
        }
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert_eq!(result["isServerNotFoundError"], true);
    assert_eq!(result["name"], "ServerNotFoundError");
    assert_eq!(result["serverId"], "nonexistent");
}

// ============================================================
// §4.1 — searchTools()
// ============================================================

#[tokio::test]
async fn search_tools_substring_match() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { searchTools } from "@codemode/discovery";
        const results = await searchTools("read");
        globalThis.__codemode_result__ = results;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let result = &response.result;
    assert_eq!(result["query"], "read");
    let results = result["results"].as_array().expect("results array");
    assert!(!results.is_empty(), "should find at least one match");
    // readFile should match "read" substring
    assert!(
        results.iter().any(|r| r["toolName"] == "readFile"),
        "readFile should match 'read'"
    );
}

#[tokio::test]
async fn search_tools_no_match() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { searchTools } from "@codemode/discovery";
        const results = await searchTools("zzzznonexistent");
        globalThis.__codemode_result__ = results.results.length;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, serde_json::json!(0));
}

#[tokio::test]
async fn search_tools_filtered_by_server() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { searchTools } from "@codemode/discovery";
        const results = await searchTools("query", { serverId: "database" });
        globalThis.__codemode_result__ = results;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let results = response.result["results"]
        .as_array()
        .expect("results array");
    // All results should be from the "database" server
    for r in results {
        assert_eq!(r["serverId"], "database");
    }
}

#[tokio::test]
async fn search_tools_with_name_detail() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { searchTools } from "@codemode/discovery";
        const results = await searchTools("search", { detail: "name" });
        globalThis.__codemode_result__ = results.results;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let results = response.result.as_array().expect("array");
    assert!(!results.is_empty());
    // Name detail: should have toolName and exportName but NOT description
    let first = &results[0];
    assert!(first["toolName"].is_string());
    assert!(first["exportName"].is_string());
    assert!(
        first.get("description").is_none() || first["description"].is_null(),
        "name detail should omit description"
    );
}

#[tokio::test]
async fn search_tools_with_full_detail() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        import { searchTools } from "@codemode/discovery";
        const results = await searchTools("search", { detail: "full" });
        globalThis.__codemode_result__ = results.results;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let results = response.result.as_array().expect("array");
    // The "search" tool from "files" server should match and have inputSchema at full detail
    let search_result = results
        .iter()
        .find(|r| r["toolName"] == "search")
        .expect("search tool in results");
    assert!(
        search_result["inputSchema"].is_object(),
        "full detail should include inputSchema"
    );
    assert!(
        search_result["description"].is_string(),
        "full detail should include description"
    );
}

// ============================================================
// §11 — Host bridge is frozen and non-writable
// ============================================================

#[tokio::test]
async fn host_bridge_is_frozen() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        const frozen = Object.isFrozen(globalThis.__codemode_internal__);
        globalThis.__codemode_result__ = frozen;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, serde_json::json!(true));
}

#[tokio::test]
async fn host_bridge_is_not_writable() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        // Attempt to overwrite should silently fail (strict mode throws)
        try {
            globalThis.__codemode_internal__ = { hacked: true };
            // Check if the original is still intact
            const hasListServers = typeof globalThis.__codemode_internal__.listServers === 'function';
            globalThis.__codemode_result__ = hasListServers ? 'protected' : 'overwritten';
        } catch (e) {
            // In strict mode (modules are strict), assignment to non-writable throws
            globalThis.__codemode_result__ = 'protected';
        }
    "#,
        )
        .await;

    assert_eq!(response.result, serde_json::json!("protected"));
}

#[tokio::test]
async fn host_bridge_is_not_configurable() {
    let servers = two_servers();
    let sandbox = sandbox_with_servers(servers).await;
    let response = sandbox
        .execute(
            r#"
        const desc = Object.getOwnPropertyDescriptor(globalThis, '__codemode_internal__');
        globalThis.__codemode_result__ = {
            configurable: desc.configurable,
            writable: desc.writable,
            enumerable: desc.enumerable,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["configurable"], false);
    assert_eq!(response.result["writable"], false);
    assert_eq!(response.result["enumerable"], false);
}
