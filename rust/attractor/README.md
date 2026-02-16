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

### DOT parsing relaxations

The parser accepts several syntax forms beyond the strict spec grammar:

- Bare identifier attribute values (e.g., `shape=Mdiamond`) as string-typed values.
- Empty attribute blocks (`[]`) even though §2.2 BNF implies at least one attribute.
- Qualified keys in top-level graph attr declarations (e.g., `tool_hooks.pre="..."`).
- Unquoted stylesheet values with `/` (e.g., provider/model IDs) in addition to identifier-like characters.

### Nested subgraph class inheritance

For nested subgraphs, if an inner subgraph has no label-derived class, nodes inherit the nearest parent subgraph class. This inheritance behavior for unlabeled nested subgraphs is implemented explicitly.

### Graph-level default thread key

The fidelity/thread resolver supports a graph attribute `default_thread_id` as the graph-level default thread key used in `full` fidelity thread resolution.

### Start and exit ID fallbacks

Start and exit detection accepts canonical ID aliases (`start`/`Start` and `exit`/`Exit`/`end`/`End`) in addition to shape-based detection.

### Human shape alias

`shape=human` is accepted as an alias for `shape=hexagon`, mapping to the `wait.human` handler type. This provides a more intuitive way to declare human-in-the-loop nodes (e.g., `Review [shape=human]`).

### Outcome status-file compatibility alias

Outcome deserialization accepts both `preferred_next_label` (spec field name) and `preferred_label` as input keys for compatibility with legacy/external status producers.

## Deviations

These are intentional deviations from the spec.

### Nested block comments

The parser supports nested `/* ... /* ... */ ... */` block comments. The spec does not explicitly define nesting behavior.

### Reserved keywords as node IDs

The spec's `Identifier` regex (`[A-Za-z_][A-Za-z0-9_]*`) does not exclude reserved words, but DOT keywords (`graph`, `node`, `edge`, `subgraph`, `digraph`, `strict`) cannot be used as bare node IDs because they create parsing ambiguity (e.g., `node [shape=box]` - defaults statement or node declaration?). This matches standard Graphviz behavior.

### Condition expression parsing and evaluation

Condition handling intentionally accepts and normalizes forms beyond strict §10.2 grammar: bare-key clauses (e.g., `context.flag`) are treated as truthy checks, literals can be unquoted (`preferred_label=Fix` equivalent to `preferred_label="Fix"`), and trailing separators (e.g., `outcome=success &&`) are tolerated by ignoring empty clauses. This follows the §10.5 evaluation pseudocode and common authoring patterns.

### Stylesheet specificity levels

§11.10 mentions "universal < shape < class < ID" specificity (4 levels), but the §8.2-§8.3 grammar defines only 3 selector types (`*`, `.class`, `#id`) with no shape selector. This implementation uses 3-level specificity per the grammar.

### Reachability severity

§7.2 lists the `reachability` lint rule at ERROR severity, but the §11.12 parity matrix describes "orphan node -> warning". This implementation uses ERROR per §7.2 (the normative validation section).

### wait.human matching and fallback behavior

Relative to §4.6 pseudocode, `wait.human` both expands matching behavior and changes unmatched-answer fallback: in addition to option-key matching, it accepts case-insensitive full-label matches and maps `Yes` to the first option; when no choice matches, it returns FAIL instead of silently falling back to `choices[0]`.

### Exit-node execution in engine loop

The engine executes the exit handler after goal-gate checks, while §3.2 pseudocode breaks on terminal detection before handler execution.

### Run directory node layout

§5.6 describes node files at `{run_root}/{node_id}/...`. This implementation stores them under `{run_root}/nodes/{node_id}/...` to keep root-level files (`manifest.json`, `checkpoint.json`) separate from node directories.

### Retry default source-of-truth ambiguity (§2.5/Appendix A vs §3.5)

The implementation follows §3.5 behavior and defaults to `0` retries when neither `max_retries` nor `default_max_retry` is set, instead of the `default_max_retry=50` default shown in §2.5 and Appendix A.

### Question type naming ambiguity (§6.2 vs §11.8)

The interviewer model uses the §6.2 enum names (`YES_NO`, `MULTIPLE_CHOICE`, `FREEFORM`, `CONFIRMATION`) rather than the alternate naming shown in §11.8 (`SINGLE_SELECT`, `MULTI_SELECT`, `FREE_TEXT`, `CONFIRM`).

## Limitations

The following are known limitations of this implementation of the spec.

### AttrValue serde roundtrip

`Duration` attribute values serialize as strings (e.g., `"15m"`) and deserialize back as `String` variants through JSON, since `serde_json` cannot distinguish Duration from String in the `#[serde(untagged)]` enum. Duration values are fully preserved within a running pipeline; only JSON serialization/deserialization loses the type distinction.

### Graph attribute mirroring loses non-string types in context (§5.1)

During run initialization, `graph.*` keys are mirrored into context as strings via `to_string_value`, so numeric/boolean graph attrs are not preserved as typed JSON values in context.

### Default registry wiring for dependency-backed handlers (§4.2, §4.6, §4.8)

`EngineConfig::new()` does not auto-register `wait.human` and `parallel` because they require runtime dependencies (`Interviewer`, shared registry/emitter). Pipelines using those node types must register handlers explicitly before running.

### Node `timeout` enforcement is not centralized (§2.6, §4.*)

There is no engine-level timeout wrapper applied uniformly to all handlers. `tool` applies `timeout`; other handlers generally do not enforce node-level timeouts.

### Auto-status synthesis (§2.6, Appendix C)

The `auto_status` node attribute is parsed but not acted upon. When a handler completes without writing `status.json`, the engine does not auto-generate a SUCCESS outcome. In practice, all handlers return an `Outcome` directly, so this is only relevant for future external-process backends that communicate via status files.

### Manager Loop Handler (§4.11)

`stack.manager_loop` is recognized as a handler type (shape mapping and validation) but no `stack.manager_loop` handler implementation is registered or executed yet.

### Extended Condition Operators (§10.7)

The condition engine supports `=` / `!=` with `&&` (plus documented bare-key truthy checks). Future operators (`contains`, `matches`, `>`, `<`, `OR`, `NOT`) are not implemented.

### Advanced branching strategies (§4.8, §4.9)

Parallel fan-out currently implements only `wait_all` and `first_success`, and fan-in currently uses heuristic candidate ranking only; the `k_of_n`/`quorum` joins and prompt-driven LLM fan-in evaluation path are not implemented.

### Fidelity runtime integration gaps (§5.3-§5.4, §9.2)

The fidelity resolution chain (edge -> node -> graph -> default) and resume degradation marker are implemented, but not consumed by the codergen handler, and no preamble transform is implemented for carrying runtime context into prompts for non-`full` fidelity modes. The current simulation backend has no LLM sessions to degrade.

### Tool Call Hooks (§9.7)

`tool_hooks.pre` and `tool_hooks.post` are parsed as attributes but no hook execution is implemented around tool calls.

### HTTP Server Mode (§9.5)

No HTTP API/SSE server endpoints are implemented for pipeline control, event streaming, question answering, or state inspection.

### Event system parity and streaming adapter (§9.6)

The events module provides callback, collecting, observer, and broadcast emitters, but no async stream adapter for consuming events as a `Stream<Item = PipelineEvent>`. Event categories are implemented, but payload fields differ from the spec pseudocode (for example, duration/id-rich payload variants are not modeled as-is).

### Incremental JSON for streaming

Incremental JSON/JSONL output for streaming LLM responses is not implemented (not explicitly required by the spec, but a natural extension for real backends).

### `type_known` custom handler false positives (§7.2, §7.4)

The built-in `type_known` lint rule checks a static list and cannot see runtime-registered custom handlers, so valid custom handler types can still be warned as unknown.

### Interviewer feature gaps (§6.2, §6.4)

Auto-approve, callback, queue, and recording interviewers are implemented, but a terminal stdin/stdout `ConsoleInterviewer` is not implemented, and `Question.metadata` from the spec model is not represented in the current interviewer types.

### Deferred conformance and integration tests (§11.12, §11.13)

The 21 cross-feature integration test cases from the parity matrix are deferred, and the end-to-end smoke test with a real LLM callback handler is deferred pending integration with a real LLM backend.

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
