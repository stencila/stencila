# Stencila Attractor

An implementation of the [Attractor Specification](https://github.com/strongdm/attractor/blob/main/attractor-spec.md) with extensions for Stencila.

## Usage

```rust
use stencila_attractor::parse_dot;

let dot = r#"
    digraph Pipeline {
        graph [goal="Run tests and report"]
        start [shape=Mdiamond]
        run_tests [label="Run Tests", prompt="Execute the test suite"]
        report [label="Report", prompt="Summarize results"]
        exit [shape=Msquare]

        start -> run_tests -> report -> exit
    }
"#;

let graph = parse_dot(dot).expect("valid DOT");

assert_eq!(graph.name, "Pipeline");
assert_eq!(graph.nodes.len(), 4);
assert_eq!(graph.edges.len(), 3);
```

## Extensions

The following extensions to the spec are implemented.

- **Bare identifier attribute values**: In addition to the spec's quoted strings, the parser accepts bare identifiers (e.g., `shape=Mdiamond`) as string-typed attribute values. This matches standard DOT tooling conventions and reduces quoting noise for simple values.

- **Empty attribute blocks**: The parser accepts `[]` (empty attribute blocks), even though the BNF grammar in §2.2 implies at least one attribute. This is harmless and consistent with standard DOT tooling.

## Deviations

These are intentional deviations from the spec.

- **Nested block comments**: The parser supports nested `/* ... /* ... */ ... */` block comments. The spec does not explicitly define nesting behavior.

- **Reserved keywords as node IDs**: The spec's `Identifier` regex (`[A-Za-z_][A-Za-z0-9_]*`) does not exclude reserved words, but DOT keywords (`graph`, `node`, `edge`, `subgraph`, `digraph`, `strict`) cannot be used as bare node IDs because they create parsing ambiguity (e.g., `node [shape=box]` — defaults statement or node declaration?). This matches standard Graphviz behavior.

- **Bare-key condition clauses**: The §10.2 grammar defines `Clause ::= Key Op Literal` (requiring `=` or `!=`), but the §10.5 evaluation pseudocode has an explicit bare-key branch for truthy checks. This implementation supports bare keys (e.g., `context.flag`) as truthy checks in both evaluation and validation for consistency with the pseudocode.

- **Unquoted condition literals**: Condition literals are compared as plain text without requiring DOT-style `"..."` quoting. Surrounding double quotes on a literal are stripped so that `preferred_label="Fix"` and `preferred_label=Fix` are equivalent. All §10.6 examples use unquoted literals.

- **Reachability severity**: §7.2 lists the `reachability` lint rule at ERROR severity, but the §11.12 parity matrix describes "orphan node → warning". This implementation uses ERROR per §7.2 (the normative validation section).

- **Stylesheet specificity levels**: §11.10 mentions "universal < shape < class < ID" specificity (4 levels), but the §8.2–8.3 grammar defines only 3 selector types (`*`, `.class`, `#id`) with no shape selector. This implementation uses 3-level specificity per the grammar.

- **Wait-for-human unmatched answer**: The §4.6 pseudocode falls back to `choices[0]` when an answer does not match any choice. This implementation returns FAIL instead, to prevent silent misrouting when a human provides an unexpected input. The `find_matching_choice` function documents this deviation.

## Limitations

The following are known limitations of this implementation of the spec.

- **AttrValue serde roundtrip**: `Duration` attribute values serialize as strings (e.g., `"15m"`) and deserialize back as `String` variants through JSON, since `serde_json` cannot distinguish Duration from String in the `#[serde(untagged)]` enum. Duration values are fully preserved within a running pipeline; only JSON serialization/deserialization loses the type distinction.

- **Manager Loop Handler (§4.11)**: `stack.manager_loop` is recognized as a handler type (shape mapping and validation) but no `stack.manager_loop` handler implementation is registered or executed yet.

- **HTTP Server Mode (§9.5)**: No HTTP API/SSE server endpoints are implemented for pipeline control, event streaming, question answering, or state inspection.

- **Console Interviewer (§6.4)**: Auto-approve, callback, queue, and recording interviewers are implemented, but a terminal stdin/stdout `ConsoleInterviewer` is not implemented.

- **Extended Condition Operators (§10.7)**: The condition engine supports `=` / `!=` with `&&` (plus documented bare-key truthy checks). Future operators (`contains`, `matches`, `>`, `<`, `OR`, `NOT`) are not implemented.

- **Parallel `k_of_n`/`quorum` join policies (§4.8)**: Parallel fan-out currently implements only `wait_all` and `first_success`.

- **Fan-in LLM evaluation path (§4.9)**: Fan-in currently uses heuristic candidate ranking only; the prompt-driven LLM evaluation path is not implemented.

- **Fidelity runtime enforcement (§5.3–5.4)**: The fidelity resolution chain (edge → node → graph → default) and resume degradation marker are implemented, but not consumed by the codergen handler. Runtime fidelity behavior requires a real LLM backend; the current simulation backend has no sessions to degrade.

- **Preamble Transform (§9.2)**: No preamble transform is implemented for carrying runtime context into prompts for non-`full` fidelity modes.

- **Event stream adapter (§9.6)**: The events module provides callback, collecting, observer, and broadcast emitters. An async stream adapter for consuming events as a `Stream<Item = PipelineEvent>` is not yet implemented.

- **Incremental JSON for streaming**: Incremental JSON/JSONL output for streaming responses is not implemented.

- **Tool Call Hooks (§9.7)**: `tool_hooks.pre` and `tool_hooks.post` are parsed as attributes but no hook execution is implemented around tool calls.

- **Parity matrix (§11.12)**: The 21 cross-feature integration test cases from the parity matrix are deferred.

- **Integration smoke test (§11.13)**: The end-to-end smoke test with a real LLM callback handler is deferred, pending integration with a real LLM backend.


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
| `tests/spec_4_handlers.rs`  | §4.5, §4.10        | Codergen and tool handlers                            |
| `tests/spec_4_parallel.rs`  | §4.8–4.9           | Parallel fan-out, fan-in, join/error policies         |
| `tests/spec_5_state.rs`     | §5.3–5.5           | Artifacts, fidelity, thread IDs, checkpoint resume    |
| `tests/spec_6_human.rs`     | §4.6, §6           | Interviewers, WaitForHuman handler, accelerator keys  |
| `tests/spec_7_validation.rs`| §7                  | Validation and lint rules                             |
| `tests/spec_8_stylesheet.rs`| §8                  | Model stylesheet parsing and application              |
| `tests/spec_9_transforms.rs`| §9.1–9.4           | Transform trait and built-in transforms               |
| `tests/spec_9_events.rs`    | §9.6                | Event emitters: NoOp, Collecting, Observer, Broadcast |
| `tests/spec_10_conditions.rs`| §10                | Condition expression language                         |

Use the crate workflow below:

```sh
cargo fmt -p stencila-attractor
cargo clippy --fix --allow-dirty --all-targets -p stencila-attractor
cargo test -p stencila-attractor
```
