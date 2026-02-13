# Tests

## Structure

Test files are named `spec_N_<topic>.rs` where `N` maps to the primary spec section being tested. Each file covers a coherent set of behaviors from the [Attractor Specification](../specs/attractor-spec.md).

| File                       | Spec Sections       | Description                                           |
| -------------------------- | ------------------- | ----------------------------------------------------- |
| `spec_1_types.rs`          | §2.4, §5.1–5.3, App D | Core types, context, checkpoint                    |
| `spec_2_parser.rs`         | §2.1–2.13, App A–B | DOT parser and graph model                            |
| `spec_3_engine.rs`         | §3.1–3.8, §4.1–4.4 | Edge selection, engine core, retry, basic handlers    |
| `spec_4_handlers.rs`       | §4.5, §4.10        | Codergen, tool handlers, variable expansion           |
| `spec_4_parallel.rs`       | §4.8–4.9           | Parallel fan-out and fan-in                           |
| `spec_5_state.rs`          | §5.4–5.5           | Fidelity modes, artifact store                        |
| `spec_5_resume.rs`         | §5.3               | Checkpoint resume                                     |
| `spec_6_human.rs`          | §6, §4.6           | Human-in-the-loop, interviewer implementations        |
| `spec_7_validation.rs`     | §7                  | Validation and lint rules                             |
| `spec_8_stylesheet.rs`     | §8                  | Model stylesheet parsing and application              |
| `spec_9_transforms.rs`     | §9.1–9.4           | Transform trait and built-in transforms               |
| `spec_9_events.rs`         | §9.6               | Observability events                                  |
| `spec_10_conditions.rs`    | §10                 | Condition expression language                         |
| `spec_11_acceptance.rs`    | §11.12–11.13       | Cross-feature parity matrix and integration smoke test|

## Conventions

- **TDD workflow:** Write failing tests first, then implement to pass.
- **No `unwrap()` or `expect()`:** Clippy denies `unwrap_used`. Use `?` with `-> Result<(), Box<dyn std::error::Error>>` or `.ok_or("message")?`.
- **Deterministic tests:** No real network calls, wall-clock dependence, or random outcomes unless explicitly integration-gated.
- **Integration gating:** Tests requiring API keys or external services check for the key via env var and skip silently when absent. Use `#[ignore]` or a guard function at the top of the test.
- **Spec traceability:** Each test should reference the spec section it validates via a comment. See `spec-traceability.md` for the full mapping.

## Running

```sh
# Standard workflow (format, lint, test)
cargo fmt -p stencila-attractor
cargo clippy --fix --allow-dirty --all-targets -p stencila-attractor
cargo test -p stencila-attractor

# With all features
cargo test -p stencila-attractor --all-features
```
