# Stencila Agents

An implementation of the [Coding Agent Loop Specification](https://github.com/strongdm/attractor/blob/main/coding-agent-loop-spec.md) with extensions for Stencila.

## Usage

### Run a session and consume events

Create a session with `create_session`, submit user input, and drain the
resulting events:

```rust,no_run
use stencila_agents::convenience::create_session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an agent session
    let (mut session, mut receiver) = create_session(None, None).await?;

    // Submit a request
    session.submit("Create hello.py that prints 'Hello World'").await?;
    session.close();

    // Drain events
    while let Some(event) = receiver.recv().await {
        println!("{:?}: {:?}", event.kind, event.data);
    }

    Ok(())
}
```

### Steer a running session

Inject a steering message before or during the agentic loop. Steering messages
are appended to the next LLM request and emit a `SteeringInjected` event:

```rust,ignore
session.steer("Use Python 3 type hints in all new code.");
session.submit("Refactor utils.py to use dataclasses").await?;
```

### Abort a session

Use `AbortController` to cancel a running session from another task:

```rust,ignore
use stencila_agents::session::AbortController;

let controller = AbortController::new();
session.set_abort_signal(controller.signal());

// In another task:
controller.abort();
```

## Extensions

The following extensions to the spec are implemented.

### AwaitingInput auto-detection (`§2.3`)

The spec defines a `PROCESSING → AWAITING_INPUT` transition but does not specify a detection mechanism. This implementation adds heuristic detection: when the model produces a text-only response (no tool calls) whose last line ends with `?` or begins with a solicitation phrase ("Would you like...", "Shall I...", "Let me know..."), the session transitions to `AwaitingInput` instead of `Idle`. Detection only runs on natural completions, not limit-triggered exits. The host can disable this via `SessionConfig::auto_detect_awaiting_input = false` and use the manual `set_awaiting_input()` API instead.

### MCP direct tool registration (`feature = "mcp"`)

The spec mentions MCP as a natural extension for registering tools from external servers. This implementation discovers MCP server configurations from multiple sources (Stencila, Claude, Codex, Gemini config files), connects to them via a `ConnectionPool`, and registers each tool directly in the agent's `ToolRegistry` with namespaced names (`mcp__<server_id>__<tool_name>`). The LLM sees and calls each MCP tool individually, the same way it calls built-in tools like `read_file` or `shell`. This mode is straightforward but can overwhelm the LLM's tool selection when many servers expose many tools. Controlled by `SessionConfig::enable_mcp` (default `false`).

### Codemode: sandboxed MCP orchestration (`feature = "codemode"`)

Instead of exposing every MCP tool individually, codemode registers a single `codemode` tool. The LLM writes JavaScript (ES module syntax) that imports typed functions from `@codemode/servers/<server_id>` modules and orchestrates multiple MCP calls in a sandboxed QuickJS environment. TypeScript declarations describing every available server and tool are included in the system prompt (budget-capped at 4000 characters, falling back to a summary with runtime discovery via `@codemode/discovery`).

Advantages over direct MCP registration:

- **Scales to many tools.** The LLM sees one tool regardless of how many MCP servers and tools are available, avoiding tool-selection confusion.
- **Composable.** A single `codemode` call can chain multiple MCP calls, filter results, transform data, and return a structured answer --- work that would otherwise require many sequential tool-call rounds.
- **Sandboxed.** Code runs in QuickJS with configurable timeouts, memory limits, and tool-call caps. No filesystem or network access beyond the MCP servers.
- **Observable.** The response includes structured diagnostics, console logs, and a redacted tool-call trace for debugging.

Controlled by `SessionConfig::enable_codemode` (default `false`). Both modes can be enabled simultaneously --- direct MCP tools for simple one-shot calls, codemode for complex orchestration.

When MCP and/or codemode are enabled in a parent session, spawned subagents inherit the parent's shared MCP connection pool (and codemode dirty-server tracker when enabled) instead of rediscovering/reconnecting servers. This keeps tool availability consistent across parent/child sessions and avoids duplicated server startup overhead.

### Workspace skills (`feature = "skills"`)

The spec identifies skills as a natural extension point for reusable prompts. This implementation discovers skill files (markdown with YAML frontmatter) from `.stencila/skills/` and provider-specific directories (e.g. `.claude/skills/` for Anthropic). Compact metadata for all discovered skills is included in the system prompt, and a `use_skill` tool is registered so the LLM can load a skill's full instructions on demand. This progressive-disclosure approach keeps the system prompt small while making the complete skill library accessible. Controlled by `SessionConfig::enable_skills` (default `true`).

### Reasoning stream events (`§2.9`)

In addition to the spec's assistant text lifecycle events, this implementation emits `ASSISTANT_REASONING_START`, `ASSISTANT_REASONING_DELTA`, and `ASSISTANT_REASONING_END` when provider streams include reasoning tokens.

### Streaming-first request path with fallback (`§2.9`)

The core loop prefers `Client::stream()` for incremental `ASSISTANT_TEXT_DELTA` delivery and falls back to `Client::complete()` when streaming is unsupported by the active provider/model. In fallback mode, a synthesized single text-delta event is emitted so downstream consumers can keep one event-processing path.

### Forward-compatible reasoning effort values (`§2.2`, `§2.7`)

The spec enumerates `"low" | "medium" | "high" | null`. This implementation additionally accepts arbitrary strings via `ReasoningEffort::Custom(String)` for provider-specific or future effort levels.

### Explicit line-limit overrides in `SessionConfig` (`§5.3`)

The truncation pipeline in `§5.3` references `config.tool_line_limits`, but the `SessionConfig` record in `§2.2` does not define it. This implementation adds `SessionConfig::tool_line_limits: HashMap<String, usize>` so hosts can override per-tool line caps (`shell`, `grep`, `glob`) without modifying character limits.

### Session-level user instruction override field (`§6.1` layer 5)

The spec defines a fifth prompt layer ("User instructions override") but does not define where that value comes from in `SessionConfig`. This implementation adds `SessionConfig::user_instructions: Option<String>`, appended last in `Session::new()` so hosts can provide explicit per-session override text.

### `delete_file` on `ExecutionEnvironment` (App A support)

To support `apply_patch` delete operations cleanly, this implementation extends `ExecutionEnvironment` with `delete_file(path)`. This operation is used by the local environment and patch applicator but is not part of the core interface list in `§4.1`.

### Deterministic and non-blocking event receiver APIs (`§2.9`)

The spec requires async event delivery but does not define testing/consumption helpers. This implementation additionally exposes `events::channel_with_id(...)` for deterministic session IDs in tests and `EventReceiver::try_recv()` for non-blocking polling.

### Scoped subagent prompt guidance (`§7.2`)

When `spawn_agent` includes `working_dir`, the child system prompt is augmented with explicit scope instructions ("You are scoped to the subdirectory..."). This extra guidance is not required by the spec but improves model behavior by reinforcing directory boundaries in addition to runtime `ScopedExecutionEnvironment` checks.

### `create_session` accepts `google` as a Gemini alias

The convenience constructor accepts both `"gemini"` and `"google"` provider names and maps them to the Gemini profile. The core provider profile ID remains `"gemini"`.

## Deviations

These are intentional deviations from the spec.

### Session command timeout source (`§2.2`)

`SessionConfig.default_command_timeout_ms` exists but profile-specific shell defaults remain the effective source of default timeout behavior (OpenAI/Gemini 10s, Anthropic 120s).

### Surfaced SDK errors always close the session (`§2.8`, App B)

The implementation treats any SDK error that reaches `Session` as unrecoverable and transitions to `Closed`, including retryable classes (e.g. rate limit / network) after SDK-level retries are exhausted.

### `follow_up()` processing scope (`§2.8`)

The spec text implies follow-ups run after natural completion (text-only model response). This implementation processes follow-ups after loop exit on both natural-completion and turn-limit paths.

### `max_turns` counting semantics (`§2.5`)

The pseudocode uses `count_turns(session)` without defining whether this means history entries or LLM cycles. This implementation counts LLM request/response cycles (`total_turns`) rather than raw history entries, so assistant/tool-heavy exchanges do not consume the turn budget as quickly.

### Tool descriptions prompt layer (`§6.1`)

The spec's layered prompt includes a dedicated tool-descriptions layer. This implementation passes tool schemas via the API `tools` parameter instead of serializing them into the system prompt.

### `SESSION_END` emission timing (`§2.9`)

The pseudocode emits `SESSION_END` at each loop completion, but this implementation emits it only when a session is actually closed (close/error/abort), to avoid noisy lifecycle events for normal idle transitions.

### Context warning event kind (`§5.5`)

The spec pseudocode uses a `WARNING` event, but the spec event enum does not define that kind. This implementation emits `ERROR` events with `"severity": "warning"` for context-usage warnings.

### `send_input` target status (`§7.2`)

Spec wording says `send_input` targets a running subagent. This implementation accepts any non-failed agent (including completed ones) so that follow-up messages can be sent after the initial task finishes.

### Gemini grounding configuration (`§3.6`)

The spec says Gemini provider options should configure safety settings and grounding. Safety settings are configured (`BLOCK_ONLY_HIGH` for all categories), but grounding (`google_search_retrieval`) is intentionally not configured.

### Provider parity strictness (`§3.1`)

The spec says the initial provider prompts and tool definitions should be exact byte-for-byte copies of the reference agents. This implementation is provider-aligned but not byte-identical to codex-rs / Claude Code / gemini-cli prompts and harnesses.

### Git commit summary format (`§6.4`)

The spec describes recent commit *subject lines*. This implementation gathers commits with `git log --oneline -10`, which includes abbreviated hashes plus subjects.

### Subagent profile inheritance (`§7.3`)

The spec text says subagents use the parent's `ProviderProfile` (or overridden model). Child sessions currently recreate the provider's base profile and do not inherit parent-registered custom tools.

### Mixed subagent + regular tool calls are forced sequential (`§2.5`, `§7.2`)

When an assistant message contains any subagent tool call (`spawn_agent`, `send_input`, `wait`, `close_agent`), this implementation executes all tool calls in that message sequentially, even if the profile supports parallel tool calls. This preserves subagent-manager borrowing safety but deviates from unconditional parallelization for multi-tool rounds.

### Relative path acceptance for file tools (`§3.3`)

`read_file` and `write_file` schemas describe absolute paths, but the local implementation resolves both absolute and relative paths against `working_directory()`. This is intentionally more permissive than the strict wording in the tool definitions.

### `TURN_LIMIT` event payload shape (`§2.5`, `§2.9`)

The pseudocode examples emit `TURN_LIMIT` with fields like `round` or `total_turns`. This implementation emits a normalized payload `{ "limit_type": "...", "count": N }` for both round and turn limits.

### System prompt layers are snapshot-at-start, not rebuilt per loop (`§2.5`, `§6.1`)

The spec pseudocode builds the system prompt inside each loop iteration. This implementation builds it once at session creation and reuses that snapshot for all subsequent requests in the session.

## Limitations

The following are known limitations of this implementation of the spec.

### Abort permanently closes the session

`AbortController::abort()` transitions the session to `Closed` state, making
future `submit()` calls return `SessionClosed`. A "soft abort" that stops the
current exchange but returns to `Idle` state would enable proper per-exchange
cancellation in multi-turn sessions (e.g. TUI chat). TODO: add a soft-abort
mechanism that cancels the current processing without closing the session.

### `TOOL_CALL_OUTPUT_DELTA` emission (`§2.9`)

`TOOL_CALL_OUTPUT_DELTA` is defined and the emitter supports it, but the tool execution pipeline does not yet stream tool output incrementally.

### Image tool output support (`§3.3`)

`read_file` returns image data for multimodal providers via the `ToolOutput::ImageWithText` variant. Image content is currently included in tool result messages for Anthropic only; OpenAI and Gemini receive the text placeholder because their adapters do not support non-text content parts in tool-role messages. Images larger than 5 MB fall back to the text placeholder.

### Non-image binary files return generic I/O errors (`§3.3`)

`read_file` detects images by extension, but other binary files are read via `read_to_string` and fail as generic I/O decoding errors rather than a distinct "binary file" tool error classification.

### Scoped subagent `working_dir` enforcement (`§7.2`)

`ScopedExecutionEnvironment` enforces `working_dir` at the `ExecutionEnvironment` layer with the following caveats:

- **Shell commands:** `exec_command` sets the working directory to the scope but cannot prevent commands like `cat /etc/passwd` from accessing arbitrary paths via arguments. Full shell sandboxing requires OS-level mechanisms (namespaces, seccomp).

- **Symlinks:** Existing paths are fully canonicalized (symlinks resolved). For non-existent paths (e.g. `write_file` targets), the deepest existing ancestor is canonicalized, catching symlinked intermediate directories. A symlink created after validation but before the inner operation completes is not caught (TOCTOU).

- **Recursive operations:** `list_directory`, `grep`, and `glob_files` post-filter results to remove entries whose real path falls outside scope, but the inner walkers still traverse symlinked directories during collection.

### Knowledge cutoff in environment context (`§6.3`)

The environment block supports a `Knowledge cutoff` line, but session startup currently passes `None`, so this field is omitted.

### Subagent shutdown confirmation (`§7.3`, App B graceful shutdown)

`close_all()` is best-effort and non-blocking because `Session::close()` is synchronous. Child tasks are signaled to abort and may briefly outlive parent closure.

### Subagent events are not surfaced to the parent host (`§7.1`-`§7.4`)

Spawned child sessions have their event receivers dropped immediately, so subagent-internal `SESSION_*`, `ASSISTANT_*`, and `TOOL_CALL_*` events are not available through the parent session's event stream.

### MCP tool-name collisions after sanitization/truncation (`§8` MCP extension)

Direct MCP tool registration sanitizes names to `[a-zA-Z0-9_]` and truncates to 64 characters. Distinct MCP tools that collapse to the same final name are skipped (with a warning), so some connected server tools may be unavailable to the model.

### Execution environment lifecycle hooks are not orchestrated by sessions (`§4.1`)

`ExecutionEnvironment` defines `initialize()` and `cleanup()`, but session startup/shutdown paths do not call them. Custom environments that require explicit lifecycle management must currently handle setup/teardown outside `Session`.

### Explicit `close()` is required for deterministic teardown (`§2.9`, App B graceful shutdown)

`Session` has no `Drop`-based shutdown path. If a host drops a live session without calling `close()`, cleanup that normally runs in `close()` (subagent shutdown, top-level MCP pool shutdown, `SESSION_END` emission) is skipped.

### `send_input` does not interrupt an in-flight initial subagent task (`§7.2`)

Subagent command processing starts only after the initial `spawn_agent` task finishes, so `send_input` cannot steer a still-running first task mid-flight. It waits for initial completion, then sends follow-up input.

### Event buffering is unbounded (`§2.9`)

The event channel uses `tokio::sync::mpsc::unbounded_channel`. If a host keeps the receiver alive but consumes slowly, buffered events can grow without backpressure and increase memory usage.

### Project doc discovery reads full files before budget truncation (`§6.5`)

Project instruction discovery enforces a 32KB final prompt budget, but each discovered file is read in full first (`read_file(..., limit = usize::MAX)`) and only then truncated/filtered into the aggregate budget. Very large instruction files can therefore increase startup I/O and memory before truncation.

### Context-usage warnings are not throttled (`§5.5`)

`check_context_usage()` runs on every loop iteration and emits another warning whenever usage remains above 80% of context window. Long runs near the limit can therefore produce repeated warning events rather than a single threshold-crossing notification.

## Bugs

The following are implementation bugs found in the current codebase. Priority key: `P0` (highest) → `P3` (lowest).

### Project docs truncation can exceed the 32KB budget (`§6.5`) (P3)

When truncation is triggered, `discover_project_docs` enforces remaining content bytes but appends `\n` + truncation marker afterward without including marker bytes in the budget check, so final output can exceed 32KB (`src/project_docs.rs`).

### Head/tail truncation under-reports removed characters for odd limits (`§5.1`) (P3)

`truncate_output` computes `removed = char_count - max_chars` but keeps only `2 * (max_chars / 2)` characters in head/tail mode. For odd `max_chars`, this drops one extra character while reporting a smaller removed count in the warning marker (`src/truncation.rs`).

### Scoped grep path parsing breaks on `:<digits>:` in file names (`§7.2`) (P3)

`extract_grep_path` treats the first `:<digits>:` sequence as the line-number delimiter. For valid Linux file names containing this pattern before the real line number, scoped grep post-filtering can parse the path incorrectly and drop in-scope matches (`src/execution/scoped.rs`).



## Development

### Updating the spec

A vendored copy of the spec is kept in `specs/` for reference. Use the protocol below when upstream changes.

1. Preview upstream changes without mutating the repo:

```sh
make spec-diff
```

2. Vendor the latest spec:

```sh
make update-spec
```

3. Generate the repo diff for review and PR context:

```sh
git --no-pager diff -- specs/coding-agent-loop-spec.md
```

4. Convert spec diffs into implementation work:

- Update requirement rows and status in `tests/spec-traceability.md`.
- Add or update failing tests in the matching `tests/spec_*.rs` file(s) first.
- Implement the minimum code changes in `src/` and adapters until tests pass.
- Keep deferred subsections explicit in `## Limitations` if any gaps remain.

5. Run the required crate workflow:

```sh
cargo fmt -p stencila-agents
cargo clippy --fix --allow-dirty --all-targets -p stencila-agents
cargo test -p stencila-agents
```

6. If feature-gated paths changed, also run:

```sh
cargo test -p stencila-agents --all-features
```

### Testing

Test files map to spec sections. See `tests/README.md` for details and `tests/spec-traceability.md` for the full mapping.

| File                   | Spec Sections             | Description                                                                                           |
| ---------------------- | ------------------------- | ----------------------------------------------------------------------------------------------------- |
| `spec_1_types.rs`      | 2.1-2.4, 2.9, 4.1, App B  | Core types, error hierarchy, serde                                                                    |
| `spec_2_events.rs`     | 2.9                       | Event system                                                                                          |
| `spec_2_loop.rs`       | 2.1, 2.5-2.8, 2.10, App B | Session and agentic loop: tool execution, steering, follow-up, loop detection, error handling, parity |
| `spec_3_patch.rs`      | App A                     | apply_patch tool: v4a parser (success + parse failures), applicator, executor                         |
| `spec_3_profiles.rs`   | 3.1-3.7                   | Provider profiles: tool sets, capability flags, timeout defaults, schema parity                       |
| `spec_3_registry.rs`   | 3.8                       | Tool registry                                                                                         |
| `spec_3_tools.rs`      | 3.3, 3.6                  | Core tool implementations, schema parity                                                              |
| `spec_4_execution.rs`  | 4.1-4.2, 5.4, 7.2         | Execution environment, file/cmd ops, scoped working_dir                                               |
| `spec_5_truncation.rs` | 5.1-5.3                   | Tool output truncation                                                                                |
| `spec_6_prompts.rs`    | 6.1-6.5                   | System prompts: environment context, git context, project docs, prompt assembly                       |
| `spec_7_subagents.rs`  | 7.1-7.4                   | Subagent lifecycle: spawn, send_input, wait, close_agent, depth limiting, auto-registration           |
| `spec_9_acceptance.rs` | 9.12-9.13                 | Live integration tests: parity matrix + smoke tests (env-gated)                                       |

Use the crate workflow below:

```sh
cargo fmt -p stencila-agents
cargo clippy --fix --allow-dirty --all-targets -p stencila-agents
cargo test -p stencila-agents
```
