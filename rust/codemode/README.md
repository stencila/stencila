# Stencila Code Mode

An implementation of the concepts in [Code Mode](https://blog.cloudflare.com/code-mode/) from Cloudflare and [Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp) from Anthropic.

## Usage

TODO

## Deviations

### Deviations from spec §3.5

- `setTimeout` delay parameter is accepted but ignored — callbacks fire on the next microtask tick. QuickJS's async runtime cannot easily support real delays within `async_with!`; real delay support deferred to a later phase if needed.
- `setInterval` is explicitly removed (not provided) per spec §3.5.

### Polyfill fidelity (§3.5)

- `URL`: Rust-backed via the `url` crate. Supports `protocol`, `hostname`, `port`, `pathname`, `search`, `hash`, `username`, `password`, `href`, `origin`, `toString()`. Does not support `searchParams` property (use `URLSearchParams` separately).
- `URLSearchParams`: Pure JS implementation. Supports `get`, `set`, `has`, `delete`, `append`, `toString`, `entries`, `keys`, `values`, `forEach`. Does not implement full `Iterator` protocol.
- `TextEncoder`/`TextDecoder`: Rust-backed UTF-8 only. `TextDecoder` does not support non-UTF-8 encodings or streaming decode options.

### API design (not spec-prescribed)

- Dirty server tracking uses `DirtyServerTracker` + `Sandbox::with_dirty_servers()`. The host calls `tracker.mark_changed(server_id)` when receiving `tools/listChanged`, then passes `tracker.dirty()` (or `tracker.take_dirty()`) to `Sandbox::with_dirty_servers()` before the next invocation.
- `Sandbox::new()` (without dirty set) continues to work unchanged — equivalent to passing an empty dirty set.

### API design (not spec-prescribed)

- `codemode_run()` takes `&RunRequest` (borrowed) instead of `RunRequest` (owned) — avoids unnecessary cloning at the call site.
- `dirty_servers` is a separate `&HashSet<String>` parameter (not embedded in `RunRequest`) — separates host-level state tracking from the request schema.

## Limitations

- **`setTimeout` delay is ignored** — callbacks fire on the next microtask tick regardless of the specified delay (see Phase 2 deviations).
- **`setInterval` not provided** — spec §3.5 does not require it.
- **`URL.searchParams` property not supported** — use `new URLSearchParams(url.search)` separately.
- **`URLSearchParams` iterator protocol incomplete** — `for...of` is not supported; use `forEach`, `keys`, `values`, or `entries`.
- **`TextDecoder` UTF-8 only** — non-UTF-8 encodings and streaming decode options are not supported.
- **No host-level parallelism for cross-server tool calls** — `Promise.all` works within the sandbox (JS-level concurrency) but tool calls execute sequentially on the Rust side. True parallel dispatch across servers would require `tokio::spawn` per call.

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
