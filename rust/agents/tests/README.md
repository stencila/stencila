# Test Structure

Tests are organized by spec section, following the same convention as `stencila-models3`.

## Test Files

| File | Spec Section | Phase | Description |
|---|---|---|---|
| `spec_1_types.rs` | 2.1-2.4, 2.9, 4.1, App B | 1 | Core types, error hierarchy, serde round-trips |
| `spec_5_truncation.rs` | 5.1-5.3 | 2 | Tool output truncation: char-based, line-based, pipeline |
| `spec_4_execution.rs` | 4.1-4.2, 5.4 | 3 | Execution environment: file ops, command exec, env filtering, search |
| `spec_2_events.rs` | 2.9 | 4 | Event system: emitter/receiver channel, all 13 event kinds, ordering, drop semantics |
| `spec_3_registry.rs` | 3.8 | 5 | Tool registry: register/unregister, lookup, execute, argument validation |
| `spec_3_tools.rs` | 3.3, 3.6 | 5b+6a | Core tool implementations: executors, schema parity, registration |
| `spec_3_patch.rs` | App A | 6b | apply_patch tool: v4a parser (success + parse failures), applicator, executor |
| `spec_3_profiles.rs` | 3.1-3.7 | 7a | Provider profiles: tool sets, capability flags, timeout defaults, schema parity |
| `spec_6_prompts.rs` | 6.1-6.5 | 7b | System prompts: environment context, git context, project docs, prompt assembly |

Files for future phases (added as implemented):
- `spec_2_loop.rs` — Session and agentic loop (2.1, 2.5-2.8, 2.10)
- `spec_7_subagents.rs` — Subagents (7.1-7.4)
- `spec_9_acceptance.rs` — Live integration tests (9.12-9.13)

## Conventions

- **No `unwrap()`**: workspace lints deny `clippy::unwrap_used`. Use `?` with `AgentResult` or `.ok_or()`.
- **Deterministic**: No real network calls, no wall-clock dependence. Mock `Client` and `ExecutionEnvironment`.
- **TDD**: Test file created/updated before source file in each phase.
- **Traceability**: Every test maps to a spec section in `spec-traceability.md`.

## Running Tests

```sh
cargo fmt -p stencila-agents
cargo clippy --fix --allow-dirty --all-targets -p stencila-agents
cargo test -p stencila-agents
```
