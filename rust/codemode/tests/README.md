# Test Structure

## File Layout

Tests are organized by spec section. Each file covers a cohesive set of requirements:

| File | Spec Sections | Description |
|---|---|---|
| `spec_3_outer_tool.rs` | §1.4, §3, §4.2, §4.3 | Types, sandbox execution, globals, polyfills, limits |
| `spec_4_discovery.rs` | §4.1, §4.3, §11, §12.1 | Discovery module, detail levels, host bridge |
| `spec_5_server_modules.rs` | §3.2.4, §3.3.2, §5, §7.2 | Server module tool bindings, validation, traces |
| `spec_6_codegen.rs` | §5.0.1, §6.1, §6.2, §6.3 | Identifiers, TS declarations, annotations |
| `spec_7_errors.rs` | §7.1, §7.3 | Error class hierarchy, hints |
| `spec_8_tool_changes.rs` | §8.1, §8.2 | Tool list changes, dirty server refresh, snapshot freezing |
| `spec_9_logging.rs` | §3.3.1 | Console capture, serialization, log truncation |
| `spec_10_multi_server.rs` | §3.2.3, §3.3.4, §10 | codemode_run, capabilities, multi-server orchestration |

Unit tests live alongside source code in `src/` modules (e.g. `src/codegen/mod.rs`, `src/codegen/ts_declarations.rs`).

## Shared Helpers

`tests/common/mod.rs` provides:

- `MockServer` — configurable `McpServer` implementation for integration tests
- `simple_tool(name, description)` — creates a `McpToolInfo` with no schema
- `echo_server()`, `math_server()` — pre-built server fixtures

## Conventions

- **Naming**: test functions use `snake_case` descriptive names (e.g. `schema_validation_missing_required_field`)
- **Async tests**: use `#[tokio::test]` for tests requiring async sandbox execution
- **Sync tests**: use `#[test]` with `tokio::runtime::Runtime::new().block_on()` when only the setup is async
- **No unwrap**: use `?` with `Result` return types or `.expect("reason")` — clippy denies `unwrap_used`
- **Deterministic**: no real network calls, wall-clock dependence, or randomness
- **Spec traceability**: every test maps to a spec section in `tests/spec-traceability.md`

## Running

```sh
cargo test -p stencila-codemode
```
