# Stencila Agents

A Rust implementation of the [Coding Agent Loop Specification](specs/coding-agent-loop-spec.md).

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
    let profile = Box::new(AnthropicProfile::new("claude-sonnet-4-5-20250929")?);
    let env = Arc::new(LocalExecutionEnvironment::new("."));
    let client = Arc::new(Models3Client::new(
        stencila_models3::client::Client::from_env()?,
    ));
    let system_prompt = prompts::build_system_prompt(&*profile, &*env).await?;
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
- Keep deferred subsections explicit in the `Deferred Items` table if any gaps remain.

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

| File | Spec Sections | Description |
|---|---|---|
| `spec_1_types.rs` | 2.1-2.4, 2.9, 4.1, App B | Core types, error hierarchy, serde |
| `spec_5_truncation.rs` | 5.1-5.3 | Tool output truncation |
| `spec_4_execution.rs` | 4.1-4.2, 5.4 | Execution environment, file/cmd ops |
| `spec_2_events.rs` | 2.9 | Event system |
| `spec_3_registry.rs` | 3.8 | Tool registry |
| `spec_3_tools.rs` | 3.3, 3.6 | Core tool implementations, schema parity |
| `spec_3_patch.rs` | App A | apply_patch tool: v4a parser (success + parse failures), applicator, executor |
| `spec_3_profiles.rs` | 3.1-3.7 | Provider profiles: tool sets, capability flags, timeout defaults, schema parity |
| `spec_6_prompts.rs` | 6.1-6.5 | System prompts: environment context, git context, project docs, prompt assembly |
| `spec_2_loop.rs` | 2.1, 2.5-2.8, 2.10, App B | Session and agentic loop: tool execution, steering, follow-up, loop detection, error handling, parity |
| `spec_7_subagents.rs` | 7.1-7.4 | Subagent lifecycle: spawn, send_input, wait, close_agent, depth limiting, auto-registration |
| `spec_9_acceptance.rs` | 9.12-9.13 | Live integration tests: parity matrix + smoke tests (env-gated) |

Use the crate workflow below:

```sh
cargo fmt -p stencila-agents
cargo clippy --fix --allow-dirty --all-targets -p stencila-agents
cargo test -p stencila-agents
```
