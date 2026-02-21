use super::ServerToolset;
use super::SnapshotTool;

/// Generate the complete JS module source for a `@codemode/servers/<id>` module.
///
/// The generated module imports error classes from `@codemode/errors`,
/// exposes one `async function` per tool, and a frozen `__meta__` export.
pub(crate) fn generate_module(server: &ServerToolset) -> String {
    let mut parts = Vec::new();

    // Imports
    parts.push(
        "import { SchemaValidationError, ToolCallError, SandboxLimitError, ToolNotFoundError, AuthenticationError } from \"@codemode/errors\";\n\
         const __internal__ = globalThis.__codemode_internal__;\n"
            .to_string(),
    );

    // Shared result handler
    parts.push(generate_handle_result());

    // One async function per tool
    for tool in &server.tools {
        parts.push(generate_tool_function(server, tool));
    }

    // __meta__ export
    parts.push(generate_meta(server));

    parts.join("\n")
}

/// Generate the `__handleResult__` helper that parses the JSON envelope
/// returned by the Rust bridge and throws the appropriate error on failure.
fn generate_handle_result() -> String {
    r#"function __handleResult__(json) {
    const r = JSON.parse(json);
    if (r.ok) return r.value;
    switch (r.error) {
        case "schema_validation":
            throw new SchemaValidationError(r.message, { toolName: r.toolName, exportName: r.exportName, path: r.path, expected: r.expected, received: r.received, hint: r.hint });
        case "tool_call":
            throw new ToolCallError(r.message, { serverId: r.serverId, toolName: r.toolName, hint: r.hint });
        case "sandbox_limit":
            throw new SandboxLimitError(r.message, { kind: r.kind, hint: r.hint });
        case "tool_not_found":
            throw new ToolNotFoundError(r.message, { serverId: r.serverId, toolName: r.toolName, hint: r.hint });
        case "authentication":
            throw new AuthenticationError(r.message, { serverId: r.serverId, hint: r.hint });
        default:
            throw new ToolCallError(r.message || "Unknown error", { hint: "An unexpected error occurred." });
    }
}"#
    .to_string()
}

/// Generate one `async function exportName(input) { ... }` for a single tool.
fn generate_tool_function(server: &ServerToolset, tool: &SnapshotTool) -> String {
    let server_id = js_escape(&server.normalized_id);
    let tool_name = js_escape(&tool.name);
    let export_name = &tool.export_name;

    format!(
        r#"export async function {export_name}(input) {{
    const payload = (input !== undefined) ? JSON.stringify(input) : "{{}}";
    return __handleResult__(await __internal__.callTool("{server_id}", "{tool_name}", payload));
}}"#
    )
}

/// Generate the `__meta__` export with server info and tool list.
fn generate_meta(server: &ServerToolset) -> String {
    let server_id = js_escape(&server.normalized_id);
    let server_name = js_escape(&server.server_name);
    let server_version = server.version.as_deref().map(js_escape).unwrap_or_default();

    let tool_entries: Vec<String> = server
        .tools
        .iter()
        .map(|t| {
            let tn = js_escape(&t.name);
            let en = js_escape(&t.export_name);
            let desc = t
                .description
                .as_deref()
                .map(|d| format!(r#", description: "{}""#, js_escape(d)))
                .unwrap_or_default();
            format!(r#"        Object.freeze({{ toolName: "{tn}", exportName: "{en}"{desc} }})"#)
        })
        .collect();

    let tools_array = tool_entries.join(",\n");

    format!(
        r#"export const __meta__ = Object.freeze({{
    serverId: "{server_id}",
    serverName: "{server_name}",
    serverVersion: "{server_version}",
    tools: Object.freeze([
{tools_array}
    ]),
}});"#
    )
}

/// Escape a string for embedding inside a JavaScript string literal (double-quoted).
fn js_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_escape_handles_special_chars() {
        assert_eq!(js_escape(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(js_escape("line\nnew"), "line\\nnew");
    }

    #[test]
    fn generate_module_produces_valid_structure() {
        let server = ServerToolset {
            original_id: "files".into(),
            normalized_id: "files".into(),
            server_name: "File Server".into(),
            description: Some("A file server".into()),
            version: Some("1.0.0".into()),
            capabilities: None,
            tools: vec![SnapshotTool {
                name: "readFile".into(),
                export_name: "readFile".into(),
                description: Some("Read a file".into()),
                input_schema: None,
                output_schema: None,
                annotations: None,
            }],
        };

        let source = generate_module(&server);

        // Check key structural elements
        assert!(source.contains("import { SchemaValidationError"));
        assert!(source.contains("function __handleResult__"));
        assert!(source.contains("export async function readFile(input)"));
        assert!(source.contains("__internal__.callTool(\"files\", \"readFile\""));
        assert!(source.contains("export const __meta__"));
        assert!(source.contains("serverId: \"files\""));
        assert!(source.contains("serverVersion: \"1.0.0\""));
    }
}
