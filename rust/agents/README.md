# Stencila Agents

A Rust implementation of the [Coding Agent Loop Specification](specs/coding-agent-loop-spec.md).

## Usage

TODO

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

Use the crate workflow below:

```sh
cargo fmt -p stencila-agents
cargo clippy --fix --allow-dirty --all-targets -p stencila-agents
cargo test -p stencila-agents
```
