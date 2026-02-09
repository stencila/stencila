# Stencila Models

A Rust implementation of the [Unified LLM Client Specification](https://github.com/strongdm/attractor/blob/main/unified-llm-spec.md). This crate provides a single interface across multiple LLM providers (OpenAI, Anthropic, Google Gemini, and others), allowing provider-agnostic code where switching models requires changing only a single string identifier.

## Development

### Updating the spec

A vendored copy of the spec is kept in `specs/` for reference. To update it to the latest upstream version:

```sh
make update-spec
```

### Updating the model catalog

The curated model catalog lives in `src/catalog/models.json`. At build time, the `build.rs` script can optionally fetch current model listings from provider APIs and append any newly discovered models to this file. This is gated behind the `REFRESH_MODEL_CATALOG` environment variable and requires API keys for the providers you want to refresh:

```sh
REFRESH_MODEL_CATALOG=1 \
  OPENAI_API_KEY=sk-... \
  ANTHROPIC_API_KEY=sk-ant-... \
  GEMINI_API_KEY=AI... \
  cargo build -p stencila-models3
```

Providers whose keys are absent are silently skipped. Discovered models are appended after curated entries (so curated metadata is preserved and `get_latest_model` continues to prefer them). Models already in the catalog by `(provider, id)` are not duplicated.

### Testing

Use the crate workflow below:

```sh
cargo fmt -p stencila-models3
cargo clippy --fix --allow-dirty --all-targets -p stencila-models3
cargo test -p stencila-models3
```

For test organization and TDD conventions, see `tests/README.md`.
For spec coverage tracking, see `tests/spec-traceability.md`.
