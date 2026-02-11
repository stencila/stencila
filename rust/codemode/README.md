# Stencila Code Mode

An implementation of the concepts in [Code Mode](https://blog.cloudflare.com/code-mode/) from Cloudflare and [Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp) from Anthropic.

## Usage

### Implement `McpServer`

Provide one or more MCP servers by implementing the `McpServer` trait:

```rust
use stencila_codemode::{McpServer, McpToolInfo, McpToolResult, McpContent, CodemodeError};

struct MyServer;

#[async_trait::async_trait]
impl McpServer for MyServer {
    fn server_id(&self) -> &str { "my-server" }
    fn server_name(&self) -> &str { "My Server" }

    async fn tools(&self) -> Result<Vec<McpToolInfo>, CodemodeError> {
        Ok(vec![McpToolInfo {
            name: "greet".into(),
            description: Some("Say hello".into()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": { "name": { "type": "string" } },
                "required": ["name"]
            })),
            output_schema: None,
            annotations: None,
        }])
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<McpToolResult, CodemodeError> {
        match tool_name {
            "greet" => {
                let name = input["name"].as_str().unwrap_or("world");
                Ok(McpToolResult {
                    content: vec![McpContent::Text {
                        text: format!("Hello, {name}!"),
                    }],
                    structured_content: None,
                    is_error: false,
                })
            }
            _ => Err(CodemodeError::ToolNotFound {
                tool_name: tool_name.into(),
                server_id: "my-server".into(),
            }),
        }
    }
}
```

### Run code with `codemode_run`

The top-level entry point always returns a `RunResponse` — errors are captured as diagnostics, never propagated:

```rust
use std::collections::HashSet;
use std::sync::Arc;
use stencila_codemode::{codemode_run, RunRequest, Limits};

let servers = vec![Arc::new(MyServer) as Arc<dyn McpServer>];

let request = RunRequest {
    code: r#"
        import { greet } from "@codemode/servers/my-server";
        export default await greet({ name: "Alice" });
    "#.into(),
    limits: Some(Limits {
        timeout_ms: Some(5000),
        max_tool_calls: Some(10),
        ..Limits::default()
    }),
    requested_capabilities: None,
};

let response = codemode_run(&request, &servers, &HashSet::new()).await;
// response.result  == "Hello, Alice!"
// response.logs    — captured console output
// response.diagnostics — any errors/warnings
// response.tool_trace  — redacted tool call log
```

### Generate TypeScript declarations

Generate `.d.ts` content for injection into the agent's system prompt:

```rust
use stencila_codemode::generate_declarations;

let declarations = generate_declarations(&servers).await?;
// Contains typed declarations for @codemode/discovery,
// @codemode/errors, and @codemode/servers/my-server
```

## Specification

At the time of writing, no "Code Mode spec" is available. This implementation was developed against our own [spec](specs/codemode.md).

## Limitations

The following are known limitations of this implementation.

- **`setTimeout` delay ignored (§3.5)** — delay parameter is accepted but ignored; callbacks fire on the next microtask tick. QuickJS's async runtime cannot easily support real delays within `async_with!`.

- **`URL.searchParams` not supported (§3.5)** — use `new URLSearchParams(url.search)` separately.

- **`URLSearchParams` iterator protocol incomplete (§3.5)** — `for...of` not supported; use `forEach`, `keys`, `values`, or `entries`.

- **`TextDecoder` UTF-8 only (§3.5)** — non-UTF-8 encodings and streaming decode options are not supported.

- **No cancellation support (§7.4)** — sandbox execution runs to completion or until a limit is exceeded; no external cancellation of in-flight executions or MCP tool calls.

- **No host-level parallelism for cross-server tool calls (§10.1)** — `Promise.all` works within the sandbox (JS-level concurrency) but tool calls execute sequentially on the Rust side.

### Testing

Test files map to spec sections. See `tests/spec-traceability.md` for the full mapping.

| File                       | Spec Sections            | Description                                                |
| -------------------------- | ------------------------ | ---------------------------------------------------------- |
| `spec_3_outer_tool.rs`     | §1.4, §3, §4.2, §4.3     | Types, sandbox execution, globals, limits                  |
| `spec_4_discovery.rs`      | §4.1, §4.3, §11, §12.1   | Discovery module, detail levels, host bridge               |
| `spec_5_server_modules.rs` | §3.2.4, §3.3.2, §5, §7.2 | Server module tool bindings, validation, traces            |
| `spec_6_codegen.rs`        | §5.0.1, §6.1, §6.2, §6.3 | Identifiers, TS declarations, annotations                  |
| `spec_7_errors.rs`         | §7.1, §7.3               | Error class hierarchy, hints                               |
| `spec_8_tool_changes.rs`   | §8.1, §8.2               | Tool list changes, dirty server refresh, snapshot freezing |
| `spec_9_logging.rs`        | §3.3.1                   | Console capture, serialization, log truncation             |
| `spec_10_multi_server.rs`  | §3.2.3, §3.3.4, §10      | codemode_run, capabilities, multi-server orchestration     |

Use the crate workflow below:

```sh
cargo fmt -p stencila-codemode
cargo clippy --fix --allow-dirty --all-targets -p stencila-codemode
cargo test -p stencila-codemode
```
