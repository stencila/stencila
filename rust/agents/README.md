# Stencila Agents

An implementation of the [Coding Agent Loop Specification](https://github.com/strongdm/attractor/blob/main/coding-agent-loop-spec.md) with extensions for Stencila.

## Usage

### Run a session and consume events

Create a profile, execution environment, and models3 client, then submit user
input and drain the resulting events:

```rust,no_run
use std::sync::Arc;

use stencila_agents::{
    execution::LocalExecutionEnvironment,
    profiles::AnthropicProfile,
    prompts,
    session::{Models3Client, Session},
    types::SessionConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = Box::new(AnthropicProfile::new("claude-sonnet-4-5-20250929", 600_000)?);
    let env = Arc::new(LocalExecutionEnvironment::new("."));
    let client = Arc::new(Models3Client::new(
        stencila_models3::client::Client::from_env()?,
    ));
    let system_prompt = prompts::build_system_prompt(&mut *profile, &*env, true).await?;
    let config = SessionConfig::default();

    let (mut session, mut receiver) =
        Session::new(profile, env, client, config, system_prompt, 0);

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

## Deviations

These are intentional deviations from the spec.

### Session command timeout source (`§2.2`)

`SessionConfig.default_command_timeout_ms` exists but profile-specific shell defaults remain the effective source of default timeout behavior (OpenAI/Gemini 10s, Anthropic 120s).

### `follow_up()` processing scope (`§2.8`)

The spec text implies follow-ups run after natural completion (text-only model response). This implementation processes follow-ups after loop exit on both natural-completion and turn-limit paths.

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

## Limitations

The following are known limitations of this implementation of the spec.

### `TOOL_CALL_OUTPUT_DELTA` emission (`§2.9`)

`TOOL_CALL_OUTPUT_DELTA` is defined and the emitter supports it, but the tool execution pipeline does not yet stream tool output incrementally.

### Image tool output support (`§3.3`)

`read_file` returns image data for multimodal providers via the `ToolOutput::ImageWithText` variant. Image content is currently included in tool result messages for Anthropic only; OpenAI and Gemini receive the text placeholder because their adapters do not support non-text content parts in tool-role messages. Images larger than 5 MB fall back to the text placeholder.

### Scoped subagent `working_dir` enforcement (`§7.2`)

`ScopedExecutionEnvironment` enforces `working_dir` at the `ExecutionEnvironment` layer with the following caveats:

- **Shell commands:** `exec_command` sets the working directory to the scope but cannot prevent commands like `cat /etc/passwd` from accessing arbitrary paths via arguments. Full shell sandboxing requires OS-level mechanisms (namespaces, seccomp).

- **Symlinks:** Existing paths are fully canonicalized (symlinks resolved). For non-existent paths (e.g. `write_file` targets), the deepest existing ancestor is canonicalized, catching symlinked intermediate directories. A symlink created after validation but before the inner operation completes is not caught (TOCTOU).

- **Recursive operations:** `list_directory`, `grep`, and `glob_files` post-filter results to remove entries whose real path falls outside scope, but the inner walkers still traverse symlinked directories during collection.


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
