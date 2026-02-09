# AGENTS.md

## Overview

This crate (`stencila-agents`) is a **greenfield** Rust implementation of the [Coding Agent Loop Specification](https://github.com/strongdm/attractor/blob/main/coding-agent-loop-spec.md). A copy of the spec is in `specs/coding-agent-loop-spec.md`.

It builds on the new `stencila-models3` crate for Stencila v3 and should follow the conventions used there. **Do NOT reference or get distracted by the other sibling `stencila-*` crates**.

## Workflow

After making changes, always run the following commands **in order**:

```sh
cargo fmt -p stencila-agents
cargo clippy --fix --allow-dirty --all-targets -p stencila-agents
cargo test -p stencila-agents
```

1. **Format first** (`cargo fmt`) so that clippy and tests run against consistently formatted code.
2. **Lint second** (`cargo clippy --fix ...`) to auto-fix what it can and surface remaining warnings.
3. **Test last** (`cargo test`) to verify correctness after formatting and lint fixes.

## Documentation

- Keep a concise `Testing` section in the root `README.md` with the canonical command sequence.
- Keep detailed test structure and conventions in `tests/README.md`.
- Keep spec coverage tracking in `tests/spec-traceability.md`.
- When testing workflow or spec-conformance coverage changes, update all three docs in the same change.

## Definition of Done

- Every behavior change includes tests for both success and failure paths.
- Include edge-case coverage for boundary conditions (empty inputs, missing fields, unsupported variants).
- For protocol/serialization changes, add round-trip or fixture-based tests that prove spec conformance.
- Keep tests deterministic: avoid real network calls, wall-clock dependence, and random outcomes unless explicitly integration-gated.
- When modifying feature-gated code paths, also run:

```sh
cargo test -p stencila-agents --all-features
```

## Rust Guidelines

- Write idiomatic Rust:
  - Prefer enums with pattern matching over booleans or magic numbers
  - Use `&str` / `&[T]` for borrowed data, `String` / `Vec<T>` for owned data
  - Prefer iterator adapters (`map`, `filter`, `collect`) over manual loops
  - Use `Option` / `Result` with the `?` operator — avoid `unwrap()`, `expect()`, and `panic!()`, even in tests
  - Clippy has `unwrap_used` deny — always use `?` or `.ok_or()` in tests
  - Derive common traits (`Debug`, `Clone`, `PartialEq`, `Eq`) where appropriate
- Define domain-specific error types using `thiserror` — avoid stringly-typed errors.
- Propagate errors with `?`; only handle them at boundaries where a meaningful recovery or user-facing message is needed.
- Keep the implementation focused on the spec — avoid over-engineering or adding features not required by the specification.
- Regularly review the codebase for opportunities to keep it DRY — extract shared logic into common functions, traits, or helper modules rather than duplicating code across providers or message types.
- Prefer existing workspace dependencies and stdlib facilities before adding new crates.
- Any new dependency should include a brief rationale in the change description.

## Working with the Spec

- **Always consult `specs/coding-agent-loop-spec.md`** before implementing or modifying a feature. The spec is the source of truth.
- If the spec is ambiguous, do not guess; add a comment with the exact spec section and the open question.
- Use this format for ambiguity tracking comments:

```rust
// TODO(spec-ambiguity): <question> (spec: <section-or-heading>)
```
