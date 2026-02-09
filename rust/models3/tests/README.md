# Tests

This directory is organized for spec-driven TDD.

## Layout

- `smoke.rs`: basic harness sanity check.
- `spec_2_client.rs` to `spec_8_acceptance.rs`: conformance-focused integration test files grouped by spec section.
- `spec-traceability.md`: living matrix from spec requirements to concrete test coverage.
- `common/`: shared deterministic test helpers.
- `fixtures/`: static request/response/SSE fixtures.
- `integration/`: env-gated real-provider smoke/parity tests.

## TDD Workflow

1. Add or update a requirement row in `spec-traceability.md`.
2. Write a failing test in the matching `spec_*.rs` file.
3. Implement the minimum code to pass.
4. Add success, failure, and boundary coverage before closing the row.

## Rules

- Prefer fixture-based tests for protocol translation and serialization.
- Keep tests deterministic: no wall-clock dependence, randomness, or live network by default.
- Gate real-provider tests in `tests/integration/` behind explicit environment variables.
