mod common;

use std::sync::Arc;

use stencila_codemode::{
    CodemodeError, McpServer, McpToolInfo, generate_declarations, normalize_server_id,
    resolve_export_collisions, resolve_server_collisions, tool_name_to_export,
};

use common::MockServer;

// ============================================================
// §6.1 Identifier Mapping — tool_name_to_export
// ============================================================

#[test]
fn identity_passthrough() {
    assert_eq!(tool_name_to_export("readFile"), "readFile");
    assert_eq!(tool_name_to_export("search"), "search");
    assert_eq!(tool_name_to_export("a1b2"), "a1b2");
}

#[test]
fn illegal_chars_replaced_with_underscore() {
    assert_eq!(tool_name_to_export("read-file"), "read_file");
    assert_eq!(tool_name_to_export("read.file"), "read_file");
    assert_eq!(tool_name_to_export("ns::tool"), "ns__tool");
    assert_eq!(tool_name_to_export("a b c"), "a_b_c");
    assert_eq!(tool_name_to_export("tool@v2"), "tool_v2");
}

#[test]
fn unicode_letters_preserved() {
    // Unicode letters are valid JS identifier chars and should not be replaced
    assert_eq!(tool_name_to_export("überTool"), "überTool");
    assert_eq!(tool_name_to_export("名前"), "名前");
    assert_eq!(tool_name_to_export("café"), "café");
    assert_eq!(tool_name_to_export("données"), "données");
}

#[test]
fn digit_prefix_gets_underscore() {
    assert_eq!(tool_name_to_export("123tool"), "_123tool");
    assert_eq!(tool_name_to_export("0"), "_0");
    assert_eq!(tool_name_to_export("9lives"), "_9lives");
}

#[test]
fn reserved_words_get_trailing_underscore() {
    let reserved = [
        "await",
        "break",
        "case",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "let",
        "new",
        "null",
        "return",
        "static",
        "super",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
    ];
    for word in reserved {
        let export = tool_name_to_export(word);
        assert_eq!(export, format!("{word}_"), "reserved word: {word}");
    }
}

#[test]
fn non_reserved_not_suffixed() {
    // Words that look similar but are NOT reserved
    assert_eq!(tool_name_to_export("async"), "async"); // not in the spec's list
    assert_eq!(tool_name_to_export("constructor"), "constructor");
    assert_eq!(tool_name_to_export("undefined"), "undefined");
}

#[test]
fn dollar_sign_preserved() {
    assert_eq!(tool_name_to_export("$helper"), "$helper");
    assert_eq!(tool_name_to_export("get$"), "get$");
}

#[test]
fn combined_rules_digit_after_replacement() {
    // "1-tool" → "1_tool" (illegal char) → "_1_tool" (digit prefix)
    assert_eq!(tool_name_to_export("1-tool"), "_1_tool");
}

// ============================================================
// §6.1 Collision Resolution — resolve_export_collisions
// ============================================================

#[test]
fn no_collisions() {
    let result = resolve_export_collisions(&["alpha", "beta", "gamma"]);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], ("alpha".into(), "alpha".into()));
    assert_eq!(result[1], ("beta".into(), "beta".into()));
    assert_eq!(result[2], ("gamma".into(), "gamma".into()));
}

#[test]
fn two_way_collision() {
    let result = resolve_export_collisions(&["read.file", "read-file"]);
    // Alphabetical: "read-file" < "read.file"
    assert_eq!(result[0], ("read-file".into(), "read_file".into()));
    assert_eq!(result[1], ("read.file".into(), "read_file__2".into()));
}

#[test]
fn three_way_collision() {
    let result = resolve_export_collisions(&["a.b", "a-b", "a b"]);
    // Alphabetical: "a b" < "a-b" < "a.b"
    assert_eq!(result[0], ("a b".into(), "a_b".into()));
    assert_eq!(result[1], ("a-b".into(), "a_b__2".into()));
    assert_eq!(result[2], ("a.b".into(), "a_b__3".into()));
}

#[test]
fn collision_ordering_is_deterministic() {
    let fwd = resolve_export_collisions(&["z-x", "a-x"]);
    let rev = resolve_export_collisions(&["a-x", "z-x"]);
    assert_eq!(fwd, rev);
}

#[test]
fn mixed_collision_and_non_collision() {
    let result = resolve_export_collisions(&["foo-bar", "foo.bar", "unique"]);
    assert_eq!(result[0], ("foo-bar".into(), "foo_bar".into()));
    assert_eq!(result[1], ("foo.bar".into(), "foo_bar__2".into()));
    assert_eq!(result[2], ("unique".into(), "unique".into()));
}

// ============================================================
// §5.0.1 Server ID Normalization — normalize_server_id
// ============================================================

#[test]
fn server_id_passthrough() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("google-drive")?, "google-drive");
    assert_eq!(normalize_server_id("server1")?, "server1");
    Ok(())
}

#[test]
fn server_id_uppercase_lowered() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("Google-Drive")?, "google-drive");
    assert_eq!(normalize_server_id("ALLCAPS")?, "allcaps");
    assert_eq!(normalize_server_id("CamelCase")?, "camelcase");
    Ok(())
}

#[test]
fn server_id_special_chars_replaced() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("my_server")?, "my-server");
    assert_eq!(normalize_server_id("ns::server")?, "ns-server");
    assert_eq!(normalize_server_id("server.v2")?, "server-v2");
    assert_eq!(normalize_server_id("a b c")?, "a-b-c");
    Ok(())
}

#[test]
fn server_id_consecutive_dashes_collapsed() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("a---b")?, "a-b");
    assert_eq!(normalize_server_id("a__b")?, "a-b");
    assert_eq!(normalize_server_id("a-_-b")?, "a-b");
    Ok(())
}

#[test]
fn server_id_leading_trailing_stripped() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("-server-")?, "server");
    assert_eq!(normalize_server_id("--server--")?, "server");
    assert_eq!(normalize_server_id("___server___")?, "server");
    Ok(())
}

#[test]
fn server_id_complex_mixed() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("My Server (v2.1)")?, "my-server-v2-1");
    assert_eq!(normalize_server_id("@scope/package")?, "scope-package");
    Ok(())
}

#[test]
fn server_id_all_invalid_chars_returns_invalid_server_id_error() {
    // Verify the correct error variant is returned
    for input in &["___", "---", "", "@#$"] {
        match normalize_server_id(input) {
            Err(CodemodeError::InvalidServerId { server_id }) => {
                assert_eq!(server_id, *input);
            }
            other => panic!("expected InvalidServerId for {input:?}, got {other:?}"),
        }
    }
}

// ============================================================
// §5.0.1 Server Collision Resolution — resolve_server_collisions
// ============================================================

#[test]
fn server_no_collisions() -> Result<(), CodemodeError> {
    let result = resolve_server_collisions(&["server-a", "server-b"])?;
    assert_eq!(result[0], ("server-a".into(), "server-a".into()));
    assert_eq!(result[1], ("server-b".into(), "server-b".into()));
    Ok(())
}

#[test]
fn server_collision_disambiguation() -> Result<(), CodemodeError> {
    let result = resolve_server_collisions(&["Server_A", "server-a"])?;
    // Alphabetical: "Server_A" < "server-a"
    assert_eq!(result[0], ("Server_A".into(), "server-a".into()));
    assert_eq!(result[1], ("server-a".into(), "server-a--2".into()));
    Ok(())
}

#[test]
fn server_collision_deterministic() -> Result<(), CodemodeError> {
    let fwd = resolve_server_collisions(&["server-a", "Server_A"])?;
    let rev = resolve_server_collisions(&["Server_A", "server-a"])?;
    assert_eq!(fwd, rev);
    Ok(())
}

#[test]
fn server_three_way_collision() -> Result<(), CodemodeError> {
    let result = resolve_server_collisions(&["A_B", "a-b", "a.b"])?;
    // All normalize to "a-b". Alphabetical: "A_B" < "a-b" < "a.b"
    assert_eq!(result[0], ("A_B".into(), "a-b".into()));
    assert_eq!(result[1], ("a-b".into(), "a-b--2".into()));
    assert_eq!(result[2], ("a.b".into(), "a-b--3".into()));
    Ok(())
}

#[test]
fn server_collision_with_invalid_id_returns_error() {
    let result = resolve_server_collisions(&["server-a", "___"]);
    assert!(result.is_err());
}

// ============================================================
// §6.2 TypeScript Declaration Generation — generate_declarations
// ============================================================

/// Helper to run async generate_declarations in tests.
fn gen_decls(servers: Vec<Arc<dyn McpServer>>) -> String {
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async { generate_declarations(&servers).await.expect("declarations") })
}

#[test]
fn declarations_include_discovery_module() {
    let decls = gen_decls(vec![]);
    assert!(decls.contains(r#"declare module "@codemode/discovery""#));
    assert!(decls.contains("export const specVersion: string"));
    assert!(decls.contains("export function listServers"));
    assert!(decls.contains("export function describeServer"));
    assert!(decls.contains("export function listTools"));
    assert!(decls.contains("export function getTool"));
    assert!(decls.contains("export function searchTools"));
}

#[test]
fn declarations_include_errors_module() {
    let decls = gen_decls(vec![]);
    assert!(decls.contains(r#"declare module "@codemode/errors""#));
    assert!(decls.contains("class CodemodeError extends Error"));
    assert!(decls.contains("class SchemaValidationError"));
    assert!(decls.contains("class ToolNotFoundError"));
    assert!(decls.contains("class ServerNotFoundError"));
    assert!(decls.contains("class ToolCallError"));
    assert!(decls.contains("class AuthenticationError"));
    assert!(decls.contains("class SandboxLimitError"));
}

#[test]
fn declarations_for_server_with_no_schema_tools() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "test-server",
        "Test Server",
        vec![common::simple_tool("ping", "Ping the server")],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains(r#"declare module "@codemode/servers/test-server""#));
    assert!(decls.contains("export function ping(): Promise<unknown>"));
    assert!(decls.contains("* Ping the server"));
}

#[test]
fn declarations_for_tool_with_typed_input() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "files",
        "File Server",
        vec![McpToolInfo {
            name: "readFile".into(),
            description: Some("Read a file".into()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "encoding": {"type": "string"}
                },
                "required": ["path"]
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("path: string"));
    assert!(decls.contains("encoding?: string"));
    assert!(decls.contains("Promise<unknown>"));
}

#[test]
fn declarations_for_tool_with_typed_output() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "api",
        "API Server",
        vec![McpToolInfo {
            name: "getStatus".into(),
            description: Some("Get status".into()),
            input_schema: None,
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "ok": {"type": "boolean"},
                    "code": {"type": "number"}
                },
                "required": ["ok", "code"]
            })),
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("Promise<{ code: number; ok: boolean }>"));
}

// §6.2: enum → union of literals
#[test]
fn declarations_enum_mapped_to_union() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "setMode".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "mode": {"enum": ["dark", "light", "auto"]}
                },
                "required": ["mode"]
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains(r#""dark" | "light" | "auto""#));
}

// §6.2: const
#[test]
fn declarations_const_mapped_to_literal() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "constTool".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "version": {"const": 2}
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("version?: 2"));
}

// §6.2: oneOf / anyOf
#[test]
fn declarations_oneof_mapped_to_union() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "flexible".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "value": {"oneOf": [{"type": "string"}, {"type": "number"}]}
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("value?: string | number"));
}

// §6.2: nullable
#[test]
fn declarations_nullable_appends_null() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "nullableTool".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "nullable": true}
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("name?: string | null"));
}

// §6.2: $ref within same schema
#[test]
fn declarations_ref_resolved_inline() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "refTool".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {"$ref": "#/$defs/Address"}
                },
                "$defs": {
                    "Address": {
                        "type": "object",
                        "properties": {
                            "street": {"type": "string"},
                            "city": {"type": "string"}
                        },
                        "required": ["street", "city"]
                    }
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("city: string"));
    assert!(decls.contains("street: string"));
}

// §6.2: recursive schema
#[test]
fn declarations_recursive_schema_generates_named_interface() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "tree",
        "Tree",
        vec![McpToolInfo {
            name: "buildTree".into(),
            description: Some("Build a tree".into()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "label": {"type": "string"},
                    "children": {
                        "type": "array",
                        "items": {"$ref": "#"}
                    }
                },
                "required": ["label"]
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    // Should generate a named interface for the recursive type
    assert!(decls.contains("export interface BuildTreeInput"));
    // Function should reference the named interface
    assert!(decls.contains("export function buildTree(input: BuildTreeInput)"));
}

// §6.2: additionalProperties
#[test]
fn declarations_additional_properties_as_record() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "setHeaders".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "headers": {
                        "type": "object",
                        "additionalProperties": {"type": "string"}
                    }
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("headers?: Record<string, string>"));
}

// §6.2: patternProperties
#[test]
fn declarations_pattern_properties_as_record() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "setExtensions".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "extensions": {
                        "type": "object",
                        "patternProperties": {
                            "^x-": {"type": "string"}
                        }
                    }
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("extensions?: Record<string, string>"));
}

// §6.2: tuple schema
#[test]
fn declarations_tuple_schema() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "setCoords".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "point": {
                        "type": "array",
                        "items": [{"type": "number"}, {"type": "number"}]
                    }
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("point?: [number, number]"));
}

// §6.2: unsupported schema falls back to unknown
#[test]
fn declarations_unsupported_falls_back_to_unknown() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "exotic".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "data": {"type": "custom_type"}
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("unknown"));
}

// §6.3: tool annotations in doc comments
#[test]
fn declarations_tool_annotations_in_doc_comments() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "files",
        "Files",
        vec![McpToolInfo {
            name: "deleteFile".into(),
            description: Some("Delete a file from the filesystem".into()),
            input_schema: None,
            output_schema: None,
            annotations: Some(serde_json::json!({
                "destructiveHint": true,
                "idempotentHint": true,
                "readOnlyHint": false
            })),
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("* Delete a file from the filesystem"));
    assert!(decls.contains("@destructiveHint true"));
    assert!(decls.contains("@idempotentHint true"));
    assert!(decls.contains("@readOnlyHint false"));
}

// §6.2: multiple servers generate separate module declarations
#[test]
fn declarations_multiple_servers() {
    let server_a: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "alpha",
        "Alpha",
        vec![common::simple_tool("foo", "Foo tool")],
    ));
    let server_b: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "beta",
        "Beta",
        vec![common::simple_tool("bar", "Bar tool")],
    ));
    let decls = gen_decls(vec![server_a, server_b]);
    assert!(decls.contains(r#"declare module "@codemode/servers/alpha""#));
    assert!(decls.contains(r#"declare module "@codemode/servers/beta""#));
    assert!(decls.contains("export function foo()"));
    assert!(decls.contains("export function bar()"));
}

// §6.2: __meta__ export in server declarations
#[test]
fn declarations_meta_export_shape() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![common::simple_tool("test", "A test")],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("export const __meta__"));
    assert!(decls.contains("readonly serverId: string"));
    assert!(decls.contains("readonly serverName: string"));
    assert!(decls.contains("readonly tools: ReadonlyArray"));
}

// §6.2: recursive output schema generates named interface
#[test]
fn declarations_recursive_output_schema_generates_named_interface() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "tree",
        "Tree",
        vec![McpToolInfo {
            name: "getTree".into(),
            description: Some("Get a tree".into()),
            input_schema: None,
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "label": {"type": "string"},
                    "children": {
                        "type": "array",
                        "items": {"$ref": "#"}
                    }
                }
            })),
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("export interface GetTreeOutput"));
    assert!(decls.contains("export function getTree(): Promise<GetTreeOutput>"));
}

// §6.2: named properties + additionalProperties → intersection type
#[test]
fn declarations_properties_with_additional_uses_intersection() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "mixed".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                },
                "additionalProperties": {"type": "number"},
                "required": ["name"]
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("{ name: string } & Record<string, number>"));
}

// §6.2: named properties + patternProperties → intersection type
#[test]
fn declarations_properties_with_pattern_uses_intersection() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "tagged".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "number"}
                },
                "patternProperties": {"^x-": {"type": "string"}},
                "required": ["id"]
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("{ id: number } & Record<string, string>"));
}

// §6.2: recursive non-object schema uses type alias, not interface
#[test]
fn declarations_recursive_array_schema_uses_type_alias() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "list",
        "List",
        vec![McpToolInfo {
            name: "flatten".into(),
            description: Some("Flatten nested arrays".into()),
            input_schema: Some(serde_json::json!({
                "type": "array",
                "items": {"$ref": "#"}
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    // Non-object recursive type should use `export type`, not `export interface`
    assert!(decls.contains("export type FlattenInput ="));
    assert!(!decls.contains("export interface FlattenInput"));
    assert!(decls.contains("export function flatten(input: FlattenInput)"));
}

// §6.2: enum/const string literals are properly escaped in .d.ts
#[test]
fn declarations_enum_with_special_chars_escaped() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![McpToolInfo {
            name: "quoting".into(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "style": {"enum": ["he said \"hi\"", "path\\to"]}
                }
            })),
            output_schema: None,
            annotations: None,
        }],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains(r#""he said \"hi\"""#));
    assert!(decls.contains(r#""path\\to""#));
}

// §7.1: error class fields match runtime nullability
#[test]
fn declarations_error_fields_nullable() {
    let decls = gen_decls(vec![]);
    // All runtime fields use `|| null`, so declarations should be `string | null`
    assert!(decls.contains("hint: string | null"));
    assert!(decls.contains("kind: string | null"));
}

// §5.2: __meta__.serverVersion is always present (not optional)
#[test]
fn declarations_meta_server_version_not_optional() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "s",
        "S",
        vec![common::simple_tool("t", "T")],
    ));
    let decls = gen_decls(vec![server]);
    assert!(decls.contains("readonly serverVersion: string"));
    assert!(!decls.contains("readonly serverVersion?"));
}

// ============================================================
// §6.2 Failure-path tests — generate_declarations error propagation
// ============================================================

/// A mock server that returns an error from `tools()`.
struct FailingToolsServer;

#[async_trait::async_trait]
impl McpServer for FailingToolsServer {
    fn server_id(&self) -> &str {
        "failing"
    }
    fn server_name(&self) -> &str {
        "Failing Server"
    }
    async fn tools(&self) -> Result<Vec<McpToolInfo>, CodemodeError> {
        Err(CodemodeError::Runtime("tools() failed".into()))
    }
    async fn call_tool(
        &self,
        _tool_name: &str,
        _input: serde_json::Value,
    ) -> Result<stencila_codemode::McpToolResult, CodemodeError> {
        Err(CodemodeError::Runtime("not implemented".into()))
    }
}

#[test]
fn declarations_propagates_tools_error() {
    let server: Arc<dyn McpServer> = Arc::new(FailingToolsServer);
    let result = tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async { generate_declarations(&[server]).await });
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(
        err.to_string().contains("tools() failed"),
        "unexpected error: {err}"
    );
}

/// A mock server with an invalid ID that normalizes to empty.
struct InvalidIdServer;

#[async_trait::async_trait]
impl McpServer for InvalidIdServer {
    fn server_id(&self) -> &str {
        "---"
    }
    fn server_name(&self) -> &str {
        "Invalid"
    }
    async fn tools(&self) -> Result<Vec<McpToolInfo>, CodemodeError> {
        Ok(vec![])
    }
    async fn call_tool(
        &self,
        _tool_name: &str,
        _input: serde_json::Value,
    ) -> Result<stencila_codemode::McpToolResult, CodemodeError> {
        Err(CodemodeError::Runtime("not implemented".into()))
    }
}

#[test]
fn declarations_propagates_invalid_server_id() {
    let server: Arc<dyn McpServer> = Arc::new(InvalidIdServer);
    let result = tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async { generate_declarations(&[server]).await });
    assert!(result.is_err());
    match result.expect_err("expected error") {
        CodemodeError::InvalidServerId { server_id } => {
            assert_eq!(server_id, "---");
        }
        other => panic!("expected InvalidServerId, got: {other:?}"),
    }
}
