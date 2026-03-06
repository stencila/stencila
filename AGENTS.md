Stencila is a platform for authoring, collaborating on, and publishing executable documents. The codebase is primarily Rust, with TypeScript and Python bindings.

# Project Structure

- `schema/` — YAML schema definitions for document node types
- `rust/` — Rust workspace with 100+ crates, key ones include:
  - `cli` — CLI entry point (`stencila-cli`)
  - `cli-utils` — CLI utilities including the `message!` macro
  - `schema` — generated Rust types from `schema/` YAML files
  - `schema-gen` — code generator for Rust, TypeScript, and Python types
  - `codecs`, `codec-*` — document format encoders/decoders
  - `kernels`, `kernel-*` — execution kernels (Python, R, Node.js, etc.)
  - `models`, `models-*`, `model` — LLM provider integrations
  - `document` — core document model
  - `server` — HTTP/WebSocket server
  - `lsp` — Language Server Protocol implementation
  - `tui` — terminal UI
- `ts/` — Generated TypeScript types
- `node/` — Node.js bindings
- `python/` — Python bindings
- `web/` — Web frontend (TypeScript)

# Rust Development

Write idiomatic Rust:

- Prefer enums with pattern matching over booleans or magic numbers
- Use `&str` and `&[T]` for borrowed data, `String` and `Vec<T>` for owned data
- Prefer iterator adapters (`map`, `filter`, `collect`) over manual loops
- Use `if let` chains where appropriate (supported in Rust edition 2024)
- Use `Option`/`Result` and the `?` operator
- Do not use `unwrap()` in production and tests
- Avoid `expect()` and `panic!()` where possible
- In tests, prefer functions that return `Result<()>` and use `?`; use `expect()` only when it adds a clear failure message and there is no cleaner alternative
- Derive common traits (`Debug`, `Clone`, `Eq`, `IntoIterator`) where appropriate
- Use workspace utility crates to reduce boilerplate:
  - `derive_more` for `Display`, `Deref`, `DerefMut`, `IntoIterator` derives
  - `strum` for enum-string conversions (`EnumString`, `Display`, `EnumIter`, etc.)
  - `Inflector` for case conversions (e.g., `to_snake_case()`, `to_camel_case()`)
  - `smart-default` for `#[derive(SmartDefault)]` with `#[default]` field attributes
  - `serde_with` for custom serialization helpers
  - `thiserror` for ergonomic error type definitions

After making changes to a crate, run clippy, format, and test:

```sh
cargo clippy --fix --allow-dirty --all-targets -p <crate>
cargo fmt -p <crate>
cargo test -p <crate>
```

If a change affects shared crates, public APIs, generators, or multiple packages, widen verification beyond a single crate and use the nearest workspace or package `Makefile` target.

In doc comments, do not backtick-wrap proper names like OpenAI, Anthropic, DeepSeek, etc. Backticks create intra-doc links and produce warnings for non-Rust items.

# Package-Specific Development

Use the nearest package `Makefile` target where possible:

- `rust/`: `make -C rust fix`, `make -C rust lint`, `make -C rust test`
- `node/`: `make -C node fix`, `make -C node lint`, `make -C node test`
- `python/stencila/`: `make -C python/stencila fix`, `make -C python/stencila lint`, `make -C python/stencila test`
- `web/`: `make -C web fix`, `make -C web lint`, `make -C web test`

For changes that cross package boundaries, prefer the top-level `make fix`, `make lint`, and `make test` targets.

# Schema Changes

To modify the Stencila Schema, edit the relevant YAML file in `schema/`. Ensure any additions are consistent with existing properties in other schema types. Then regenerate types:

```sh
cargo run -p stencila-schema-gen --no-default-features
```

# Generated Files

Do not edit generated files directly unless the task explicitly requires it. Edit the source of truth first, then regenerate outputs.

For changes affecting generated code or derived docs, use the narrowest command that updates the relevant outputs. If unsure, run:

```sh
make generated
```

# Commit Messages

This repo uses [Conventional Commits](https://www.conventionalcommits.org/). Every commit message must follow the format:

```
<type>(<scope>): <description>
```

## Types

- `feat` — a new feature or user-visible behavior
- `fix` — a bug fix
- `refactor` — code restructuring with no behavior change
- `chore` — build, CI, dependency updates, or other housekeeping
- `docs` — documentation-only changes
- `test` — adding or updating tests
- `perf` — performance improvements
- `ci` — CI configuration changes

## Scopes

The scope is the crate or package name most affected by the change — use the short name, not the `stencila-` prefix (e.g., `agents`, `codec-markdown`, `tui`, `web`). For cross-cutting changes use a comma-separated list (e.g., `feat(agents,interviews,tui): ...`). For changes that don't map to a single crate, use a general scope like `rust`, `deps`, or `docs`. A scope is strongly preferred but may be omitted for truly global changes (e.g., `fix: resolve clippy warnings across crates`).

## Description

- Use the imperative mood ("add X", not "added X" or "adds X")
- Start with a lowercase letter
- Do not end with a period
- Keep the first line under ~72 characters

# CLI Development

In `stencila-cli`, prefer `message!` and `message` from `stencila_cli_utils` over `eprintln!` for user-facing output. The `message!` macro provides formatting, text wrapping, and consistent styling. Use `eprint!` only for same-line prompts where user input follows immediately.
