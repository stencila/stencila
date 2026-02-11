mod common;

use common::sandbox_with_server;

// ============================================================
// §7.1 — Error class hierarchy
// ============================================================

#[tokio::test]
async fn codemode_error_extends_error() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError } from "@codemode/errors";
        const e = new CodemodeError("test error", "try again");
        globalThis.__codemode_result__ = {
            isError: e instanceof Error,
            name: e.name,
            message: e.message,
            hint: e.hint,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isError"], true);
    assert_eq!(response.result["name"], "CodemodeError");
    assert_eq!(response.result["message"], "test error");
    assert_eq!(response.result["hint"], "try again");
}

#[tokio::test]
async fn schema_validation_error_hierarchy() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError, SchemaValidationError } from "@codemode/errors";
        const e = new SchemaValidationError("invalid input", {
            toolName: "search",
            exportName: "search",
            path: "/query",
            expected: "string",
            received: "number",
            hint: "Provide a string for query"
        });
        globalThis.__codemode_result__ = {
            isCodemodeError: e instanceof CodemodeError,
            isError: e instanceof Error,
            name: e.name,
            message: e.message,
            hint: e.hint,
            toolName: e.toolName,
            exportName: e.exportName,
            path: e.path,
            expected: e.expected,
            received: e.received,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isCodemodeError"], true);
    assert_eq!(response.result["isError"], true);
    assert_eq!(response.result["name"], "SchemaValidationError");
    assert_eq!(response.result["toolName"], "search");
    assert_eq!(response.result["path"], "/query");
    assert_eq!(response.result["hint"], "Provide a string for query");
}

#[tokio::test]
async fn tool_not_found_error_hierarchy() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError, ToolNotFoundError } from "@codemode/errors";
        const e = new ToolNotFoundError("tool not found", {
            serverId: "server-a",
            toolName: "missing",
            hint: "Check available tools with listTools()"
        });
        globalThis.__codemode_result__ = {
            isCodemodeError: e instanceof CodemodeError,
            isError: e instanceof Error,
            name: e.name,
            serverId: e.serverId,
            toolName: e.toolName,
            hint: e.hint,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isCodemodeError"], true);
    assert_eq!(response.result["name"], "ToolNotFoundError");
    assert_eq!(response.result["serverId"], "server-a");
    assert_eq!(
        response.result["hint"],
        "Check available tools with listTools()"
    );
}

#[tokio::test]
async fn server_not_found_error_hierarchy() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError, ServerNotFoundError } from "@codemode/errors";
        const e = new ServerNotFoundError("server gone", {
            serverId: "ghost",
            hint: "Use listServers() to find available servers"
        });
        globalThis.__codemode_result__ = {
            isCodemodeError: e instanceof CodemodeError,
            name: e.name,
            serverId: e.serverId,
            hint: e.hint,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isCodemodeError"], true);
    assert_eq!(response.result["name"], "ServerNotFoundError");
}

#[tokio::test]
async fn tool_call_error_hierarchy() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError, ToolCallError } from "@codemode/errors";
        const e = new ToolCallError("call failed", {
            serverId: "s1",
            toolName: "boom",
            hint: "Retry the operation"
        });
        globalThis.__codemode_result__ = {
            isCodemodeError: e instanceof CodemodeError,
            name: e.name,
            serverId: e.serverId,
            toolName: e.toolName,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isCodemodeError"], true);
    assert_eq!(response.result["name"], "ToolCallError");
}

#[tokio::test]
async fn authentication_error_hierarchy() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError, AuthenticationError } from "@codemode/errors";
        const e = new AuthenticationError("bad creds", {
            serverId: "secure-server",
            hint: "Check your API key"
        });
        globalThis.__codemode_result__ = {
            isCodemodeError: e instanceof CodemodeError,
            name: e.name,
            serverId: e.serverId,
            hint: e.hint,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isCodemodeError"], true);
    assert_eq!(response.result["name"], "AuthenticationError");
    assert_eq!(response.result["hint"], "Check your API key");
}

#[tokio::test]
async fn sandbox_limit_error_hierarchy() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError, SandboxLimitError } from "@codemode/errors";
        const e = new SandboxLimitError("timeout exceeded", {
            kind: "timeout",
            hint: "Increase timeoutMs or simplify the code"
        });
        globalThis.__codemode_result__ = {
            isCodemodeError: e instanceof CodemodeError,
            isError: e instanceof Error,
            name: e.name,
            kind: e.kind,
            hint: e.hint,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isCodemodeError"], true);
    assert_eq!(response.result["isError"], true);
    assert_eq!(response.result["name"], "SandboxLimitError");
    assert_eq!(response.result["kind"], "timeout");
}

// ============================================================
// §7.3 — Error hints
// ============================================================

#[tokio::test]
async fn error_hint_is_null_when_omitted() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError } from "@codemode/errors";
        const e = new CodemodeError("no hint given");
        globalThis.__codemode_result__ = e.hint;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, serde_json::Value::Null);
}

// ============================================================
// §7.1 — All six subclasses are instanceof CodemodeError
// ============================================================

#[tokio::test]
async fn all_error_classes_extend_codemode_error() {
    let sandbox = sandbox_with_server().await;
    let response = sandbox
        .execute(
            r#"
        import {
            CodemodeError,
            SchemaValidationError,
            ToolNotFoundError,
            ServerNotFoundError,
            ToolCallError,
            AuthenticationError,
            SandboxLimitError,
        } from "@codemode/errors";

        const classes = [
            SchemaValidationError,
            ToolNotFoundError,
            ServerNotFoundError,
            ToolCallError,
            AuthenticationError,
            SandboxLimitError,
        ];

        const results = classes.map(C => {
            const e = new C("test");
            return {
                name: e.name,
                isCodemodeError: e instanceof CodemodeError,
                isError: e instanceof Error,
            };
        });

        globalThis.__codemode_result__ = results;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert_eq!(arr.len(), 6);
    for entry in arr {
        assert_eq!(
            entry["isCodemodeError"], true,
            "{} should extend CodemodeError",
            entry["name"]
        );
        assert_eq!(
            entry["isError"], true,
            "{} should extend Error",
            entry["name"]
        );
    }
}
