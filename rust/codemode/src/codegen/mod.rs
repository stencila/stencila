pub(crate) mod ts_declarations;

use std::fmt::Write as _;
use std::sync::Arc;

use crate::error::CodemodeError;
use crate::modules::{ServerToolset, ToolSnapshot};
use stencila_mcp::McpServer;

use ts_declarations::{convert_schema, generate_doc_comment, to_pascal_case};

/// Generate TypeScript declarations for all tools across the given servers.
///
/// Returns a `.d.ts` string suitable for injection into agent context.
/// The host is responsible for injection (system prompt, file, etc.) per §4.4.
///
/// The generated declarations cover:
/// - `@codemode/discovery` module (static type declarations)
/// - `@codemode/errors` module (error class hierarchy)
/// - `@codemode/servers/<id>` modules (one per server, with typed tool bindings)
///
/// # Errors
///
/// Returns `CodemodeError` if tool snapshot building fails (e.g. a server's
/// `tools()` call errors, or a server ID normalizes to an empty string).
///
/// # Deviation from `PLAN.md`
///
/// The plan specifies `pub fn generate_declarations(servers: &[&dyn McpServer]) -> String`.
/// This implementation is `async` (because `McpServer::tools()` is async) and returns
/// `Result<String, CodemodeError>` (idiomatic error propagation). It accepts
/// `&[Arc<dyn McpServer>]` to match the existing crate convention.
pub async fn generate_declarations(
    servers: &[Arc<dyn McpServer>],
) -> Result<String, CodemodeError> {
    let snapshot = ToolSnapshot::build(servers).await?;
    Ok(declarations_from_snapshot(&snapshot))
}

/// Generate TypeScript declarations from a pre-built tool snapshot (sync).
pub(crate) fn declarations_from_snapshot(snapshot: &ToolSnapshot) -> String {
    let mut output = String::new();

    output.push_str(&discovery_declarations());
    output.push('\n');
    output.push_str(&errors_declarations());

    for server in &snapshot.servers {
        output.push('\n');
        output.push_str(&server_declarations(server));
    }

    output
}

/// Static declarations for the `@codemode/discovery` module.
fn discovery_declarations() -> String {
    r#"declare module "@codemode/discovery" {
  export const specVersion: string;

  export interface ServerInfo {
    serverId: string;
    serverName: string;
    capabilities?: string[];
  }

  export interface ServerDescription extends ServerInfo {
    description?: string;
    version?: string;
  }

  export interface ToolSummary {
    toolName: string;
    exportName: string;
    description?: string;
    annotations?: Record<string, unknown>;
  }

  export interface ToolDefinition extends ToolSummary {
    inputSchema?: Record<string, unknown>;
    outputSchema?: Record<string, unknown>;
  }

  export interface ListToolsOptions {
    detail?: "name" | "description" | "full";
  }

  export interface SearchToolsOptions {
    detail?: "name" | "description" | "full";
    serverId?: string;
    limit?: number;
  }

  export interface SearchResults {
    query: string;
    results: (ToolDefinition & { serverId: string })[];
  }

  export function listServers(): Promise<ServerInfo[]>;
  export function describeServer(serverId: string): Promise<ServerDescription>;
  export function listTools(serverId: string, options?: ListToolsOptions): Promise<ToolDefinition[]>;
  export function getTool(serverId: string, toolName: string): Promise<ToolDefinition>;
  export function searchTools(query: string, options?: SearchToolsOptions): Promise<SearchResults>;
}
"#
    .to_string()
}

/// Static declarations for the `@codemode/errors` module.
fn errors_declarations() -> String {
    r#"declare module "@codemode/errors" {
  export class CodemodeError extends Error {
    hint: string | null;
  }

  export class SchemaValidationError extends CodemodeError {
    toolName: string | null;
    exportName: string | null;
    path: string | null;
    expected: string | null;
    received: string | null;
  }

  export class ToolNotFoundError extends CodemodeError {
    serverId: string | null;
    toolName: string | null;
  }

  export class ServerNotFoundError extends CodemodeError {
    serverId: string | null;
  }

  export class ToolCallError extends CodemodeError {
    serverId: string | null;
    toolName: string | null;
  }

  export class AuthenticationError extends CodemodeError {
    serverId: string | null;
  }

  export class SandboxLimitError extends CodemodeError {
    kind: string | null;
  }
}
"#
    .to_string()
}

/// Generate declarations for a single server's `@codemode/servers/<id>` module.
fn server_declarations(server: &ServerToolset) -> String {
    let mut output = String::new();
    let module_path = format!("@codemode/servers/{}", server.normalized_id);

    let _ = writeln!(output, "declare module \"{module_path}\" {{");

    for tool in &server.tools {
        // Convert each schema once; reuse for both named type and function signature
        let input_conv = tool.input_schema.as_ref().map(|schema| {
            let type_name = format!("{}Input", to_pascal_case(&tool.export_name));
            let (ts, has_recursion) = convert_schema(schema, &type_name);
            (type_name, ts, has_recursion)
        });
        let output_conv = tool.output_schema.as_ref().map(|schema| {
            let type_name = format!("{}Output", to_pascal_case(&tool.export_name));
            let (ts, has_recursion) = convert_schema(schema, &type_name);
            (type_name, ts, has_recursion)
        });

        // Emit named type declarations for recursive schemas
        if let Some((ref name, ref ts, true)) = input_conv {
            emit_named_type(&mut output, name, ts);
        }
        if let Some((ref name, ref ts, true)) = output_conv {
            emit_named_type(&mut output, name, ts);
        }

        // Function declaration with doc comment
        let doc =
            generate_doc_comment(tool.description.as_deref(), tool.annotations.as_ref(), "  ");
        output.push_str(&doc);

        let input_type = input_conv.map(|(name, ts, has_rec)| if has_rec { name } else { ts });
        let return_type = output_conv.map_or_else(
            || "unknown".to_string(),
            |(name, ts, has_rec)| if has_rec { name } else { ts },
        );

        match input_type {
            Some(it) => {
                let _ = writeln!(
                    output,
                    "  export function {}(input: {}): Promise<{}>;",
                    tool.export_name, it, return_type
                );
            }
            None => {
                let _ = writeln!(
                    output,
                    "  export function {}(): Promise<{}>;",
                    tool.export_name, return_type
                );
            }
        }
    }

    // __meta__ export
    output.push_str(&generate_meta_declaration());

    output.push_str("}\n");
    output
}

/// Emit a named type declaration for a recursive schema.
///
/// Uses `export interface` when the type is a plain object literal (`{ ... }`),
/// otherwise falls back to `export type` alias (valid for arrays, unions, etc.).
fn emit_named_type(output: &mut String, name: &str, ts_type: &str) {
    let trimmed = ts_type.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let _ = writeln!(output, "  export interface {name} {trimmed}\n");
    } else {
        let _ = writeln!(output, "  export type {name} = {trimmed};\n");
    }
}

/// Generate the `__meta__` type declaration.
fn generate_meta_declaration() -> String {
    r"
  export const __meta__: {
    readonly serverId: string;
    readonly serverName: string;
    readonly serverVersion: string;
    readonly tools: ReadonlyArray<{
      readonly toolName: string;
      readonly exportName: string;
      readonly description?: string;
    }>;
  };
"
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::SnapshotTool;

    fn make_snapshot(servers: Vec<ServerToolset>) -> ToolSnapshot {
        ToolSnapshot { servers }
    }

    fn make_server(id: &str, name: &str, tools: Vec<SnapshotTool>) -> ServerToolset {
        ServerToolset {
            original_id: id.into(),
            normalized_id: id.into(),
            server_name: name.into(),
            description: None,
            version: None,
            capabilities: None,
            tools,
        }
    }

    fn make_tool(name: &str, export_name: &str) -> SnapshotTool {
        SnapshotTool {
            name: name.into(),
            export_name: export_name.into(),
            description: Some(format!("Call {name}")),
            input_schema: None,
            output_schema: None,
            annotations: None,
        }
    }

    fn make_tool_with_schemas(
        name: &str,
        export_name: &str,
        input_schema: Option<serde_json::Value>,
        output_schema: Option<serde_json::Value>,
    ) -> SnapshotTool {
        SnapshotTool {
            name: name.into(),
            export_name: export_name.into(),
            description: Some(format!("Call {name}")),
            input_schema,
            output_schema,
            annotations: None,
        }
    }

    #[test]
    fn empty_snapshot_has_discovery_and_errors() {
        let snapshot = make_snapshot(vec![]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains(r#"declare module "@codemode/discovery""#));
        assert!(decls.contains(r#"declare module "@codemode/errors""#));
    }

    #[test]
    fn server_module_declared() {
        let snapshot = make_snapshot(vec![make_server(
            "files",
            "File Server",
            vec![make_tool("readFile", "readFile")],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains(r#"declare module "@codemode/servers/files""#));
        assert!(decls.contains("export function readFile(): Promise<unknown>"));
    }

    #[test]
    fn tool_with_input_schema_has_typed_param() {
        let snapshot = make_snapshot(vec![make_server(
            "files",
            "File Server",
            vec![make_tool_with_schemas(
                "readFile",
                "readFile",
                Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                })),
                None,
            )],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(
            decls.contains("export function readFile(input: { path: string }): Promise<unknown>")
        );
    }

    #[test]
    fn tool_with_output_schema_has_typed_return() {
        let snapshot = make_snapshot(vec![make_server(
            "files",
            "File Server",
            vec![make_tool_with_schemas(
                "readFile",
                "readFile",
                None,
                Some(serde_json::json!({"type": "string"})),
            )],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains("export function readFile(): Promise<string>"));
    }

    #[test]
    fn tool_annotations_in_doc_comment() {
        let snapshot = make_snapshot(vec![make_server(
            "files",
            "File Server",
            vec![SnapshotTool {
                name: "delete".into(),
                export_name: "delete_".into(),
                description: Some("Delete a file".into()),
                input_schema: None,
                output_schema: None,
                annotations: Some(serde_json::json!({
                    "destructiveHint": true,
                    "readOnlyHint": false
                })),
            }],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains("* Delete a file"));
        assert!(decls.contains("@destructiveHint true"));
        assert!(decls.contains("@readOnlyHint false"));
    }

    #[test]
    fn meta_export_declared() {
        let snapshot = make_snapshot(vec![make_server(
            "files",
            "File Server",
            vec![make_tool("ping", "ping")],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains("export const __meta__"));
        assert!(decls.contains("readonly serverId: string"));
        assert!(decls.contains("readonly tools: ReadonlyArray"));
    }

    #[test]
    fn recursive_input_schema_generates_interface() {
        let snapshot = make_snapshot(vec![make_server(
            "tree",
            "Tree Server",
            vec![make_tool_with_schemas(
                "createTree",
                "createTree",
                Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "children": {
                            "type": "array",
                            "items": {"$ref": "#"}
                        }
                    },
                    "required": ["name"]
                })),
                None,
            )],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains("export interface CreateTreeInput"));
        assert!(decls.contains("export function createTree(input: CreateTreeInput)"));
    }

    #[test]
    fn recursive_output_schema_generates_interface() {
        let snapshot = make_snapshot(vec![make_server(
            "tree",
            "Tree Server",
            vec![make_tool_with_schemas(
                "getTree",
                "getTree",
                None,
                Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "label": {"type": "string"},
                        "children": {
                            "type": "array",
                            "items": {"$ref": "#"}
                        }
                    }
                })),
            )],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains("export interface GetTreeOutput"));
        assert!(decls.contains("export function getTree(): Promise<GetTreeOutput>"));
    }

    #[test]
    fn recursive_array_schema_uses_type_alias() {
        let snapshot = make_snapshot(vec![make_server(
            "s",
            "S",
            vec![make_tool_with_schemas(
                "flatten",
                "flatten",
                Some(serde_json::json!({
                    "type": "array",
                    "items": {"$ref": "#"}
                })),
                None,
            )],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains("export type FlattenInput = "));
        assert!(!decls.contains("export interface FlattenInput"));
    }

    #[test]
    fn properties_with_additional_uses_intersection() {
        let snapshot = make_snapshot(vec![make_server(
            "s",
            "S",
            vec![make_tool_with_schemas(
                "mixed",
                "mixed",
                Some(serde_json::json!({
                    "type": "object",
                    "properties": {"name": {"type": "string"}},
                    "additionalProperties": {"type": "number"},
                    "required": ["name"]
                })),
                None,
            )],
        )]);
        let decls = declarations_from_snapshot(&snapshot);
        assert!(decls.contains("{ name: string } & Record<string, number>"));
    }
}
