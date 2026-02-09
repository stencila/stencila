# Stencila Models

A Rust implementation of the [Unified LLM Client Specification](https://github.com/strongdm/attractor/blob/main/unified-llm-spec.md). This crate provides a single interface across multiple LLM providers (OpenAI, Anthropic, Google Gemini, and others), allowing provider-agnostic code where switching models requires changing only a single string identifier.

## Development

A vendored copy of the spec is kept in `specs/` for reference. To update it to the latest upstream version:

```sh
make update-spec
```

## Testing

Use the crate workflow below:

```sh
cargo fmt -p stencila-models3
cargo clippy --fix --allow-dirty --all-targets -p stencila-models3
cargo test -p stencila-models3
```

For test organization and TDD conventions, see `tests/README.md`.
For spec coverage tracking, see `tests/spec-traceability.md`.
