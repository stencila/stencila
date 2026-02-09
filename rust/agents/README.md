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

Test files map to spec sections (`spec_1_types.rs` covers spec 2.1-2.4 types, etc.). See `tests/README.md` for details and `tests/spec-traceability.md` for the full mapping.

Use the crate workflow below:

```sh
cargo fmt -p stencila-agents
cargo clippy --fix --allow-dirty --all-targets -p stencila-agents
cargo test -p stencila-agents
```
