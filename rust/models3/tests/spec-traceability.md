# Spec Traceability Matrix

Use this matrix to track MUST/SHOULD requirements from `specs/unified-llm-spec.md` to test coverage.

| Spec Section | Requirement Summary | Priority | Test File | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| 2.3 | Middleware applies to both `complete()` and `stream()` | MUST | `tests/spec_2_client.rs` | Planned | Verify onion order and stream wrapping |
| 2.5 | Module-level default client behavior | MUST | `tests/spec_2_client.rs` | Planned | Lazy init + override |
| 2.4 | ProviderAdapter trait defaults and object safety | MUST | `tests/spec_2_client.rs` | Covered | 5 tests: name, close, initialize, supports_tool_choice, object safety |
| 2.9 | Model catalog lookup/list/latest helpers | SHOULD | `tests/spec_2_client.rs` | Covered | 11 tests: lookup, alias, filter, latest, capabilities, unknown capability |
| 3.x | Core type serde round-trips and boundary cases | MUST | `tests/spec_3_types.rs` | Covered | 72 tests: all types, ContentPart validate (Extension fallback detection), ToolDefinition validate (name/desc/params), ToolChoice shape, malformed tool args parse_error |
| 4.1-4.2 | Low-level `Client.complete()` / `Client.stream()` behavior | MUST | `tests/spec_4_generation_streaming.rs` | Planned | No automatic retries |
| 4.3-4.6 | High-level `generate()`, `stream()`, object APIs | MUST | `tests/spec_4_generation_streaming.rs` | Planned | Prompt/messages exclusivity |
| 5.1 | Tool definition validation (name format, description, params root) | MUST | `tests/spec_3_types.rs` | Covered | 4 tests: validate round-trip, bad name, empty desc, non-object params |
| 5.3 | ToolChoice modes and wire shape | MUST | `tests/spec_3_types.rs` | Covered | Rust enum shape documented; adapters translate to wire format. 2 tests: round-trip + shape |
| 5.5 | Active vs passive tools behavior | MUST | `tests/spec_5_tools.rs` | Planned | Passive returns tool calls |
| 5.7 | Parallel tool execution with result ordering | MUST | `tests/spec_5_tools.rs` | Planned | Batch continuation |
| 6.4 | HTTP status -> error mapping | MUST | `tests/spec_6_errors_retry.rs` | Covered | 16 tests: status mapping, retryable, classify, serde, unknown-status retryable agreement, error_code preservation |
| 6.6 | Retry policy constraints | MUST | `tests/spec_6_errors_retry.rs` + `src/retry.rs` | Covered | 11 tests: backoff growth, max_delay cap, jitter range, resolve_delay (non-retryable, exhausted, retry-after override/exceed, zero retries), retry fn (success, transient, exhausted, non-retryable, callback) |
| 6.7 | Rate limit header parsing | SHOULD | `src/http/headers.rs` | Covered | 7 tests: OpenAI headers, Anthropic headers, partial, missing, Retry-After int/float/invalid/missing, reset_at |
| 7.7 | SSE parsing (all 5 line types + [DONE]) | MUST | `src/http/sse.rs` | Covered | 18 tests: simple data, event type, multi-line, multiple events, comments, retry, retry persistence, invalid retry, [DONE], empty data, no-space, blank lines, CRLF, chunked, Anthropic-style, OpenAI-style, unknown fields, trailing-newline-less |
| 7.x | Provider request/response/stream/error translation | MUST | `tests/spec_7_adapters.rs` | Covered | 32 tests across all 4 adapters: OpenAI Responses (request/response/stream/error + provider_options + non-object rejection), Chat Completions (request/response/stream/error + stream-error event + builtin-tools guard via openai + builtin-tools guard via adapter options), Anthropic (request/response/stream/error + thinking signature round-trip + auto-cache conversation prefix + provider_options beta_headers + beta_features alias + cache tokens), Gemini (request/response/stream/error + stream-error event + TextEnd on finishReason + thinking + function calls + provider_options passthrough + non-object rejection + unknown tool_call_id rejection) |
| 7.10 | OpenAI-compatible adapter uses Chat Completions | MUST | `tests/spec_7_adapters.rs` | Covered | Verifies Chat Completions wire shape and rejects Responses-only built-in tools |
| 8.x | Validation checklist and cross-provider parity | MUST | `tests/spec_8_acceptance.rs` | Planned | Env-gated integration |

## Intentional Spec Deviations

| Area | Spec Shape | Rust Shape | Rationale |
| --- | --- | --- | --- |
| ToolChoice | `{ mode, tool_name? }` record | `enum { Auto, None, Required, Tool(String) }` | Makes invalid states unrepresentable; adapters translate to wire format |
| ProviderAdapter::stream | Returns `Stream<StreamEvent>` | Returns `Future<Result<Stream<Result<StreamEvent>>>>` | Two-phase: outer Future for connection, inner Stream for events. Distinguishes connection-time vs streaming failures |

## Status Values

- `Planned`: requirement identified, tests not implemented yet.
- `In Progress`: at least one failing/passing test exists; coverage incomplete.
- `Covered`: success/failure/boundary behavior implemented and passing.
