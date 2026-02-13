# Stencila Attractor

An implementation of the [Attractor Specification](https://github.com/strongdm/attractor/blob/main/attractor-spec.md) with extensions for Stencila.

## Usage

TODO

## Extensions

The following extensions to the spec are implemented.

TODO

## Deviations

These are intentional deviations from the spec.

TODO

## Limitations

The following are known limitations of this implementation of the spec.

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
make spec-update
```

3. Generate the repo diff for review and PR context:

```sh
git --no-pager diff -- specs/attractor-spec.md
```

4. Convert spec diffs into implementation work:

- Update requirement rows and status in `tests/spec-traceability.md`.
- Add or update failing tests in the matching `tests/spec_*.rs` file(s) first.
- Implement the minimum code changes in `src/` and adapters until tests pass.
- Keep deferred subsections explicit in `## Limitations` if any gaps remain.

5. Run the required crate workflow:

```sh
cargo fmt -p stencila-attractor
cargo clippy --fix --allow-dirty --all-targets -p stencila-attractor
cargo test -p stencila-attractor
```

6. If feature-gated paths changed, also run:

```sh
cargo test -p stencila-attractor --all-features
```

### Testing

Test files map to spec sections. See `tests/README.md` for details and `tests/spec-traceability.md` for the full mapping.

| File                        | Spec Sections       | Description                                           |
| --------------------------- | ------------------- | ----------------------------------------------------- |
| `tests/spec_1_types.rs`     | §2.4, §5.1–5.3, App D | Core types, context, checkpoint                       |
| `tests/spec_2_parser.rs`    | §2.1–2.13, App A–B | DOT parser and graph model                            |
| `tests/spec_3_engine.rs`    | §3.1–3.8, §4.1–4.4 | Edge selection, engine core, retry, basic handlers    |
| `tests/spec_4_handlers.rs`  | §4.5, §4.10        | Codergen, tool handlers, variable expansion           |
| `tests/spec_4_parallel.rs`  | §4.8–4.9           | Parallel fan-out and fan-in                           |
| `tests/spec_5_state.rs`     | §5.4–5.5           | Fidelity modes, artifact store                        |
| `tests/spec_5_resume.rs`    | §5.3               | Checkpoint resume                                     |
| `tests/spec_6_human.rs`     | §6, §4.6           | Human-in-the-loop, interviewer implementations        |
| `tests/spec_7_validation.rs`| §7                  | Validation and lint rules                             |
| `tests/spec_8_stylesheet.rs`| §8                  | Model stylesheet parsing and application              |
| `tests/spec_9_transforms.rs`| §9.1–9.4           | Transform trait and built-in transforms               |
| `tests/spec_9_events.rs`    | §9.6                | Observability events                                  |
| `tests/spec_10_conditions.rs`| §10                | Condition expression language                         |
| `tests/spec_11_acceptance.rs`| §11.12–11.13      | Cross-feature parity matrix and integration smoke test|

Use the crate workflow below:

```sh
cargo fmt -p stencila-attractor
cargo clippy --fix --allow-dirty --all-targets -p stencila-attractor
cargo test -p stencila-attractor
```
