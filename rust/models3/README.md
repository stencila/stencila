# Stencila Models

An implementation of the [Unified LLM Client Specification](https://github.com/strongdm/attractor/blob/main/unified-llm-spec.md) with extensions for Stencila.

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

## Extensions

The following extensions to the spec are implemented.

### OAuth credential auto-detection

The spec does not define an authentication layer beyond API keys. This crate extends `Client::from_env()` and `Client::from_env_with_auth()` with:

- **`AuthCredential` trait** — abstracts over static API keys and expiring OAuth tokens, with automatic refresh via `OAuthToken`.
- **`AuthOverrides`** — per-provider credential injection for explicit OAuth login flows (via the `stencila-oauth` crate).
- **Claude Code auto-detection** — when no `ANTHROPIC_API_KEY` is set, the client automatically checks the system keyring (`Claude Code-credentials` service) and `~/.claude/.credentials.json` for existing Claude Code OAuth credentials. This lets Claude Code subscribers use Anthropic models out of the box without any extra setup.
- **Codex CLI auto-detection (OpenAI)** — when no `OPENAI_API_KEY` is set, the client checks `~/.codex/auth.json` and uses exchanged `OPENAI_API_KEY` values when present; otherwise it requires OAuth tokens that already include `api.responses.write`.

Credential precedence for Anthropic in `from_env()`:

1. `ANTHROPIC_API_KEY` (env/keyring) — `x-api-key` header
2. Claude Code credentials (keyring, then file) — `Authorization: Bearer` with OAuth beta headers
3. Neither — Anthropic provider not registered

Credential precedence for OpenAI in `from_env()`:

1. `OPENAI_API_KEY` (env/keyring) — `Authorization: Bearer` API key
2. Codex CLI credentials (`~/.codex/auth.json`) — exchanged `OPENAI_API_KEY` (preferred) or OAuth bearer token with `api.responses.write`, plus optional `ChatGPT-Account-Id`
3. Neither — OpenAI provider not registered

### Secrets integration

When compiled with the `secrets` feature, `get_secret` reads API keys from environment variables first, then the OS keyring via `stencila-secrets`; without it, env vars are used only.

### Command line interface

When compiled with the `cli` feature, this crate exposes CLI access to model listing/model info parts of the spec, plus generation workflows (primarily for testing and validation).
For full command and option details, run `stencila models --help`.

### Additional providers beyond the base spec

In addition to OpenAI / Anthropic / Gemini, this crate also ships adapters for Mistral, DeepSeek, and Ollama (all through their native or OpenAI-compatible APIs).

### Provider allowlist and priority from `stencila.toml`

`Client::from_env()` and `Client::from_env_with_auth()` support `models.providers` configuration in `stencila.toml` to constrain which providers are registered and to control provider selection priority.

### Anthropic prompt-caching auto-marking and beta aliasing

The Anthropic adapter automatically injects `cache_control: { "type": "ephemeral" }` markers on cacheable request blocks and enables the `prompt-caching-2024-07-31` beta header by default (`auto_cache: true`). It also accepts `provider_options.anthropic.beta_features` as an alias of `beta_headers`.

### Tool-loop step boundary stream event

During multi-step tool execution in `stream_generate()`, this crate emits an additional `step_finish` stream event between model rounds. This event includes the completed step's `finish_reason` and `usage` and is intended to make tool-loop boundaries explicit to stream consumers.

## Deviations

These are intentional deviations from the spec.

### `ToolChoice` shape

- Spec shape: `{ mode, tool_name? }` record
- Rust shape: `enum { Auto, None, Required, Tool(String) }`
- Rationale: invalid states are unrepresentable in Rust; adapters translate to wire format.

### `ProviderAdapter` stream shape

- Spec shape: `Stream<StreamEvent>`
- Rust shape: `Future<Result<Stream<Result<StreamEvent>>>>`
- Rationale: two-phase stream setup separates connection-time failures from in-stream event failures.

### High-level stream function naming

- Spec naming: `stream()`
- Rust naming: `stream_generate()`
- Rationale: avoids collision with low-level `Client::stream()`.

### High-level streaming result API shape

- Spec shape: returned value is directly an async iterator over stream events; `partial_response: Response | None`; `response() -> Response` after stream end
- Rust shape: returns `StreamResult` with `next_event()` / `collect()` / `text_stream()` helpers, `partial_response() -> Response`, and `response() -> Option<Response>`
- Rationale: keeps accumulation state and convenience accessors on one owned handle, avoids optional handling in hot loops for partial snapshots, and makes completion state explicit for final responses.

### Provider resolution when `provider` is omitted

- Spec behavior (`§2.2`): when `request.provider` is omitted, use `default_provider` and do not guess.
- Rust behavior: attempts provider inference from model ID/alias in the catalog before falling back to default provider.
- Rationale: makes provider-agnostic calls work when model names or aliases are unambiguous.

## Limitations

These are known limitations of this implementation of the spec.

### Structured output gaps (`§4.5-§4.6`)

`stream_object()` currently collects the full stream and parses once at the end; incremental partial-object parsing is not yet implemented. Anthropic does not natively support `json_schema` response format in the same way as other providers, and fallback strategies (system-prompt shaping or tool-based extraction) are not yet implemented.

### StreamAccumulator multi-segment support (`§4.4`)

Accumulator behavior currently assumes a single in-flight segment per kind and does not fully support concurrent/interleaved multi-segment assembly.

### Timeout coverage gaps (`§4.7`)

`stream_generate()` does not enforce `total` timeout internally because streams are lazy; callers should wrap stream consumption in `tokio::time::timeout`. `per_step` timeout currently applies to provider stream connection setup, not per-event reads after connection is established. `Timeout.connect` and `Timeout.stream_idle` exist on the public `Timeout` type but are not currently enforced as request-level overrides in provider execution paths.

### No implicit latest-model selection (`§2.9`)

The public `Request` / high-level option builders require an explicit model string; they do not currently auto-select a provider's latest model when no model is provided.

### No sync wrapper API (`§2.6`)

The crate currently exposes async APIs only (`Client`, `generate`, `stream_generate`, `generate_object`, `stream_object`) and does not provide blocking/sync wrappers.

### Tool execute context injection not implemented (`§5.2`)

Tool execute handlers currently receive only parsed argument JSON; injected context parameters like `messages`, `abort_signal`, and `tool_call_id` are not supported.

### Tool-call repair hook not implemented (`§5.8`)

When tool-call argument parsing/schema validation fails, the current behavior is to return an error `ToolResult`; a configurable `repair_tool_call` flow is not implemented.

### `set_default_client()` leaks replaced clients (`§2.5`)

`set_default_client()` uses `Box::leak` to produce `&'static Client` references. When called more than once, the previously leaked client cannot be safely reclaimed because callers may still hold `&'static` references to it. In practice the leak is bounded (typically 0-2 calls per process lifetime). Switching to `Arc<Client>` would eliminate the leak but is a breaking API change.

## Development

### Workflow

The `make check` recipe performs the workflow:

```sh
cargo clippy --fix --allow-dirty --all-targets -p stencila-models3
cargo fmt -p stencila-models3
cargo test --all-features -p stencila-models3
```

### Updating the spec

A vendored copy of the spec is kept in `specs/` for reference. Use the protocol below when upstream changes. All `make` targets run from `rust/models3/`.

1. Preview upstream changes without mutating the repo:

```sh
make spec-diff
```

2. Vendor the latest spec:

```sh
make spec-update
```

3. Review the vendored diff (for commit / PR context):

```sh
git --no-pager diff -- specs/unified-llm-spec.md
```

If the diff is cosmetic (e.g. typo fixes, link updates, rewording with no new or changed requirements), stop here — no implementation work is needed.

4. Convert spec requirement changes into implementation work:

- Update requirement rows and status in `tests/spec-traceability.md`.
- Add or update failing tests in the matching `tests/spec_*.rs` file(s) first.
- Implement the minimum code changes in `src/` and adapters until tests pass.
- Keep deferred subsections explicit in `## Limitations` if any gaps remain.

5. Run the crate check recipe:

```sh
make check
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

Use the crate check recipe:

```sh
make check
```

Acceptance tests in `tests/spec_8_acceptance.rs` are env-gated and skip per-provider when live API quota/rate-limit conditions prevent execution.

For test organization and TDD conventions, see `tests/README.md`.
For spec coverage tracking, including deferred conformance gaps, see `tests/spec-traceability.md`.
