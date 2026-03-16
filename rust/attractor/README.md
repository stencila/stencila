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

### Attribute key case normalization

Attribute keys in node, edge, and graph attribute blocks are normalized to `snake_case` at parse time. Users can write keys in kebab-case (`max-retries`), `snake_case` (`max_retries`), or camelCase (`maxRetries`) — all three resolve to the same canonical `snake_case` key in the parsed graph.

This applies to both bare keys and dotted qualified keys:

```dot
// All equivalent after parsing:
A [max-retries=3, goal-gate=true]
A [max_retries=3, goal_gate=true]
A [maxRetries=3, goalGate=true]

// Dotted keys normalize each segment independently:
A [agent.reasoning-effort="high"]   // → agent.reasoning_effort
A [agent.trustLevel="high"]         // → agent.trust_level
```

Kebab-case is the recommended convention for consistency with Stencila skill, agent, and workflow configuration. All documentation examples use kebab-case.

Internal lookup code uses the spec canonical `snake_case` form exclusively, so adding support for a new attribute only requires a single `get_attr("snake_case")` call.

### Nested subgraph class inheritance

For nested subgraphs, if an inner subgraph has no label-derived class, nodes inherit the nearest parent subgraph class. This inheritance behavior for unlabeled nested subgraphs is implemented explicitly.

### Graph-level default thread key

The fidelity/thread resolver supports a graph attribute `default_thread_id` as the graph-level default thread key used in `full` fidelity thread resolution.

### Start, exit, and fail ID fallbacks

Start, exit, and fail detection accepts canonical ID aliases (`start`/`Start`, `exit`/`Exit`/`end`/`End`, and `fail`/`Fail`) in addition to shape-based detection.

### Fail node

The attractor spec has no dedicated node type for declaring explicit failure. The spec's failure mechanisms (§3.7) cover *routing after* a handler fails, but the only way to produce a FAIL outcome is through a handler that actually errors (e.g., a shell node with a non-zero exit code). This means pipelines that need an explicit failure path must use workarounds like `type="shell", shell_command="exit 1"`.

The `fail` handler type fills this gap by providing a first-class failure node, mirroring the `start`/`exit` pattern. A node is recognized as a fail node by shape (`invtriangle`), by ID (`fail` or `Fail`), or by explicit `type="fail"`. The handler returns `Outcome::fail()`, terminating the pipeline with a failure status.

### Runtime variable expansion in prompts

In addition to the parse-time `$goal` expansion, the codergen handler expands runtime variables at execution time so that each stage can reference the outputs of previously completed stages:

| Variable         | Expands to                                              |
|------------------|---------------------------------------------------------|
| `$last_output`   | Full text of the last stage's output                    |
| `$last_stage`    | Node ID of the last completed stage                     |
| `$last_outcome`  | Outcome status of the last stage (e.g. `success`, `fail`) |

These are expanded per-stage during the engine loop, not at graph parse time.

### Agent node attribute

The `agent` node attribute specifies which Stencila agent should execute the node (e.g., `agent="code-engineer"`). This is not part of the attractor spec; the spec uses the `codergen` handler with a pluggable `CodergenBackend`. The `agent` attribute is a Stencila-specific extension that maps nodes to named agents in the Stencila agent registry.

Agent properties can be overridden per-node using `agent.*` dotted-key attributes:

| Attribute                 | Description                                       |
|---------------------------|---------------------------------------------------|
| `agent.model`             | Override the model (e.g. `"gpt-4o"`)              |
| `agent.provider`          | Override the provider (e.g. `"openai"`)           |
| `agent.reasoning-effort`  | Override reasoning effort (`"low"`, `"medium"`, `"high"`) |
| `agent.trust-level`       | Override trust level (`"low"`, `"medium"`, `"high"`)      |
| `agent.max-turns`         | Override maximum conversation turns (e.g. `"10"`)         |

These take precedence over stylesheet-derived values (`llm_model`, `llm_provider`, `reasoning_effort`, `trust_level`, `max_turns`). When not specified, the agent definition's own values are used as defaults. Example:

```dot
Build [agent="code-engineer", agent.provider="openai", agent.model="o3"]
```

All casing variants are accepted thanks to attribute key normalization (see above). Kebab-case is recommended for consistency with agent configuration.

### Human shape alias

`shape=human` is accepted as an alias for `shape=hexagon`, mapping to the `wait.human` handler type. This provides a more intuitive way to declare human-in-the-loop nodes (e.g., `Review [shape=human]`).

### Extended human question types (`question-type` attribute)

The attractor spec's `wait.human` handler (§4.6) only supports multiple-choice questions derived from outgoing edge labels. This extension adds a `question-type` node attribute that overrides the default question type:

| `question-type` value | Question presented | Routing behavior |
|---|---|---|
| _(absent)_ | Single-select from edge labels | Selected edge (original behavior) |
| `"single-select"` | Single-select from edge labels | Selected edge |
| `"multi-select"` | Multi-select from edge labels | Selected edge |
| `"freeform"` | Free-form text input | First outgoing edge |
| `"yes-no"` | Yes/no binary choice | First outgoing edge |
| `"confirm"` | Confirmation prompt | First outgoing edge |

Both `kebab-case` (recommended) and `snake_case` forms are accepted for values that contain separators. The attribute key itself is normalized to `snake_case` at parse time as usual (so `question-type` in DOT becomes `question_type` internally).

For non-choice types (freeform, yes-no, confirm), the handler follows the first outgoing edge unconditionally — there is no choice-matching step. The node still requires at least one outgoing edge for routing.

Unknown `question-type` values are silently treated as the default (single select from edges).

### Human answer storage (`store` attribute)

The `store` node attribute on a `wait.human` node writes the human's answer into the pipeline context under the specified key. This enables later nodes to reference the answer via `$KEY` expansion (see below).

```dot
Feedback [ask="What should be improved?", question_type="freeform", store="human.feedback"]
```

Answer values are stored as strings:

| Answer type | Stored value |
|---|---|
| `Text("...")` | The text content |
| `Selected("K")` | The selected key |
| `MultiSelected(["A","B"])` | Comma-separated keys (`"A,B"`) |
| `Yes` | `"yes"` |
| `No` | `"no"` |
| `Timeout` / `Skipped` | _(key not set — resolves to `""` via `Context::get_string`)_ |

When `store` is absent, no additional context key is written (the existing `human.gate.selected` and `human.gate.label` keys are still set for choice-based questions).

### Multi-question interviews (`interview` attribute)

The `interview` attribute specifies a multi-question interview as a YAML or JSON string. In workflows, this is typically populated via `interview-ref` pointing to a YAML code block:

```yaml #review-interview
preamble: |
  Please review the draft.

questions:
  - question: "Ready to publish?"
    type: single-select
    options:
      - label: Approve
      - label: Revise
    store: review.decision

  - question: "What should change?"
    store: review.feedback
```

```dot
Review [interview-ref="#review-interview"]
```

Each question's answer is stored under its `store` key. Routing is driven by the first `single-select` question's answer, matched against outgoing edge labels. `multi-select` questions do not drive routing. After storing answers and setting compatibility context keys, the handler uses the same route-selection behavior as existing single-question human nodes.

When an interview has no `single-select` question, the handler follows the first outgoing edge (same as `question_type="freeform"` for single-question nodes).

### Shell output storage (`store` and `store_as` attributes)

The `store` attribute on a shell node writes the command's trimmed stdout into the pipeline context under the specified key, in addition to the standard `shell.output` and `last_output_full` keys. This enables shell nodes to produce structured data (JSON arrays, objects) that downstream nodes — including dynamic `fan_out` — can consume as typed values.

By default, the handler attempts to parse the output as JSON and falls back to storing it as a plain string. The `store_as` attribute overrides this behavior:

| `store_as` | Behavior |
|---|---|
| _(absent)_ | Try JSON parse; fall back to string |
| `"json"` | JSON parse; fail the node if output is not valid JSON |
| `"string"` | Always store as a string, no JSON parsing |

```dot
Seed [cmd="echo '[\"alpha\",\"beta\",\"gamma\"]'", store="items"]
Seed -> FanOut

FanOut [fan_out="items"]
```

In this example, `Seed` outputs a JSON array. Because `store_as` is absent, the handler parses it as JSON and stores a `Value::Array` under the `items` context key. The `FanOut` node then reads `items` as a real JSON array for dynamic fan-out.

For plain text output (`echo hello`), JSON parsing fails silently and the value is stored as a `Value::String`. To force string storage even for output that happens to be valid JSON (e.g., `echo 42`), use `store_as="string"`.


### Generic context variable expansion (`$KEY`)

In addition to the built-in `$last_output`, `$last_stage`, and `$last_outcome` runtime variables, the codergen handler expands `$KEY` references against the pipeline context at execution time:

```dot
Create [prompt="Create a skill for: $goal\n\nHuman feedback: $human.feedback"]
```

KEY may contain letters, digits, underscores, and dots (e.g., `$human.feedback`, `$step_1.result`). Missing keys resolve to an empty string, consistent with `Context::get_string` behavior.

This enables the end-to-end human feedback pattern: a freeform `wait.human` node stores feedback with `store="human.feedback"`, and a subsequent codergen node interpolates it with `$human.feedback`.

### Dynamic parallel fan-out (`fan_out` attribute)

The attractor spec's parallel fan-out (§4.8) is **static**: the number of concurrent branches is fixed at graph-definition time by the outgoing edges. The `fan_out` node attribute adds a dynamic fan-out mechanism where the branch count is determined at runtime from a variable-length list in the pipeline context.

A node with `fan_out` must have exactly one outgoing edge pointing to the template entry node. The engine spawns one concurrent branch per list item, each executing the same downstream subgraph with per-item context injection:

```dot
FanOutSkills [fan_out="llm.skills"]
FanOutSkills -> ProcessSkill
ProcessSkill -> Merge [shape=tripleoctagon]
```

| Attribute form | Behavior |
|---|---|
| `fan_out="key"` | Resolve context key `key` as a JSON array |
| `fan_out=true` | Derive key from node ID in snake_case (e.g., `FanOutSkills` → `fan_out_skills`) |
| `fan_out=false` | Runtime error (configuration mistake) |

Each branch receives an isolated context clone with these injected keys:

| Context key | Value |
|---|---|
| `fan_out.item` | The current item (any JSON value) |
| `fan_out.index` | Zero-based index into the source list |
| `fan_out.total` | Total number of items |
| `fan_out.key` | The resolved context key |
| `fan_out.item.<prop>` | Top-level properties of object items (enables `$fan_out.item.name` interpolation) |

The `fan_out` attribute implies `shape=component` in the sugar transform (placed after `interview` and before `prompt`/`agent` in precedence). Unlike other sugar keys, `fan_out` is **not** drained — it remains on the node for the `ParallelHandler` to read at runtime.

After all branches complete, results are aggregated into `parallel.results` (with `fan_out_index` and `fan_out_item` fields per entry) and `parallel.outputs` (a JSON array of branch outputs indexed by source position). Existing `join_policy`, `error_policy`, and `max_parallel` semantics apply unchanged.

For empty lists, the handler sets empty `parallel.results` and `parallel.outputs`, then jumps past the fan-in node to its successor. Three validation rules enforce correctness: `dynamic_fan_out` (exactly one outgoing edge), `dynamic_fan_out_missing_fan_in` (warn if no `tripleoctagon` fan-in is reachable), and `nested_dynamic_fan_out` (reject dynamic fan-out inside another dynamic fan-out's template subgraph).

### Outcome status-file compatibility alias

Outcome deserialization accepts both `preferred_next_label` (spec field name) and `preferred_label` as input keys for compatibility with legacy/external status producers.

## Deviations

These are intentional deviations from the spec.

### Input/output terminology (spec: prompt/response)

The spec uses "prompt" and "response" for stage I/O, reflecting its LLM-centric origin. This implementation renames them to "input" and "output" in runtime artifacts, context keys, events, and database columns (e.g., `input.md`/`output.md`, `$last_output`, `StageInput`/`StageOutput`, `workflow_node_outputs`). The DOT authoring attribute `prompt="..."` is kept as a parse-time alias that maps to node input internally.

**Rationale.** Attractor pipelines are not exclusively LLM workflows — they can orchestrate shell commands, human-in-the-loop steps, parallel fan-out, and arbitrary handler types. The terms "input" and "output" are handler-agnostic and align with the terminology used by established workflow engines such as Nextflow (`input:`/`output:` channel declarations), Snakemake (`input:`/`output:` rule directives), and CWL (`inputs`/`outputs`), making the concepts immediately familiar to users coming from scientific and data-engineering workflow backgrounds. Keeping the DOT `prompt` attribute as an authoring shorthand preserves the ergonomic affordance for LLM-focused pipelines without leaking LLM-specific naming into the runtime model.

### Shell handler type (spec: `tool`)

The spec calls the `parallelogram`-shaped handler type `tool` (§4.10). This implementation renames it to `shell` because the handler exclusively runs shell commands via `sh -c`, and the name "tool" creates confusion with LLM tool calls (function calling) used in the codergen handler and elsewhere. The node attribute is `shell_command` (spec: `tool_command`) and the context key is `shell.output` (spec: `tool.output`).

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

## Limitations

The following are known limitations of this implementation of the spec.

### AttrValue serde roundtrip

`Duration` attribute values serialize as strings (e.g., `"15m"`) and deserialize back as `String` variants through JSON, since `serde_json` cannot distinguish Duration from String in the `#[serde(untagged)]` enum. Duration values are fully preserved within a running pipeline; only JSON serialization/deserialization loses the type distinction.

### Graph attribute mirroring loses non-string types in context (§5.1)

During run initialization, `graph.*` keys are mirrored into context as strings via `to_string_value`, so numeric/boolean graph attrs are not preserved as typed JSON values in context.

### Default registry wiring for dependency-backed handlers (§4.2, §4.6, §4.8)

`EngineConfig::new()` does not auto-register `wait.human` and `parallel` because they require runtime dependencies (`Interviewer`, shared registry/emitter). Pipelines using those node types must register handlers explicitly before running.

### Node `timeout` enforcement is not centralized (§2.6, §4.*)

There is no engine-level timeout wrapper applied uniformly to all handlers. `shell` applies `timeout`; other handlers generally do not enforce node-level timeouts.

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

### Interviewer types (§6.1–6.3)

The `Interviewer` trait, question/answer types (`Question`, `QuestionType`, `QuestionOption`, `Answer`, `AnswerValue`), and stateless built-in implementations (`AutoApproveInterviewer`, `CallbackInterviewer`, `QueueInterviewer`, `RecordingInterviewer`) are implemented in the `stencila-interviews` crate (`rust/interviews/`) and re-exported by `attractor` for convenience. This separation allows both `attractor` and `agents` to share the same types without a circular dependency.

### Interviewer feature gaps (§6.2, §6.4)

A terminal stdin/stdout `CliInterviewer` is not implemented, and `Question.metadata` from the spec model is not represented in the current interviewer types.

### Deferred conformance and integration tests (§11.12, §11.13)

Many of the §11.12 parity matrix items are exercised by integration test workflows in `.stencila/workflows/test-*` (see below). The end-to-end smoke test with a real LLM callback handler (§11.13) is deferred pending API key availability.

## Development

### Workflow

The `make check` recipe performs the workflow:

```sh
cargo clippy --fix --allow-dirty --all-targets -p stencila-attractor
cargo fmt -p stencila-attractor
cargo test --all-features -p stencila-attractor
```

### Updating the spec

A vendored copy of the spec is kept in `specs/` for reference. Follow the steps below when upstream changes.

1. Preview upstream changes without modifying the repo:

```sh
make spec-diff
```

No output means the vendored copy is already up to date — no further work is needed.

2. Vendor the latest spec:

```sh
make spec-update
```

3. Review the vendored diff for commit/PR context:

```sh
git --no-pager diff -- specs/attractor-spec.md
```

If the diff is cosmetic only (typo fixes, rewording with no new requirements), no further work is needed.

4. Convert spec changes into implementation work:

- Update requirement rows and status in `tests/spec-traceability.md`.
- Add or update failing tests in the matching `tests/spec_*.rs` file(s) first.
- Implement the minimum code changes in `src/` until tests pass.
- Note any extensions, deviations, and/or limitations in the sections above.

5. Run the crate check recipe:

```sh
make check
```

### Testing

Test files map to spec sections. See `tests/README.md` for details and `tests/spec-traceability.md` for the full mapping.

| File                        | Spec Sections       | Description                                           |
| --------------------------- | ------------------- | ----------------------------------------------------- |
| `tests/spec_1_types.rs`     | §2.4, §5.1–5.3, App D | Core types, context, checkpoint                       |
| `tests/spec_2_parser.rs`    | §2.1–2.13, App A–B | DOT parser and graph model                            |
| `tests/spec_3_engine.rs`    | §3.1–3.8, §4.1–4.4 | Edge selection, engine core, retry, basic handlers    |
| `tests/spec_4_handlers.rs`  | §4.5, §4.10        | Codergen handler, shell handler, `$KEY` expansion |
| `tests/spec_4_parallel.rs`  | §4.8–4.9           | Parallel fan-out, fan-in, join/error policies         |
| `tests/spec_4_dynamic_fan_out.rs` | §4.8 (ext)   | Dynamic fan-out: runtime list resolution, per-item context, validation |
| `tests/spec_5_state.rs`     | §5.3–5.5           | Artifacts, fidelity, thread IDs, checkpoint resume    |
| `tests/spec_6_human.rs`     | §4.6, §6           | Interviewers, WaitForHuman handler, accelerator keys, `question_type`, `store` |
| `tests/spec_7_validation.rs`| §7                  | Validation and lint rules                             |
| `tests/spec_8_stylesheet.rs`| §8                  | Model stylesheet parsing and application              |
| `tests/spec_9_transforms.rs`| §9.1–9.4           | Transform trait and built-in transforms               |
| `tests/spec_9_events.rs`    | §9.6                | Event emitters: NoOp, Collecting, Observer, Broadcast |
| `tests/spec_10_conditions.rs`| §10                | Condition expression language                         |

Use the crate check recipe:

```sh
make check
```

### Integration test workflows

End-to-end integration tests live in `.stencila/workflows/test-*/WORKFLOW.md` at the repository root. Each workflow exercises a specific combination of pipeline features with a real LLM backend:

| Workflow                      | Concepts Exercised |
| ----------------------------- | --------------------------------------------------- |
| `test-no-op`                  | Minimal `Start → End` graph |
| `test-count-to-three`         | Linear chain, `$last_output`, `$goal` expansion, `Fail` node |
| `test-count-to-goal`          | Looping (self-edge), label-based branching via `workflow_set_route` or XML tag fallback |
| `test-human-gates`            | `wait.human` via `ask=` sugar, binary/three-way/single-choice gates, `question_type`, `store` |
| `test-fan-out-fan-in`         | Parallel fan-out via `FanOut` ID, fan-in convergence |
| `test-conditional-branching`  | `Check*` ID → diamond shape, `outcome=` conditions, edge retry loop |
| `test-overrides`              | `overrides` frontmatter, `*` / `.class` / `#id` selectors |
| `test-goal-gates`             | `goal_gate=true`, `retryTarget` loopback |
| `test-max-retries`            | `max_retries=N` node attribute |
| `test-shell-nodes`            | `cmd=` / `shell=` sugar → `shell` handler, no LLM calls |
| `test-edge-weights`           | Edge `weight=` routing priority |
| `test-subgraph-defaults`      | `subgraph` blocks, scoped `node [...]` defaults |
| `test-agent-reference`        | `agent=` node attribute, agent resolution |
| `test-kitchen-sink`           | All patterns combined (shell, parallel, conditional, human, retry, stylesheet) |
| `test-context-conditions`     | Multi-way `context.*` edge conditions, fallback edge |

Run a specific workflow with:

```sh
cargo run -- workflows run <name>
```

### Documentation

User-facing pipeline features (new node types, attributes, shapes, workflow patterns) are documented in `site/docs/workflows/`. If you make changes that affect how users write or run pipelines, update the relevant docs there as well, in particular `site/docs/workflows/pipelines.md`.
