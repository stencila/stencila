# Stencila Models

A Rust implementation of the [Unified LLM Client Specification](https://github.com/strongdm/attractor/blob/main/unified-llm-spec.md). This crate provides a single interface across multiple LLM providers (OpenAI, Anthropic, Google Gemini, and others), allowing provider-agnostic code where switching models requires changing only a single string identifier.

## Usage

### High-level text generation (`generate`)

```rust
use stencila_models3::api::generate::{generate, GenerateOptions};
use stencila_models3::client::Client;

# async fn run() -> stencila_models3::error::SdkResult<()> {
let client = Client::from_env()?;

let result = generate(
    GenerateOptions::new("gpt-5-mini")
        .prompt("Write one sentence about rust safety.")
        .client(&client),
)
.await?;

println!("{}", result.text);
# Ok(())
# }
```

### Low-level provider call (`Client.complete`)

```rust
use stencila_models3::client::Client;
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;

# async fn run() -> stencila_models3::error::SdkResult<()> {
let client = Client::from_env()?;
let request = Request::new("claude-sonnet-4-5-20250929", vec![Message::user("Say hello")]);
let response = client.complete(request).await?;

println!("{}", response.text());
# Ok(())
# }
```

### Streaming events (`stream_generate`)

```rust
use stencila_models3::api::stream::{stream_generate, StreamOptions};
use stencila_models3::client::Client;

# async fn run() -> stencila_models3::error::SdkResult<()> {
let client = Client::from_env()?;
let mut stream = stream_generate(
    StreamOptions::new("gemini-2.5-flash")
        .prompt("List three short facts about Mars.")
        .client(&client),
)
.await?;

while let Some(event) = stream.next_event().await {
    let event = event?;
    if let Some(delta) = event.delta {
        print!("{delta}");
    }
}

# Ok(())
# }
```

### Structured output (`generate_object`)

```rust
use serde_json::json;
use stencila_models3::api::generate_object::{generate_object, GenerateObjectOptions};
use stencila_models3::client::Client;

# async fn run() -> stencila_models3::error::SdkResult<()> {
let client = Client::from_env()?;
let schema = json!({
    "type": "object",
    "properties": { "city": { "type": "string" }, "country": { "type": "string" } },
    "required": ["city", "country"]
});

let result = generate_object(
    GenerateObjectOptions::new("gpt-5-mini", schema)
        .prompt("Return Paris as JSON with city and country fields.")
        .client(&client),
)
.await?;

if let Some(output) = result.output {
    println!("{output}");
}
# Ok(())
# }
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
git --no-pager diff -- specs/unified-llm-spec.md
```

4. Convert spec diffs into implementation work:

- Update requirement rows and status in `tests/spec-traceability.md`.
- Add or update failing tests in the matching `tests/spec_*.rs` file(s) first.
- Implement the minimum code changes in `src/` and adapters until tests pass.
- Keep deferred subsections explicit in the `Deferred Items` table if any gaps remain.

5. Run the required crate workflow:

```sh
cargo fmt -p stencila-models3
cargo clippy --fix --allow-dirty --all-targets -p stencila-models3
cargo test -p stencila-models3
```

6. If feature-gated paths changed, also run:

```sh
cargo test -p stencila-models3 --all-features
```

### Updating the model catalog

The curated model catalog lives in `src/catalog/models.json`. At build time, the `build.rs` script can optionally fetch current model listings from provider APIs and append any newly discovered models to this file. This is gated behind the `REFRESH_MODEL_CATALOG` environment variable and requires API keys for the providers you want to refresh:

```sh
REFRESH_MODEL_CATALOG=1 \
  OPENAI_API_KEY=sk-... \
  ANTHROPIC_API_KEY=sk-ant-... \
  GEMINI_API_KEY=AI... \
  MISTRAL_API_KEY=... \
  DEEPSEEK_API_KEY=... \
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

Acceptance tests in `tests/spec_8_acceptance.rs` are env-gated and skip per-provider when live API quota/rate-limit conditions prevent execution.

For test organization and TDD conventions, see `tests/README.md`.
For spec coverage tracking, including deferred conformance gaps, see `tests/spec-traceability.md`.
Current spec-focused coverage includes default-client override behavior (`§2.5`) plus provider translation edge cases such as local image-file paths and reasoning-token mapping (`§8.3`, `§8.5`).
