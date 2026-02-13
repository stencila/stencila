# Attractor Specification

A DOT-based pipeline runner that uses directed graphs (defined in Graphviz DOT syntax) to orchestrate multi-stage AI workflows. Each node in the graph is an AI task (LLM call, human review, conditional branch, parallel fan-out, etc.) and edges define the flow between them.

---

## Table of Contents

1. [Overview and Goals](#1-overview-and-goals)
2. [DOT DSL Schema](#2-dot-dsl-schema)
3. [Pipeline Execution Engine](#3-pipeline-execution-engine)
4. [Node Handlers](#4-node-handlers)
5. [State and Context](#5-state-and-context)
6. [Human-in-the-Loop (Interviewer Pattern)](#6-human-in-the-loop-interviewer-pattern)
7. [Validation and Linting](#7-validation-and-linting)
8. [Model Stylesheet](#8-model-stylesheet)
9. [Transforms and Extensibility](#9-transforms-and-extensibility)
10. [Condition Expression Language](#10-condition-expression-language)
11. [Definition of Done](#11-definition-of-done)

---

## 1. Overview and Goals

### 1.1 Problem Statement

AI-powered software workflows -- code generation, code review, testing, deployment planning -- often require multiple LLM calls chained together with conditional logic, human approvals, and parallel execution. Without a structured orchestration layer, developers either write fragile imperative scripts or build ad-hoc state machines that are difficult to visualize, version, or debug.

Attractor solves this by letting pipeline authors define multi-stage AI workflows as directed graphs using Graphviz DOT syntax. The graph is the workflow: nodes are tasks, edges are transitions, and attributes configure behavior. The result is a declarative, visual, version-controllable pipeline definition that an execution engine can traverse deterministically.

### 1.2 Why DOT Syntax

DOT is chosen as the pipeline definition format for several reasons:

- **DOT is inherently a graph description language.** Workflow pipelines are directed graphs. Using DOT means the structure (nodes and edges) maps directly to the language's primary construct, rather than being encoded in a data format like YAML or JSON that has no native concept of graphs.
- **Existing tooling.** DOT files can be rendered to SVG/PNG with standard Graphviz tooling, giving pipeline authors immediate visual feedback. Editors, linters, and parsers already exist.
- **Declarative and human-readable.** A `.dot` file is a complete, self-contained workflow definition that can be version-controlled, diffed, and reviewed in pull requests.
- **Constrained extensibility.** By restricting to a well-defined DOT subset (directed graphs only, typed attributes, no HTML labels), the DSL remains predictable while being extensible through custom attributes.

For reference on DOT syntax, see the Graphviz DOT language specification: https://graphviz.org/doc/info/lang.html

### 1.3 Design Principles

**Declarative pipelines.** The `.dot` file declares what the workflow looks like and what each stage should do. The execution engine decides how and when to run each stage. Pipeline authors do not write control flow; they declare graph structure.

**Pluggable handlers.** Each node type (LLM call, human gate, parallel fan-out) is backed by a handler that implements a common interface. New node types are added by registering new handlers. The execution engine does not know about handler internals.

**Checkpoint and resume.** After each node completes, the execution engine saves a serializable checkpoint. If the process crashes, execution resumes from the last checkpoint.

**Human-in-the-loop.** The pipeline can pause at designated nodes, present choices to a human operator, and route based on the human's decision. This supports approval gates, code review, and manual override -- critical for AI workflows where automated judgment may not be sufficient.

**Edge-based routing.** Transitions between nodes are controlled by conditions, labels, and weights on edges, with runtime condition evaluation.

### 1.4 Layering and LLM Backends

Attractor defines the orchestration layer: graph definition, traversal, state management, and extensibility. It does NOT require any specific LLM integration. The codergen handler (Section 4.5) needs a way to call an LLM and get a response -- how you provide that is up to you.

The codergen handler takes a backend that conforms to the `CodergenBackend` interface (Section 4.5). What that backend does internally is entirely up to the implementor -- use the companion [Coding Agent Loop](./coding-agent-loop-spec.md) and [Unified LLM Client](./unified-llm-spec.md) specs, spawn CLI agents (Claude Code, Codex, Gemini CLI) in subprocesses, run agents in tmux panes with a manager attaching to them, call an LLM API directly, or anything else. The pipeline definition (the DOT file) does not change regardless of backend choice.

Attractor pipelines are driven by an event stream (Section 9.6). TUI, web, and IDE frontends consume events and submit human-in-the-loop answers. The pipeline engine is headless; the presentation layer is separate.

---

## 2. DOT DSL Schema

### 2.1 Supported Subset

Attractor accepts a strict subset of the Graphviz DOT language. The restrictions exist for predictability: one graph per file, directed edges only, no HTML labels, and typed attributes with defaults.

### 2.2 BNF-Style Grammar

```
Graph           ::= 'digraph' Identifier '{' Statement* '}'

Statement       ::= GraphAttrStmt
                   | NodeDefaults
                   | EdgeDefaults
                   | SubgraphStmt
                   | NodeStmt
                   | EdgeStmt
                   | GraphAttrDecl

GraphAttrStmt   ::= 'graph' AttrBlock ';'?
NodeDefaults    ::= 'node' AttrBlock ';'?
EdgeDefaults    ::= 'edge' AttrBlock ';'?
GraphAttrDecl   ::= Identifier '=' Value ';'?

SubgraphStmt    ::= 'subgraph' Identifier? '{' Statement* '}'

NodeStmt        ::= Identifier AttrBlock? ';'?
EdgeStmt        ::= Identifier ( '->' Identifier )+ AttrBlock? ';'?

AttrBlock       ::= '[' Attr ( ',' Attr )* ']'
Attr            ::= Key '=' Value

Key             ::= Identifier | QualifiedId
QualifiedId     ::= Identifier ( '.' Identifier )+

Value           ::= String | Integer | Float | Boolean | Duration
Identifier      ::= [A-Za-z_][A-Za-z0-9_]*
String          ::= '"' ( '\\"' | '\\n' | '\\t' | '\\\\' | [^"\\] )* '"'
Integer         ::= '-'? [0-9]+
Float           ::= '-'? [0-9]* '.' [0-9]+
Boolean         ::= 'true' | 'false'
Duration        ::= Integer ( 'ms' | 's' | 'm' | 'h' | 'd' )

Direction       ::= 'TB' | 'LR' | 'BT' | 'RL'
```

### 2.3 Key Constraints

- **One digraph per file.** Multiple graphs, undirected graphs, and `strict` modifiers are rejected.
- **Bare identifiers for node IDs.** Node IDs must match `[A-Za-z_][A-Za-z0-9_]*`. Human-readable names go in the `label` attribute.
- **Commas required between attributes.** Inside attribute blocks, commas separate key-value pairs for unambiguous parsing.
- **Directed edges only.** `->` is the only edge operator. `--` (undirected) is rejected.
- **Comments supported.** Both `// line` and `/* block */` comments are stripped before parsing.
- **Semicolons optional.** Statement-terminating semicolons are accepted but not required.

### 2.4 Value Types

| Type     | Syntax                          | Examples                             |
|----------|---------------------------------|--------------------------------------|
| String   | Double-quoted with escapes      | `"Hello world"`, `"line1\nline2"`    |
| Integer  | Optional sign, digits           | `42`, `-1`, `0`                      |
| Float    | Decimal number                  | `0.5`, `-3.14`                       |
| Boolean  | Literal keywords                | `true`, `false`                      |
| Duration | Integer + unit suffix           | `900s`, `15m`, `2h`, `250ms`, `1d`   |

### 2.5 Graph-Level Attributes

Graph attributes are declared in a `graph [ ... ]` block or as top-level `key = value` declarations. They configure the entire workflow.

| Key                       | Type     | Default   | Description |
|---------------------------|----------|-----------|-------------|
| `goal`                    | String   | `""`      | Human-readable goal for the pipeline. Exposed as `$goal` in prompt templates and mirrored into the run context as `graph.goal`. |
| `label`                   | String   | `""`      | Display name for the graph (used in visualization). |
| `model_stylesheet`        | String   | `""`      | CSS-like stylesheet for per-node LLM model/provider defaults. See Section 8. |
| `default_max_retry`       | Integer  | `50`      | Global retry ceiling for nodes that omit `max_retries`. |
| `retry_target`            | String   | `""`      | Node ID to jump to if exit is reached with unsatisfied goal gates. |
| `fallback_retry_target`   | String   | `""`      | Secondary jump target if `retry_target` is missing or invalid. |
| `default_fidelity`        | String   | `""`      | Default context fidelity mode (see Section 5.4). |

### 2.6 Node Attributes

| Key                 | Type     | Default         | Description |
|---------------------|----------|-----------------|-------------|
| `label`             | String   | node ID         | Display name shown in UI, prompts, and telemetry. |
| `shape`             | String   | `"box"`         | Graphviz shape. Determines the default handler type (see mapping table below). |
| `type`              | String   | `""`            | Explicit handler type override. Takes precedence over shape-based resolution. |
| `prompt`            | String   | `""`            | Primary instruction for the stage. Supports `$goal` variable expansion. Falls back to `label` if empty for LLM stages. |
| `max_retries`       | Integer  | `0`             | Number of additional attempts beyond the initial execution. `max_retries=3` means up to 4 total executions. |
| `goal_gate`         | Boolean  | `false`         | If `true`, this node must reach SUCCESS before the pipeline can exit. |
| `retry_target`      | String   | `""`            | Node ID to jump to if this node fails and retries are exhausted. |
| `fallback_retry_target` | String | `""`          | Secondary retry target. |
| `fidelity`          | String   | inherited       | Context fidelity mode for this node's LLM session. See Section 5.4. |
| `thread_id`         | String   | derived         | Explicit thread identifier for LLM session reuse under `full` fidelity. |
| `class`             | String   | `""`            | Comma-separated class names for model stylesheet targeting. |
| `timeout`           | Duration | unset           | Maximum execution time for this node. |
| `llm_model`         | String   | inherited       | LLM model identifier. Overridable by stylesheet. |
| `llm_provider`      | String   | auto-detected   | LLM provider key. Auto-detected from model if unset. |
| `reasoning_effort`  | String   | `"high"`        | LLM reasoning effort: `low`, `medium`, `high`. |
| `auto_status`       | Boolean  | `false`         | If `true` and the handler writes no status, the engine auto-generates a SUCCESS outcome. |
| `allow_partial`     | Boolean  | `false`         | Accept PARTIAL_SUCCESS when retries are exhausted instead of failing. |

### 2.7 Edge Attributes

| Key          | Type     | Default | Description |
|--------------|----------|---------|-------------|
| `label`      | String   | `""`    | Human-facing caption and routing key. Used for preferred-label matching in edge selection. |
| `condition`  | String   | `""`    | Boolean guard expression evaluated against the current context and outcome. See Section 10. |
| `weight`     | Integer  | `0`     | Numeric priority for edge selection. Higher weight wins among equally eligible edges. |
| `fidelity`   | String   | unset   | Override fidelity mode for the target node. Highest precedence in fidelity resolution. |
| `thread_id`  | String   | unset   | Override thread ID for session reuse at the target node. |
| `loop_restart` | Boolean | `false` | When `true`, terminates the current run and re-launches with a fresh log directory. |

### 2.8 Shape-to-Handler-Type Mapping

The `shape` attribute on a node determines which handler executes it, unless overridden by an explicit `type` attribute. This table defines the canonical mapping:

| Shape             | Handler Type          | Description |
|-------------------|-----------------------|-------------|
| `Mdiamond`        | `start`               | Pipeline entry point. No-op handler. Every graph must have exactly one. |
| `Msquare`         | `exit`                | Pipeline exit point. No-op handler. Every graph must have exactly one. |
| `box`             | `codergen`            | LLM task (code generation, analysis, planning). The default for all nodes without an explicit shape. |
| `hexagon`         | `wait.human`          | Human-in-the-loop gate. Blocks until a human selects an option. |
| `diamond`         | `conditional`         | Conditional routing point. Routes based on edge conditions against current context. |
| `component`       | `parallel`            | Parallel fan-out. Executes multiple branches concurrently. |
| `tripleoctagon`   | `parallel.fan_in`     | Parallel fan-in. Waits for all branches and consolidates results. |
| `parallelogram`   | `tool`                | External tool execution (shell command, API call). |
| `house`           | `stack.manager_loop`  | Supervisor loop. Orchestrates observe/steer/wait cycles over a child pipeline. |

### 2.9 Chained Edges

Chained edge declarations are syntactic sugar. The statement:

```
A -> B -> C [label="next"]
```

expands to two edges:

```
A -> B [label="next"]
B -> C [label="next"]
```

Edge attributes in a chained declaration apply to all edges in the chain.

### 2.10 Subgraphs

Subgraphs serve two purposes: **scoping defaults** and **deriving classes** for the model stylesheet.

**Scoping defaults:** Attributes declared in a subgraph's `node [ ... ]` block apply to nodes within that subgraph unless the node explicitly overrides them.

```
subgraph cluster_loop {
    label = "Loop A"
    node [thread_id="loop-a", timeout="900s"]

    Plan      [label="Plan next step"]
    Implement [label="Implement", timeout="1800s"]
}
```

Here `Plan` inherits `thread_id="loop-a"` and `timeout="900s"`, while `Implement` inherits `thread_id` but overrides `timeout`.

**Class derivation:** Subgraph labels can produce CSS-like classes for model stylesheet matching. Nodes inside a subgraph receive the derived class. The class name is derived by lowercasing the label, replacing spaces with hyphens, and stripping non-alphanumeric characters (except hyphens). For example, `label="Loop A"` yields class `loop-a`.

### 2.11 Node and Edge Default Blocks

Default blocks set baseline attributes for all subsequent nodes or edges within their scope:

```
node [shape=box, timeout="900s"]
edge [weight=0]
```

Explicit attributes on individual nodes or edges override these defaults.

### 2.12 Class Attribute

The `class` attribute assigns one or more CSS-like class names to a node for model stylesheet targeting:

```
review_code [shape=box, class="code,critical", prompt="Review the code"]
```

Classes are comma-separated. They can be referenced in the model stylesheet with dot-prefix selectors (`.code`, `.critical`).

### 2.13 Minimal Examples

**Simple linear workflow:**

```
digraph Simple {
    graph [goal="Run tests and report"]
    rankdir=LR

    start [shape=Mdiamond, label="Start"]
    exit  [shape=Msquare, label="Exit"]

    run_tests [label="Run Tests", prompt="Run the test suite and report results"]
    report    [label="Report", prompt="Summarize the test results"]

    start -> run_tests -> report -> exit
}
```

**Branching workflow with conditions:**

```
digraph Branch {
    graph [goal="Implement and validate a feature"]
    rankdir=LR
    node [shape=box, timeout="900s"]

    start     [shape=Mdiamond, label="Start"]
    exit      [shape=Msquare, label="Exit"]
    plan      [label="Plan", prompt="Plan the implementation"]
    implement [label="Implement", prompt="Implement the plan"]
    validate  [label="Validate", prompt="Run tests"]
    gate      [shape=diamond, label="Tests passing?"]

    start -> plan -> implement -> validate -> gate
    gate -> exit      [label="Yes", condition="outcome=success"]
    gate -> implement [label="No", condition="outcome!=success"]
}
```

**Human gate:**

```
digraph Review {
    rankdir=LR

    start [shape=Mdiamond, label="Start"]
    exit  [shape=Msquare, label="Exit"]

    review_gate [
        shape=hexagon,
        label="Review Changes",
        type="wait.human"
    ]

    start -> review_gate
    review_gate -> ship_it [label="[A] Approve"]
    review_gate -> fixes   [label="[F] Fix"]
    ship_it -> exit
    fixes -> review_gate
}
```

---

## 3. Pipeline Execution Engine

### 3.1 Run Lifecycle

The execution lifecycle proceeds through five phases:

```
PARSE -> VALIDATE -> INITIALIZE -> EXECUTE -> FINALIZE
```

1. **Parse:** Read the `.dot` source and produce an in-memory Graph model (nodes, edges, attributes).
2. **Validate:** Run lint rules (Section 7). Reject invalid graphs. Warn on suspicious patterns.
3. **Initialize:** Create the run directory, initial context, and checkpoint. Mirror graph attributes into the context. Apply transforms (stylesheet, variable expansion).
4. **Execute:** Traverse the graph from the start node, executing handlers and selecting edges.
5. **Finalize:** Write the final checkpoint, emit completion events, and clean up resources (close sessions, release files).

### 3.2 Core Execution Loop

The following pseudocode defines the execution engine's traversal algorithm. This is the heart of the system.

```
FUNCTION run(graph, config):
    context = new Context()
    mirror_graph_attributes(graph, context)
    checkpoint = new Checkpoint()
    completed_nodes = []
    node_outcomes = {}

    current_node = find_start_node(graph)
        -- Resolves by: (1) shape=Mdiamond, (2) id="start" or "Start"
        -- Raises error if not found

    WHILE true:
        node = graph.nodes[current_node.id]

        -- Step 1: Check for terminal node
        IF is_terminal(node):
            gate_ok, failed_gate = check_goal_gates(graph, node_outcomes)
            IF NOT gate_ok AND failed_gate exists:
                retry_target = get_retry_target(failed_gate, graph)
                IF retry_target exists:
                    current_node = graph.nodes[retry_target]
                    CONTINUE
                ELSE:
                    RAISE "Goal gate unsatisfied and no retry target"
            BREAK  -- Exit the loop; pipeline complete

        -- Step 2: Execute node handler with retry policy
        retry_policy = build_retry_policy(node, graph)
        outcome = execute_with_retry(node, context, graph, retry_policy)

        -- Step 3: Record completion
        completed_nodes.append(node.id)
        node_outcomes[node.id] = outcome

        -- Step 4: Apply context updates from outcome
        FOR EACH (key, value) IN outcome.context_updates:
            context.set(key, value)
        context.set("outcome", outcome.status)
        IF outcome.preferred_label is not empty:
            context.set("preferred_label", outcome.preferred_label)

        -- Step 5: Save checkpoint
        checkpoint = create_checkpoint(context, current_node.id, completed_nodes)
        save_checkpoint(checkpoint, logs_root)

        -- Step 6: Select next edge
        next_edge = select_edge(node, outcome, context, graph)
        IF next_edge is NONE:
            IF outcome.status == FAIL:
                RAISE "Stage failed with no outgoing fail edge"
            BREAK

        -- Step 7: Handle loop_restart
        IF next_edge has loop_restart=true:
            restart_run(graph, config, start_at=next_edge.target)
            RETURN

        -- Step 8: Advance to next node
        current_node = graph.nodes[next_edge.to_node]

    RETURN last_outcome
```

### 3.3 Edge Selection Algorithm

After a node completes, the engine selects the next edge from the node's outgoing edges. The selection is deterministic and follows a five-step priority order:

**Step 1: Condition-matching edges.** Evaluate each edge's `condition` expression (see Section 10) against the current context and outcome. Edges whose condition evaluates to `true` are eligible. Edges with no condition are not considered in this step; they proceed to later steps.

**Step 2: Preferred label match.** If the node's outcome includes a `preferred_label`, find the first eligible edge (condition-passing or unconditional) whose `label` matches after normalization. Label normalization: lowercase, trim whitespace, strip accelerator prefixes (patterns like `[Y] `, `Y) `, `Y - `).

**Step 3: Suggested next IDs.** If no label match and the outcome includes `suggested_next_ids`, find the first eligible edge whose target node ID appears in the list.

**Step 4: Highest weight.** Among remaining eligible unconditional edges, choose the one with the highest `weight` attribute (default 0).

**Step 5: Lexical tiebreak.** If weights are equal, choose the edge whose target node ID comes first lexicographically.

```
FUNCTION select_edge(node, outcome, context, graph):
    edges = graph.outgoing_edges(node.id)
    IF edges is empty:
        RETURN NONE

    -- Step 1: Condition matching
    condition_matched = []
    FOR EACH edge IN edges:
        IF edge.condition is not empty:
            IF evaluate_condition(edge.condition, outcome, context) == true:
                condition_matched.append(edge)
    IF condition_matched is not empty:
        RETURN best_by_weight_then_lexical(condition_matched)

    -- Step 2: Preferred label
    IF outcome.preferred_label is not empty:
        FOR EACH edge IN edges:
            IF normalize_label(edge.label) == normalize_label(outcome.preferred_label):
                RETURN edge

    -- Step 3: Suggested next IDs
    IF outcome.suggested_next_ids is not empty:
        FOR EACH suggested_id IN outcome.suggested_next_ids:
            FOR EACH edge IN edges:
                IF edge.to_node == suggested_id:
                    RETURN edge

    -- Step 4 & 5: Weight with lexical tiebreak (unconditional edges only)
    unconditional = [e FOR e IN edges WHERE e.condition is empty]
    IF unconditional is not empty:
        RETURN best_by_weight_then_lexical(unconditional)

    -- Fallback: any edge
    RETURN best_by_weight_then_lexical(edges)


FUNCTION best_by_weight_then_lexical(edges):
    SORT edges BY (weight DESCENDING, to_node ASCENDING)
    RETURN edges[0]
```

### 3.4 Goal Gate Enforcement

Nodes with `goal_gate=true` represent critical stages that must succeed before the pipeline can exit. When the traversal reaches a terminal node (shape=Msquare):

1. Check all visited nodes that have `goal_gate=true`.
2. If any goal gate node has a non-success outcome (not SUCCESS or PARTIAL_SUCCESS), the pipeline cannot exit.
3. Instead, jump to the `retry_target` of the unsatisfied goal gate node. If that is not set, try `fallback_retry_target`. If that is also not set, try the graph-level `retry_target` and `fallback_retry_target`.
4. If no retry target exists at any level, the pipeline fails with an error.

```
FUNCTION check_goal_gates(graph, node_outcomes):
    FOR EACH (node_id, outcome) IN node_outcomes:
        node = graph.nodes[node_id]
        IF node.goal_gate == true:
            IF outcome.status NOT IN {SUCCESS, PARTIAL_SUCCESS}:
                RETURN (false, node)
    RETURN (true, NONE)
```

### 3.5 Retry Logic

Each node has a retry policy determined by:

1. Node attribute `max_retries` (if set) -- number of additional attempts beyond the initial execution
2. Graph attribute `default_max_retry` (fallback)
3. Built-in default: 0 (no retries)

The `max_retries` attribute specifies additional attempts. So `max_retries=3` means a total of 4 executions (1 initial + 3 retries). Internally this maps to `max_attempts = max_retries + 1`.

```
FUNCTION execute_with_retry(node, context, graph, retry_policy):
    FOR attempt FROM 1 TO retry_policy.max_attempts:
        TRY:
            outcome = handler.execute(node, context, graph, logs_root)
        CATCH exception:
            IF retry_policy.should_retry(exception) AND attempt < retry_policy.max_attempts:
                delay = retry_policy.backoff.delay_for_attempt(attempt)
                sleep(delay)
                CONTINUE
            ELSE:
                RETURN Outcome(status=FAIL, failure_reason=str(exception))

        IF outcome.status IN {SUCCESS, PARTIAL_SUCCESS}:
            reset_retry_counter(node.id)
            RETURN outcome

        IF outcome.status == RETRY:
            IF attempt < retry_policy.max_attempts:
                increment_retry_counter(node.id)
                delay = retry_policy.backoff.delay_for_attempt(attempt)
                sleep(delay)
                CONTINUE
            ELSE:
                IF node.allow_partial == true:
                    RETURN Outcome(status=PARTIAL_SUCCESS, notes="retries exhausted, partial accepted")
                RETURN Outcome(status=FAIL, failure_reason="max retries exceeded")

        IF outcome.status == FAIL:
            RETURN outcome

    RETURN Outcome(status=FAIL, failure_reason="max retries exceeded")
```

### 3.6 Retry Policy

```
RetryPolicy:
    max_attempts    : Integer         -- minimum 1 (1 means no retries)
    backoff         : BackoffConfig   -- delay calculation between retries
    should_retry    : Function(Error) -> Boolean  -- predicate for retryable errors

BackoffConfig:
    initial_delay_ms  : Integer   -- first retry delay in milliseconds (default: 200)
    backoff_factor    : Float     -- multiplier for subsequent delays (default: 2.0)
    max_delay_ms      : Integer   -- cap on delay in milliseconds (default: 60000)
    jitter            : Boolean   -- add random jitter to prevent thundering herd (default: true)
```

**Delay calculation:**

```
FUNCTION delay_for_attempt(attempt, config):
    -- attempt is 1-indexed (first retry is attempt=1)
    delay = config.initial_delay_ms * (config.backoff_factor ^ (attempt - 1))
    delay = MIN(delay, config.max_delay_ms)
    IF config.jitter:
        delay = delay * random_uniform(0.5, 1.5)
    RETURN delay
```

**Preset policies:**

| Name         | Max Attempts | Initial Delay | Factor | Description |
|--------------|-------------|---------------|--------|-------------|
| `none`       | 1           | --            | --     | No retries. Fail immediately on error. |
| `standard`   | 5           | 200ms         | 2.0    | General-purpose. Delays: 200, 400, 800, 1600, 3200ms. |
| `aggressive`  | 5           | 500ms         | 2.0    | For unreliable operations. Delays: 500, 1000, 2000, 4000, 8000ms. |
| `linear`     | 3           | 500ms         | 1.0    | Fixed delay between attempts. Delays: 500, 500, 500ms. |
| `patient`    | 3           | 2000ms        | 3.0    | Long-running operations. Delays: 2000, 6000, 18000ms. |

**Default should_retry predicate:** Returns `true` for network errors, rate limit errors (HTTP 429), server errors (HTTP 5xx), and provider-reported transient failures. Returns `false` for authentication errors (HTTP 401, 403), bad request errors (HTTP 400), validation errors, and configuration errors.

### 3.7 Failure Routing

When a stage returns FAIL (or retries are exhausted), the engine attempts failure routing in this order:

1. **Fail edge:** An outgoing edge with `condition="outcome=fail"`. If found, follow it.
2. **Retry target:** Node attribute `retry_target`. Jump to that node.
3. **Fallback retry target:** Node attribute `fallback_retry_target`. Jump to that node.
4. **Pipeline termination:** No failure route found. The pipeline fails with the stage's failure reason.

### 3.8 Concurrency Model

The graph traversal is single-threaded. Only one node executes at a time in the top-level graph. This simplifies reasoning about context state and avoids race conditions.

Parallelism exists within specific node handlers (`parallel`, `parallel.fan_in`) that manage concurrent execution internally. Each parallel branch receives an isolated clone of the context. Branch results are collected but individual branch context changes are not merged back into the parent -- only the handler's outcome and its `context_updates` are applied.

---

## 4. Node Handlers

### 4.1 Handler Interface

Every node handler implements a common interface. The execution engine dispatches to the appropriate handler based on the node's `type` attribute (or shape-based resolution if `type` is empty).

```
INTERFACE Handler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome

    -- Parameters:
    --   node      : The parsed Node with all its attributes
    --   context   : The shared key-value Context for the pipeline run (read/write)
    --   graph     : The full parsed Graph (for reading outgoing edges, etc.)
    --   logs_root : Filesystem path for this run's log/artifact directory

    -- Returns:
    --   Outcome   : The result of execution (see Section 5.2)
```

### 4.2 Handler Registry

The handler registry maps type strings to handler instances. Resolution follows this order:

1. **Explicit `type` attribute** on the node (e.g., `type="wait.human"`)
2. **Shape-based resolution** using the shape-to-handler-type mapping table (Section 2.8)
3. **Default handler** (the codergen/LLM handler)

```
HandlerRegistry:
    handlers        : Map<String, Handler>   -- type string -> handler instance
    default_handler : Handler                -- fallback handler (typically codergen)

    FUNCTION register(type_string, handler):
        handlers[type_string] = handler
        -- Registering for an already-registered type replaces the previous handler

    FUNCTION resolve(node) -> Handler:
        -- 1. Explicit type attribute
        IF node.type is not empty AND node.type IN handlers:
            RETURN handlers[node.type]

        -- 2. Shape-based resolution
        handler_type = SHAPE_TO_TYPE[node.shape]
        IF handler_type IN handlers:
            RETURN handlers[handler_type]

        -- 3. Default
        RETURN default_handler
```

### 4.3 Start Handler

A no-op handler for the pipeline entry point. Returns SUCCESS immediately without performing any work.

```
StartHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        RETURN Outcome(status=SUCCESS)
```

Every graph must have exactly one start node (shape=Mdiamond). The lint rules enforce this.

### 4.4 Exit Handler

A no-op handler for the pipeline exit point. Returns SUCCESS immediately. Goal gate enforcement is handled by the execution engine (Section 3.4), not by this handler.

```
ExitHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        RETURN Outcome(status=SUCCESS)
```

Every graph must have exactly one exit node (shape=Msquare).

### 4.5 Codergen Handler (LLM Task)

The codergen handler is the default for all nodes that invoke an LLM. It reads the node's prompt, expands template variables, calls the LLM backend (see Section 1.4 for backend options), writes the prompt and response to the logs directory, and returns the outcome.

```
CodergenHandler:
    backend : CodergenBackend | None
        -- The LLM execution backend. Any implementation of the
        -- CodergenBackend interface (Section 4.5). None = simulation mode.

    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        -- 1. Build prompt
        prompt = node.prompt
        IF prompt is empty:
            prompt = node.label
        prompt = expand_variables(prompt, graph, context)

        -- 2. Write prompt to logs
        stage_dir = logs_root + "/" + node.id + "/"
        create_directory(stage_dir)
        write_file(stage_dir + "prompt.md", prompt)

        -- 3. Call LLM backend
        IF backend is not NONE:
            TRY:
                result = backend.run(node, prompt, context)
                IF result is an Outcome:
                    write_status(stage_dir, result)
                    RETURN result
                response_text = string(result)
            CATCH exception:
                RETURN Outcome(status=FAIL, failure_reason=str(exception))
        ELSE:
            response_text = "[Simulated] Response for stage: " + node.id

        -- 4. Write response to logs
        write_file(stage_dir + "response.md", response_text)

        -- 5. Write status and return outcome
        outcome = Outcome(
            status=SUCCESS,
            notes="Stage completed: " + node.id,
            context_updates={
                "last_stage": node.id,
                "last_response": truncate(response_text, 200)
            }
        )
        write_status(stage_dir, outcome)
        RETURN outcome
```

**Variable expansion:** The only built-in template variable is `$goal`, which resolves to the graph-level `goal` attribute. Variable expansion is simple string replacement, not a templating engine.

**Status file:** The handler writes `status.json` in the stage directory with the Outcome fields serialized as JSON. This file serves as an audit trail and enables the status-file contract: external tools or agents can write `status.json` to communicate outcomes back to the engine.

#### CodergenBackend Interface

```
INTERFACE CodergenBackend:
    FUNCTION run(node: Node, prompt: String, context: Context) -> String | Outcome
```

How you implement this interface is up to you. The pipeline engine only cares that it gets a String or Outcome back.

### 4.6 Wait For Human Handler

Blocks pipeline execution until a human selects an option derived from the node's outgoing edges. This implements the human-in-the-loop pattern (see Section 6 for the full Interviewer protocol).

```
WaitForHumanHandler:
    interviewer : Interviewer  -- the human interaction frontend

    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        -- 1. Derive choices from outgoing edges
        edges = graph.outgoing_edges(node.id)
        choices = []
        FOR EACH edge IN edges:
            label = edge.label OR edge.to_node
            key = parse_accelerator_key(label)
            choices.append(Choice(key=key, label=label, to=edge.to_node))

        IF choices is empty:
            RETURN Outcome(status=FAIL, failure_reason="No outgoing edges for human gate")

        -- 2. Build question from choices
        options = [Option(key=c.key, label=c.label) FOR c IN choices]
        question = Question(
            text=node.label OR "Select an option:",
            type=MULTIPLE_CHOICE,
            options=options,
            stage=node.id
        )

        -- 3. Present to interviewer and wait for answer
        answer = interviewer.ask(question)

        -- 4. Handle timeout/skip
        IF answer is TIMEOUT:
            default_choice = node.attrs["human.default_choice"]
            IF default_choice exists:
                -- Use default
            ELSE:
                RETURN Outcome(status=RETRY, failure_reason="human gate timeout, no default")

        IF answer is SKIPPED:
            RETURN Outcome(status=FAIL, failure_reason="human skipped interaction")

        -- 5. Find matching choice
        selected = find_choice_matching(answer, choices)
        IF selected is NONE:
            selected = choices[0]  -- fallback to first

        -- 6. Record in context and return
        RETURN Outcome(
            status=SUCCESS,
            suggested_next_ids=[selected.to],
            context_updates={
                "human.gate.selected": selected.key,
                "human.gate.label": selected.label
            }
        )
```

**Accelerator key parsing** extracts shortcut keys from edge labels using these patterns:

| Pattern           | Example           | Extracted Key |
|-------------------|-------------------|---------------|
| `[K] Label`       | `[Y] Yes, deploy` | `Y`           |
| `K) Label`        | `Y) Yes, deploy`  | `Y`           |
| `K - Label`       | `Y - Yes, deploy` | `Y`           |
| First character   | `Yes, deploy`     | `Y`           |

### 4.7 Conditional Handler

For diamond-shaped nodes that act as conditional routing points. The handler itself is a no-op that returns SUCCESS; the actual routing is handled by the execution engine's edge selection algorithm (Section 3.3), which evaluates conditions on outgoing edges.

```
ConditionalHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        RETURN Outcome(
            status=SUCCESS,
            notes="Conditional node evaluated: " + node.id
        )
```

This design keeps routing logic in the engine (where it can be deterministic and inspectable) rather than in the handler.

### 4.8 Parallel Handler

Fans out execution to multiple branches concurrently. Each parallel branch receives an isolated clone of the parent context and runs independently. The handler waits for all branches to complete (or applies a configurable join policy) before returning.

```
ParallelHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        -- 1. Identify fan-out edges (all outgoing edges from this node)
        branches = graph.outgoing_edges(node.id)

        -- 2. Determine join policy from node attributes
        join_policy = node.attrs.get("join_policy", "wait_all")
        error_policy = node.attrs.get("error_policy", "continue")
        max_parallel = integer(node.attrs.get("max_parallel", "4"))

        -- 3. Execute branches concurrently with bounded parallelism
        results = []
        FOR EACH branch IN branches (up to max_parallel at a time):
            branch_context = context.clone()
            branch_outcome = execute_subgraph(branch.to_node, branch_context, graph, logs_root)
            results.append(branch_outcome)

        -- 4. Evaluate join policy
        success_count = count(r FOR r IN results WHERE r.status == SUCCESS)
        fail_count = count(r FOR r IN results WHERE r.status == FAIL)

        IF join_policy == "wait_all":
            IF fail_count == 0:
                RETURN Outcome(status=SUCCESS)
            ELSE:
                RETURN Outcome(status=PARTIAL_SUCCESS)

        IF join_policy == "first_success":
            IF success_count > 0:
                RETURN Outcome(status=SUCCESS)
            ELSE:
                RETURN Outcome(status=FAIL)

        -- 5. Store results in context for downstream fan-in
        context.set("parallel.results", serialize_results(results))
        RETURN Outcome(status=SUCCESS)
```

**Join policies:**

| Policy           | Behavior |
|------------------|----------|
| `wait_all`       | All branches must complete. Join satisfied when all are done. |
| `k_of_n`         | At least K branches must succeed. |
| `first_success`  | Join satisfied as soon as one branch succeeds. Others may be cancelled. |
| `quorum`         | At least a configurable fraction of branches must succeed. |

**Error policies:**

| Policy              | Behavior |
|---------------------|----------|
| `fail_fast`         | Cancel all remaining branches on first failure. |
| `continue`          | Continue remaining branches. Collect all results. |
| `ignore`            | Ignore failures entirely. Return only successful results. |

### 4.9 Fan-In Handler

Consolidates results from a preceding parallel node and selects the best candidate.

```
FanInHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        -- 1. Read parallel results
        results = context.get("parallel.results")
        IF results is empty:
            RETURN Outcome(status=FAIL, failure_reason="No parallel results to evaluate")

        -- 2. Evaluate candidates
        IF node.prompt is not empty:
            -- LLM-based evaluation: call LLM to rank candidates
            best = llm_evaluate(node.prompt, results)
        ELSE:
            -- Heuristic: rank by outcome status, then by score
            best = heuristic_select(results)

        -- 3. Record winner in context
        context_updates = {
            "parallel.fan_in.best_id": best.id,
            "parallel.fan_in.best_outcome": best.outcome
        }

        RETURN Outcome(
            status=SUCCESS,
            context_updates=context_updates,
            notes="Selected best candidate: " + best.id
        )


FUNCTION heuristic_select(candidates):
    outcome_rank = {SUCCESS: 0, PARTIAL_SUCCESS: 1, RETRY: 2, FAIL: 3}
    SORT candidates BY (outcome_rank[c.outcome], -c.score, c.id)
    RETURN candidates[0]
```

Fan-in runs even when some candidates failed, as long as at least one candidate is available. Only when all candidates fail does fan-in return FAIL.

### 4.10 Tool Handler

Executes an external tool (shell command, API call, or other non-LLM operation) configured via node attributes.

```
ToolHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        command = node.attrs.get("tool_command", "")
        IF command is empty:
            RETURN Outcome(status=FAIL, failure_reason="No tool_command specified")

        -- Execute the command
        TRY:
            result = run_shell_command(command, timeout=node.timeout)
            RETURN Outcome(
                status=SUCCESS,
                context_updates={"tool.output": result.stdout},
                notes="Tool completed: " + command
            )
        CATCH exception:
            RETURN Outcome(status=FAIL, failure_reason=str(exception))
```

### 4.11 Manager Loop Handler

Orchestrates sprint-based iteration by supervising a child pipeline. The manager observes the child's telemetry, evaluates progress via a guard function, and optionally steers the child through intervention.

```
ManagerLoopHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        child_dotfile = graph.attrs.get("stack.child_dotfile")
        poll_interval = parse_duration(node.attrs.get("manager.poll_interval", "45s"))
        max_cycles = integer(node.attrs.get("manager.max_cycles", "1000"))
        stop_condition = node.attrs.get("manager.stop_condition", "")
        actions = split(node.attrs.get("manager.actions", "observe,wait"), ",")

        -- 1. Auto-start child if configured
        IF node.attrs.get("stack.child_autostart", "true") == "true":
            start_child_pipeline(child_dotfile)

        -- 2. Observation loop
        FOR cycle FROM 1 TO max_cycles:
            IF "observe" IN actions:
                ingest_child_telemetry(context)

            IF "steer" IN actions AND steer_cooldown_elapsed():
                steer_child(context, node)

            -- Evaluate stop conditions
            child_status = context.get_string("context.stack.child.status")
            IF child_status IN {"completed", "failed"}:
                child_outcome = context.get_string("context.stack.child.outcome")
                IF child_outcome == "success":
                    RETURN Outcome(status=SUCCESS, notes="Child completed")
                IF child_status == "failed":
                    RETURN Outcome(status=FAIL, failure_reason="Child failed")

            IF stop_condition is not empty:
                IF evaluate_condition(stop_condition, ..., context):
                    RETURN Outcome(status=SUCCESS, notes="Stop condition satisfied")

            IF "wait" IN actions:
                sleep(poll_interval)

        RETURN Outcome(status=FAIL, failure_reason="Max cycles exceeded")
```

The manager pattern implements a **supervisor architecture** where:
- **Observe** ingests worker telemetry (active stage, outcomes, retry counts, artifacts)
- **Guard** scores worker progress and routes to continue, intervene, or escalate
- **Steer** writes intervention instructions to the child's active stage directory

### 4.12 Custom Handlers

New handler types are added by implementing the Handler interface and registering with the registry:

```
-- Define a custom handler
MyCustomHandler:
    FUNCTION execute(node, context, graph, logs_root) -> Outcome:
        -- Custom logic here
        RETURN Outcome(status=SUCCESS)

-- Register it
registry.register("my_custom_type", MyCustomHandler())

-- Reference in DOT file
my_node [type="my_custom_type", shape=box, custom_attr="value"]
```

**Handler contract:**
- Handlers MUST be stateless or protect shared mutable state with synchronization.
- Handler panics/exceptions MUST be caught by the engine and converted to FAIL outcomes.
- Handlers SHOULD NOT embed provider-specific logic; LLM orchestration is delegated to the integrated SDK.

---

## 5. State and Context

### 5.1 Context

The context is a thread-safe key-value store shared across all stages during a pipeline run. It is the primary mechanism for passing data between nodes.

```
Context:
    values : Map<String, Any>      -- key-value store
    lock   : ReadWriteLock         -- thread safety for parallel access
    logs   : List<String>          -- append-only run log

    FUNCTION set(key, value):
        ACQUIRE write lock
        values[key] = value
        RELEASE write lock

    FUNCTION get(key, default=NONE) -> Any:
        ACQUIRE read lock
        result = values.get(key, default)
        RELEASE read lock
        RETURN result

    FUNCTION get_string(key, default="") -> String:
        value = get(key)
        IF value is NONE: RETURN default
        RETURN string(value)

    FUNCTION append_log(entry):
        ACQUIRE write lock
        logs.append(entry)
        RELEASE write lock

    FUNCTION snapshot() -> Map<String, Any>:
        -- Returns a serializable copy of all values
        ACQUIRE read lock
        result = shallow_copy(values)
        RELEASE read lock
        RETURN result

    FUNCTION clone() -> Context:
        -- Deep copy for parallel branch isolation
        ACQUIRE read lock
        new_context = new Context()
        new_context.values = shallow_copy(values)
        new_context.logs = copy(logs)
        RELEASE read lock
        RETURN new_context

    FUNCTION apply_updates(updates):
        -- Merge a dictionary of updates into the context
        ACQUIRE write lock
        FOR EACH (key, value) IN updates:
            values[key] = value
        RELEASE write lock
```

**Built-in context keys set by the engine:**

| Key                                   | Type    | Set By   | Description |
|---------------------------------------|---------|----------|-------------|
| `outcome`                             | String  | Engine   | Last handler outcome status (`success`, `fail`, etc.) |
| `preferred_label`                     | String  | Engine   | Last handler's preferred edge label |
| `graph.goal`                          | String  | Engine   | Mirrored from graph `goal` attribute |
| `current_node`                        | String  | Engine   | ID of the currently executing node |
| `last_stage`                          | String  | Handler  | ID of the last completed stage |
| `last_response`                       | String  | Handler  | Truncated text of the last LLM response |
| `internal.retry_count.<node_id>`      | Integer | Engine   | Retry counter for a specific node |

**Context key namespace conventions:**

| Prefix        | Purpose                                        |
|---------------|------------------------------------------------|
| `context.*`   | Semantic state shared between nodes            |
| `graph.*`     | Graph attributes mirrored at initialization    |
| `internal.*`  | Engine bookkeeping (retry counters, timing)    |
| `parallel.*`  | Parallel handler state (results, counts)       |
| `stack.*`     | Supervisor/worker state                        |
| `human.gate.*`| Human interaction state                        |
| `work.*`      | Per-item context for parallel work items       |

### 5.2 Outcome

The outcome is the result of executing a node handler. It drives routing decisions and state updates.

```
Outcome:
    status             : StageStatus     -- SUCCESS, FAIL, PARTIAL_SUCCESS, RETRY, SKIPPED
    preferred_label    : String          -- which edge label to follow (optional)
    suggested_next_ids : List<String>    -- explicit next node IDs (optional)
    context_updates    : Map<String, Any> -- key-value pairs to merge into context
    notes              : String          -- human-readable execution summary
    failure_reason     : String          -- reason for failure (when status is FAIL or RETRY)
```

**StageStatus values:**

| Status             | Meaning |
|--------------------|---------|
| `SUCCESS`          | Stage completed its work. Proceed to next edge. Reset retry counter. |
| `PARTIAL_SUCCESS`  | Stage completed with caveats. Treated as success for routing but notes describe what was incomplete. |
| `RETRY`            | Stage requests re-execution. Engine increments retry counter and re-executes if within limits. |
| `FAIL`             | Stage failed permanently. Engine looks for a fail edge or terminates the pipeline. |
| `SKIPPED`          | Stage was skipped (e.g., condition not met). Proceed without recording an outcome. |

### 5.3 Checkpoint

A serializable snapshot of execution state, saved after each node completes. Enables crash recovery and resume.

```
Checkpoint:
    timestamp       : Timestamp              -- when this checkpoint was created
    current_node    : String                  -- ID of the last completed node
    completed_nodes : List<String>            -- IDs of all completed nodes in order
    node_retries    : Map<String, Integer>    -- retry counters per node
    context_values  : Map<String, Any>        -- serialized snapshot of the context
    logs            : List<String>            -- run log entries

    FUNCTION save(path):
        -- Serialize to JSON and write to filesystem
        data = {
            "timestamp": timestamp,
            "current_node": current_node,
            "completed_nodes": completed_nodes,
            "node_retries": node_retries,
            "context": serialize_to_json(context_values),
            "logs": logs
        }
        write_json_file(path, data)

    FUNCTION load(path) -> Checkpoint:
        -- Deserialize from JSON file
        data = read_json_file(path)
        RETURN new Checkpoint from data
```

**Resume behavior:**

1. Load the checkpoint from `{logs_root}/checkpoint.json`.
2. Restore context state from `context_values`.
3. Restore `completed_nodes` to skip already-finished work.
4. Restore retry counters from `node_retries`.
5. Determine the next node to execute (the one after `current_node` in the traversal).
6. If the previous node used `full` fidelity, degrade to `summary:high` for the first resumed node, because in-memory LLM sessions cannot be serialized. After this one degraded hop, subsequent nodes may use `full` fidelity again.

### 5.4 Context Fidelity

Context fidelity controls how much prior conversation and state is carried into the next node's LLM session. This is a core mechanism for managing context window usage across multi-stage pipelines.

```
FidelityMode ::= 'full'
               | 'truncate'
               | 'compact'
               | 'summary:low'
               | 'summary:medium'
               | 'summary:high'
```

| Mode             | Session | Context Carried                                         | Approximate Token Budget |
|------------------|---------|---------------------------------------------------------|--------------------------|
| `full`           | Reused (same thread) | Full conversation history preserved                      | Unbounded (uses compaction) |
| `truncate`       | Fresh   | Minimal: only graph goal and run ID                      | Minimal |
| `compact`        | Fresh   | Structured bullet-point summary: completed stages, outcomes, key context values | Moderate |
| `summary:low`    | Fresh   | Brief textual summary with minimal event counts          | ~600 tokens |
| `summary:medium` | Fresh   | Moderate detail: recent stage outcomes, active context values, notable events | ~1500 tokens |
| `summary:high`   | Fresh   | Detailed: many recent events, tool call summaries, comprehensive context | ~3000 tokens |

**Fidelity resolution precedence (highest to lowest):**

1. Edge `fidelity` attribute (on the incoming edge)
2. Target node `fidelity` attribute
3. Graph `default_fidelity` attribute
4. Default: `compact`

**Thread resolution (for `full` fidelity):**

When fidelity resolves to `full`, the engine determines a thread key for session reuse:

1. Target node `thread_id` attribute
2. Edge `thread_id` attribute
3. Graph-level default thread
4. Derived class from enclosing subgraph
5. Fallback: previous node ID

Nodes that share the same thread key reuse the same LLM session. Nodes with different thread keys start fresh sessions.

### 5.5 Artifact Store

The artifact store provides named, typed storage for large stage outputs that do not belong in the context (which should contain only small scalar values for routing and checkpoint serialization).

```
ArtifactStore:
    artifacts : Map<String, (ArtifactInfo, Any)>
    lock      : ReadWriteLock
    base_dir  : String or NONE   -- filesystem directory for file-backed artifacts

    FUNCTION store(artifact_id, name, data) -> ArtifactInfo:
        size = byte_size(data)
        is_file_backed = (size > FILE_BACKING_THRESHOLD) AND (base_dir is not NONE)
        IF is_file_backed:
            write data to "{base_dir}/artifacts/{artifact_id}.json"
            stored_data = file_path
        ELSE:
            stored_data = data
        info = ArtifactInfo(id=artifact_id, name=name, size=size, is_file_backed=is_file_backed)
        artifacts[artifact_id] = (info, stored_data)
        RETURN info

    FUNCTION retrieve(artifact_id) -> Any:
        IF artifact_id NOT IN artifacts:
            RAISE "Artifact not found"
        (info, data) = artifacts[artifact_id]
        IF info.is_file_backed:
            RETURN read_json_file(data)
        RETURN data

    FUNCTION has(artifact_id) -> Boolean
    FUNCTION list() -> List<ArtifactInfo>
    FUNCTION remove(artifact_id)
    FUNCTION clear()

ArtifactInfo:
    id              : String
    name            : String
    size_bytes      : Integer
    stored_at       : Timestamp
    is_file_backed  : Boolean
```

The default file-backing threshold is 100KB. Artifacts below this threshold are stored in memory; above it, they are written to disk.

### 5.6 Run Directory Structure

Each pipeline execution produces a directory tree for logging, checkpoints, and artifacts:

```
{logs_root}/
    checkpoint.json              -- Serialized checkpoint after each node
    manifest.json                -- Pipeline metadata (name, goal, start time)
    {node_id}/
        status.json              -- Node execution outcome
        prompt.md                -- Rendered prompt sent to LLM
        response.md              -- LLM response text
    artifacts/
        {artifact_id}.json       -- File-backed artifacts
```

---

## 6. Human-in-the-Loop (Interviewer Pattern)

### 6.1 Interviewer Interface

All human interaction in Attractor goes through an Interviewer interface. This abstraction allows the pipeline to present questions to a human and receive answers through any frontend: CLI, web UI, Slack bot, or a programmatic queue for testing.

```
INTERFACE Interviewer:
    FUNCTION ask(question: Question) -> Answer
    FUNCTION ask_multiple(questions: List<Question>) -> List<Answer>
    FUNCTION inform(message: String, stage: String) -> Void
```

### 6.2 Question Model

```
Question:
    text            : String              -- the question to present to the human
    type            : QuestionType        -- determines the UI and valid answers
    options         : List<Option>        -- for MULTIPLE_CHOICE type
    default         : Answer or NONE      -- default if timeout/skip
    timeout_seconds : Float or NONE       -- max wait time
    stage           : String              -- originating stage name (for display)
    metadata        : Map<String, Any>    -- arbitrary key-value pairs

QuestionType:
    YES_NO              -- yes/no binary choice
    MULTIPLE_CHOICE     -- select one from a list of options
    FREEFORM            -- free text input
    CONFIRMATION        -- yes/no confirmation (semantically distinct from YES_NO)

Option:
    key   : String    -- accelerator key (e.g., "Y", "A")
    label : String    -- display text (e.g., "Yes, deploy to production")
```

### 6.3 Answer Model

```
Answer:
    value           : String or AnswerValue   -- the selected value
    selected_option : Option or NONE          -- the full selected option (for MULTIPLE_CHOICE)
    text            : String                  -- free text response (for FREEFORM)

AnswerValue:
    YES       -- affirmative
    NO        -- negative
    SKIPPED   -- human skipped the question
    TIMEOUT   -- no response within timeout
```

### 6.4 Built-In Interviewer Implementations

**AutoApproveInterviewer:** Always selects YES for yes/no questions and the first option for multiple choice. Used for automated testing and CI/CD pipelines where no human is available.

```
AutoApproveInterviewer:
    FUNCTION ask(question) -> Answer:
        IF question.type IN {YES_NO, CONFIRMATION}:
            RETURN Answer(value=YES)
        IF question.type == MULTIPLE_CHOICE AND question.options is not empty:
            RETURN Answer(value=question.options[0].key, selected_option=question.options[0])
        RETURN Answer(value="auto-approved", text="auto-approved")
```

**ConsoleInterviewer (CLI):** Reads from standard input. Displays formatted prompts with option keys. Supports timeout via non-blocking read.

```
ConsoleInterviewer:
    FUNCTION ask(question) -> Answer:
        print("[?] " + question.text)
        IF question.type == MULTIPLE_CHOICE:
            FOR EACH option IN question.options:
                print("  [" + option.key + "] " + option.label)
            response = read_input("Select: ")
            RETURN find_matching_option(response, question.options)
        IF question.type == YES_NO:
            response = read_input("[Y/N]: ")
            RETURN Answer(value=YES if response is "y" ELSE NO)
        IF question.type == FREEFORM:
            response = read_input("> ")
            RETURN Answer(text=response)
```

**CallbackInterviewer:** Delegates question answering to a provided callback function. Useful for integrating with external systems (Slack, web UI, API).

```
CallbackInterviewer:
    callback : Function(Question) -> Answer

    FUNCTION ask(question) -> Answer:
        RETURN callback(question)
```

**QueueInterviewer:** Reads answers from a pre-filled answer queue. Used for deterministic testing and replay.

```
QueueInterviewer:
    answers : Queue<Answer>

    FUNCTION ask(question) -> Answer:
        IF answers is not empty:
            RETURN answers.dequeue()
        RETURN Answer(value=SKIPPED)
```

**RecordingInterviewer:** Wraps another interviewer and records all question-answer pairs. Used for replay, debugging, and audit trails.

```
RecordingInterviewer:
    inner      : Interviewer
    recordings : List<(Question, Answer)>

    FUNCTION ask(question) -> Answer:
        answer = inner.ask(question)
        recordings.append((question, answer))
        RETURN answer
```

### 6.5 Timeout Handling

If a human does not respond within the configured `timeout_seconds`:

1. If the question has a `default` answer, use it.
2. If no default, return `Answer(value=TIMEOUT)`.
3. The handler decides how to handle a timeout (retry the question, fail, or proceed with assumptions).

For `wait.human` nodes, the node attribute `human.default_choice` specifies which edge target to select on timeout.

---

## 7. Validation and Linting

### 7.1 Diagnostic Model

Validation produces a list of diagnostics, each with a severity level. The engine must refuse to execute a pipeline with error-severity diagnostics.

```
Diagnostic:
    rule     : String                    -- rule identifier (e.g., "start_node")
    severity : Severity                  -- ERROR, WARNING, or INFO
    message  : String                    -- human-readable description
    node_id  : String                    -- related node ID (optional)
    edge     : (String, String) or NONE  -- related edge as (from, to) (optional)
    fix      : String                    -- suggested fix (optional)

Severity:
    ERROR     -- pipeline will not execute
    WARNING   -- pipeline will execute but behavior may be unexpected
    INFO      -- informational note
```

### 7.2 Built-In Lint Rules

| Rule ID                  | Severity | Description |
|--------------------------|----------|-------------|
| `start_node`             | ERROR    | Pipeline must have exactly one start node (shape=Mdiamond or id matching `start`/`Start`). |
| `terminal_node`          | ERROR    | Pipeline must have at least one terminal node (shape=Msquare or id matching `exit`/`end`). |
| `reachability`           | ERROR    | All nodes must be reachable from the start node via BFS/DFS traversal. |
| `edge_target_exists`     | ERROR    | Every edge target must reference an existing node ID. |
| `start_no_incoming`      | ERROR    | The start node must have no incoming edges. |
| `exit_no_outgoing`       | ERROR    | The exit node must have no outgoing edges. |
| `condition_syntax`       | ERROR    | Edge condition expressions must parse correctly (valid operators and keys). |
| `stylesheet_syntax`      | ERROR    | The `model_stylesheet` attribute must parse as valid stylesheet rules. |
| `type_known`             | WARNING  | Node `type` values should be recognized by the handler registry. |
| `fidelity_valid`         | WARNING  | Fidelity mode values must be one of: `full`, `truncate`, `compact`, `summary:low`, `summary:medium`, `summary:high`. |
| `retry_target_exists`    | WARNING  | `retry_target` and `fallback_retry_target` must reference existing nodes. |
| `goal_gate_has_retry`    | WARNING  | Nodes with `goal_gate=true` should have a `retry_target` or `fallback_retry_target`. |
| `prompt_on_llm_nodes`    | WARNING  | Nodes that resolve to the codergen handler should have a `prompt` or `label` attribute. |

### 7.3 Validation API

```
FUNCTION validate(graph, extra_rules=NONE) -> List<Diagnostic>:
    rules = BUILT_IN_RULES
    IF extra_rules is not NONE:
        rules = rules + extra_rules
    diagnostics = []
    FOR EACH rule IN rules:
        diagnostics.extend(rule.apply(graph))
    RETURN diagnostics


FUNCTION validate_or_raise(graph, extra_rules=NONE):
    diagnostics = validate(graph, extra_rules)
    errors = [d FOR d IN diagnostics WHERE d.severity == ERROR]
    IF errors is not empty:
        RAISE ValidationError with error messages
    RETURN diagnostics
```

### 7.4 Custom Lint Rules

Implementations may register custom lint rules by implementing the rule interface:

```
INTERFACE LintRule:
    name : String
    FUNCTION apply(graph) -> List<Diagnostic>
```

Custom rules are appended to the built-in rules and run during validation.

---

## 8. Model Stylesheet

### 8.1 Overview

The `model_stylesheet` graph attribute provides CSS-like rules for setting default LLM configuration on nodes. This centralizes model selection so that individual nodes do not need to specify `llm_model`, `llm_provider`, and `reasoning_effort` on every node.

### 8.2 Stylesheet Grammar

```
Stylesheet    ::= Rule+
Rule          ::= Selector '{' Declaration ( ';' Declaration )* ';'? '}'
Selector      ::= '*' | '#' Identifier | '.' ClassName
ClassName     ::= [a-z0-9-]+
Declaration   ::= Property ':' PropertyValue
Property      ::= 'llm_model' | 'llm_provider' | 'reasoning_effort'
PropertyValue ::= String | 'low' | 'medium' | 'high'
```

### 8.3 Selectors and Specificity

| Selector      | Matches                      | Specificity |
|---------------|------------------------------|-------------|
| `*`           | All nodes                    | 0 (lowest)  |
| `.class_name` | Nodes with that class        | 1 (medium)  |
| `#node_id`    | Specific node by ID          | 2 (highest) |

Later rules of equal specificity override earlier ones. Explicit node attributes always override stylesheet values (highest precedence).

### 8.4 Recognized Properties

| Property           | Values                     | Description |
|--------------------|----------------------------|-------------|
| `llm_model`        | Any model identifier string | Provider-native model ID (e.g., `gpt-5.2`, `claude-opus-4-6`) |
| `llm_provider`     | Provider key string         | `openai`, `anthropic`, `gemini`, etc. |
| `reasoning_effort`  | `low`, `medium`, `high`    | Controls reasoning/thinking depth for the LLM |

### 8.5 Application Order

The resolution order for any model-related property on a node is:

1. Explicit node attribute (e.g., `llm_model="gpt-5.2"` on the node) -- highest precedence
2. Stylesheet rule matching by specificity (ID > class > universal)
3. Graph-level default attribute
4. Handler/system default

The stylesheet is applied as a transform after parsing and before validation. The transform walks all nodes and applies matching rules, but only sets properties that the node does not already have explicitly.

### 8.6 Example

```
digraph Pipeline {
    graph [
        goal="Implement feature X",
        model_stylesheet="
            * { llm_model: claude-sonnet-4-5; llm_provider: anthropic; }
            .code { llm_model: claude-opus-4-6; llm_provider: anthropic; }
            #critical_review { llm_model: gpt-5.2; llm_provider: openai; reasoning_effort: high; }
        "
    ]

    start [shape=Mdiamond]
    exit  [shape=Msquare]

    plan            [label="Plan", class="planning"]
    implement       [label="Implement", class="code"]
    critical_review [label="Critical Review", class="code"]

    start -> plan -> implement -> critical_review -> exit
}
```

In this example:
- `plan` gets `claude-sonnet-4-5` from the `*` rule (no class match for `.code`).
- `implement` gets `claude-opus-4-6` from the `.code` rule.
- `critical_review` gets `gpt-5.2` from the `#critical_review` rule (highest specificity), overriding the `.code` class match.

---

## 9. Transforms and Extensibility

### 9.1 AST Transforms

Transforms are functions that modify the pipeline graph after parsing and before validation. They enable preprocessing, optimization, and structural rewriting without modifying the original DOT file.

```
INTERFACE Transform:
    FUNCTION apply(graph) -> Graph
    -- Returns a new or modified graph. Should not modify the input graph.
```

Transforms are applied in a defined order after parsing and before validation:

```
FUNCTION prepare_pipeline(dot_source):
    graph = parse(dot_source)
    FOR EACH transform IN transforms:
        graph = transform.apply(graph)
    diagnostics = validate(graph)
    RETURN (graph, diagnostics)
```

### 9.2 Built-In Transforms

**Variable Expansion Transform:** Expands `$goal` in node `prompt` attributes to the graph-level `goal` attribute value.

```
VariableExpansionTransform:
    FUNCTION apply(graph) -> Graph:
        FOR EACH node IN graph.nodes:
            IF node.prompt contains "$goal":
                node.prompt = replace(node.prompt, "$goal", graph.goal)
        RETURN graph
```

**Stylesheet Application Transform:** Applies the `model_stylesheet` to resolve `llm_model`, `llm_provider`, and `reasoning_effort` for each node. See Section 8 for details.

**Preamble Transform:** Synthesizes context carryover text for stages that do not use `full` fidelity. Applied at execution time (not at parse time) since it depends on runtime state.

### 9.3 Custom Transforms

Implementations may register custom transforms:

```
runner.register_transform(MyCustomTransform())
```

Custom transforms run after built-in transforms. Order of custom transforms follows registration order.

**Use cases for custom transforms:**
- Inject logging or auditing nodes into the graph
- Add retry wrappers around certain node types
- Merge multiple graphs into a single pipeline
- Apply organization-specific defaults

### 9.4 Pipeline Composition

Attractor supports combining multiple DOT graphs through:

**Sub-pipeline nodes:** A node whose handler runs an entire sub-graph as its execution. The manager loop handler (Section 4.11) is an example of this pattern.

**Graph merging (via transform):** A custom transform can merge nodes and edges from one graph into another, enabling modular pipeline definitions.

### 9.5 HTTP Server Mode

Implementations may expose the pipeline engine as an HTTP service for web-based management, remote human interaction, and integration with external systems.

**Core endpoints:**

| Method | Path                                    | Description |
|--------|-----------------------------------------|-------------|
| `POST` | `/pipelines`                            | Submit a DOT source and start execution. Returns pipeline ID. |
| `GET`  | `/pipelines/{id}`                       | Get pipeline status and progress. |
| `GET`  | `/pipelines/{id}/events`                | SSE stream of pipeline events in real-time. |
| `POST` | `/pipelines/{id}/cancel`                | Cancel a running pipeline. |
| `GET`  | `/pipelines/{id}/graph`                 | Get rendered graph visualization (SVG). |
| `GET`  | `/pipelines/{id}/questions`             | Get pending human interaction questions. |
| `POST` | `/pipelines/{id}/questions/{qid}/answer`| Submit answer to a pending question. |
| `GET`  | `/pipelines/{id}/checkpoint`            | Get current checkpoint state. |
| `GET`  | `/pipelines/{id}/context`               | Get current context key-value store. |

Human gates must be operable via web controls in addition to CLI. The server maintains SSE connections for real-time event streaming.

### 9.6 Observability and Events

The engine emits typed events during execution for UI, logging, and metrics integration:

**Pipeline lifecycle events:**
- `PipelineStarted(name, id)` -- pipeline begins
- `PipelineCompleted(duration, artifact_count)` -- pipeline succeeded
- `PipelineFailed(error, duration)` -- pipeline failed

**Stage lifecycle events:**
- `StageStarted(name, index)` -- stage begins
- `StageCompleted(name, index, duration)` -- stage succeeded
- `StageFailed(name, index, error, will_retry)` -- stage failed
- `StageRetrying(name, index, attempt, delay)` -- stage retrying

**Parallel execution events:**
- `ParallelStarted(branch_count)` -- parallel block started
- `ParallelBranchStarted(branch, index)` -- branch started
- `ParallelBranchCompleted(branch, index, duration, success)` -- branch done
- `ParallelCompleted(duration, success_count, failure_count)` -- all branches done

**Human interaction events:**
- `InterviewStarted(question, stage)` -- question presented
- `InterviewCompleted(question, answer, duration)` -- answer received
- `InterviewTimeout(question, stage, duration)` -- timeout reached

**Checkpoint events:**
- `CheckpointSaved(node_id)` -- checkpoint written

Events can be consumed via an observer/callback pattern or an asynchronous stream:

```
-- Observer pattern
runner.on_event = FUNCTION(event):
    log(event.description)

-- Stream pattern (for async runtimes)
FOR EACH event IN pipeline.events():
    process(event)
```

### 9.7 Tool Call Hooks

Graph-level or node-level attributes `tool_hooks.pre` and `tool_hooks.post` specify shell commands executed around each LLM tool call:

- **Pre-hook:** Executed before every LLM tool call. Receives tool metadata via environment variables and stdin JSON. Exit code 0 means proceed; non-zero means skip the tool call.
- **Post-hook:** Executed after every LLM tool call. Receives tool metadata and result. Primarily for logging and auditing.

Hook failures (non-zero exit) do not block the tool call but are recorded in the stage log.

---

## 10. Condition Expression Language

### 10.1 Overview

Edge conditions use a minimal boolean expression language to gate edge eligibility during routing. The language is deliberately simple to keep routing deterministic and inspectable.

### 10.2 Grammar

```
ConditionExpr  ::= Clause ( '&&' Clause )*
Clause         ::= Key Operator Literal
Key            ::= 'outcome'
                 | 'preferred_label'
                 | 'context.' Path
Path           ::= Identifier ( '.' Identifier )*
Operator       ::= '=' | '!='
Literal        ::= String | Integer | Boolean
```

### 10.3 Semantics

- Clauses are AND-combined, evaluated left to right.
- `outcome` refers to the executing node's outcome status (`success`, `retry`, `fail`, `partial_success`).
- `preferred_label` refers to the `preferred_label` value from the node's outcome.
- `context.*` keys look up values from the run context. Missing keys compare as empty strings (never equal to non-empty values).
- String comparison is exact and case-sensitive.
- All clauses must evaluate to true for the condition to pass.

### 10.4 Variable Resolution

```
FUNCTION resolve_key(key, outcome, context) -> String:
    IF key == "outcome":
        RETURN outcome.status as string
    IF key == "preferred_label":
        RETURN outcome.preferred_label
    IF key starts with "context.":
        value = context.get(key)
        IF value is not NONE:
            RETURN string(value)
        -- Also try without "context." prefix for convenience
        value = context.get(key without "context." prefix)
        IF value is not NONE:
            RETURN string(value)
        RETURN ""
    -- Direct context lookup for unqualified keys
    value = context.get(key)
    IF value is not NONE:
        RETURN string(value)
    RETURN ""
```

### 10.5 Evaluation

```
FUNCTION evaluate_condition(condition, outcome, context) -> Boolean:
    IF condition is empty:
        RETURN true  -- no condition means always eligible

    clauses = split(condition, "&&")
    FOR EACH clause IN clauses:
        clause = trim(clause)
        IF clause is empty:
            CONTINUE
        IF NOT evaluate_clause(clause, outcome, context):
            RETURN false
    RETURN true


FUNCTION evaluate_clause(clause, outcome, context) -> Boolean:
    IF clause contains "!=":
        (key, value) = split(clause, "!=", max=1)
        RETURN resolve_key(trim(key), outcome, context) != trim(value)
    ELSE IF clause contains "=":
        (key, value) = split(clause, "=", max=1)
        RETURN resolve_key(trim(key), outcome, context) == trim(value)
    ELSE:
        -- Bare key: check if truthy
        RETURN bool(resolve_key(trim(clause), outcome, context))
```

### 10.6 Examples

```
-- Route on success
plan -> implement [condition="outcome=success"]

-- Route on failure
plan -> fix [condition="outcome=fail"]

-- Route on success AND a context flag
validate -> deploy [condition="outcome=success && context.tests_passed=true"]

-- Route when a context value is not a specific value
review -> iterate [condition="context.loop_state!=exhausted"]

-- Route based on preferred label
gate -> fix [condition="preferred_label=Fix"]
```

### 10.7 Extended Operators (Future)

The current condition language supports only `=` (equals) and `!=` (not equals) with AND (`&&`) conjunction. Future versions may add:

- `contains` -- substring or set membership
- `matches` -- regular expression matching
- `OR` -- disjunction
- `NOT` -- negation
- `>`, `<`, `>=`, `<=` -- numeric comparison

These are documented here as potential extensions. Implementations should not add them without updating the grammar and validation rules.

---

## 11. Definition of Done

This section defines how to validate that an implementation of this spec is complete and correct. An implementation is done when every item is checked off.

### 11.1 DOT Parsing

- [ ] Parser accepts the supported DOT subset (digraph with graph/node/edge attribute blocks)
- [ ] Graph-level attributes (`goal`, `label`, `model_stylesheet`) are extracted correctly
- [ ] Node attributes are parsed including multi-line attribute blocks (attributes spanning multiple lines within `[...]`)
- [ ] Edge attributes (`label`, `condition`, `weight`) are parsed correctly
- [ ] Chained edges (`A -> B -> C`) produce individual edges for each pair
- [ ] Node/edge default blocks (`node [...]`, `edge [...]`) apply to subsequent declarations
- [ ] Subgraph blocks are flattened (contents kept, wrapper removed)
- [ ] `class` attribute on nodes merges in attributes from the stylesheet
- [ ] Quoted and unquoted attribute values both work
- [ ] Comments (`//` and `/* */`) are stripped before parsing

### 11.2 Validation and Linting

- [ ] Exactly one start node (shape=Mdiamond) is required
- [ ] Exactly one exit node (shape=Msquare) is required
- [ ] Start node has no incoming edges
- [ ] Exit node has no outgoing edges
- [ ] All nodes are reachable from start (no orphans)
- [ ] All edges reference valid node IDs
- [ ] Codergen nodes (shape=box) have non-empty `prompt` attribute (warning if missing)
- [ ] Condition expressions on edges parse without errors
- [ ] `validate_or_raise()` throws on error-severity violations
- [ ] Lint results include rule name, severity (error/warning), node/edge ID, and message

### 11.3 Execution Engine

- [ ] Engine resolves the start node and begins execution there
- [ ] Each node's handler is resolved via shape-to-handler-type mapping
- [ ] Handler is called with (node, context, graph, logs_root) and returns an Outcome
- [ ] Outcome is written to `{logs_root}/{node_id}/status.json`
- [ ] Edge selection follows the 5-step priority: condition match -> preferred label -> suggested IDs -> weight -> lexical
- [ ] Engine loops: execute node -> select edge -> advance to next node -> repeat
- [ ] Terminal node (shape=Msquare) stops execution
- [ ] Pipeline outcome is "success" if all goal_gate nodes succeeded, "fail" otherwise

### 11.4 Goal Gate Enforcement

- [ ] Nodes with `goal_gate=true` are tracked throughout execution
- [ ] Before allowing exit via a terminal node, the engine checks all goal gate nodes have status SUCCESS
- [ ] If any goal gate node has not succeeded, the engine routes to `retry_target` (if configured) instead of exiting
- [ ] If no retry_target and goal gates unsatisfied, pipeline outcome is "fail"

### 11.5 Retry Logic

- [ ] Nodes with `max_retries > 0` are retried on RETRY or FAIL outcomes
- [ ] Retry count is tracked per-node and respects the configured limit
- [ ] Backoff between retries works (constant, linear, or exponential as configured)
- [ ] Jitter is applied to backoff delays when configured
- [ ] After retry exhaustion, the node's final outcome is used for edge selection

### 11.6 Node Handlers

- [ ] **Start handler:** Returns SUCCESS immediately (no-op)
- [ ] **Exit handler:** Returns SUCCESS immediately (no-op, engine checks goal gates)
- [ ] **Codergen handler:** Expands `$goal` in prompt, calls `CodergenBackend.run()`, writes prompt.md and response.md to stage dir
- [ ] **Wait.human handler:** Presents outgoing edge labels as choices to the interviewer, returns selected label as preferred_label
- [ ] **Conditional handler:** Passes through; engine evaluates edge conditions against outcome/context
- [ ] **Parallel handler:** Fans out to multiple target nodes concurrently (or sequentially as fallback)
- [ ] **Fan-in handler:** Waits for all parallel branches to complete before proceeding
- [ ] **Tool handler:** Executes configured tool/command and returns result
- [ ] Custom handlers can be registered by type string

### 11.7 State and Context

- [ ] Context is a key-value store accessible to all handlers
- [ ] Handlers can read context and return `context_updates` in the Outcome
- [ ] Context updates are merged after each node execution
- [ ] Checkpoint is saved after each node completion (current_node, completed_nodes, context, retry counts)
- [ ] Resume from checkpoint: load checkpoint -> restore state -> continue from current_node
- [ ] Artifacts are written to `{logs_root}/{node_id}/` (prompt.md, response.md, status.json)

### 11.8 Human-in-the-Loop

- [ ] Interviewer interface works: `ask(question) -> Answer`
- [ ] Question supports types: SINGLE_SELECT, MULTI_SELECT, FREE_TEXT, CONFIRM
- [ ] AutoApproveInterviewer always selects the first option (for automation/testing)
- [ ] ConsoleInterviewer prompts in terminal and reads user input
- [ ] CallbackInterviewer delegates to a provided function
- [ ] QueueInterviewer reads from a pre-filled answer queue (for testing)

### 11.9 Condition Expressions

- [ ] `=` (equals) operator works for string comparison
- [ ] `!=` (not equals) operator works
- [ ] `&&` (AND) conjunction works with multiple clauses
- [ ] `outcome` variable resolves to the current node's outcome status
- [ ] `preferred_label` variable resolves to the outcome's preferred label
- [ ] `context.*` variables resolve to context values (missing keys = empty string)
- [ ] Empty condition always evaluates to true (unconditional edge)

### 11.10 Model Stylesheet

- [ ] Stylesheet is parsed from the graph's `model_stylesheet` attribute
- [ ] Selectors by shape name work (e.g., `box { model = "claude-opus-4-6" }`)
- [ ] Selectors by class name work (e.g., `.fast { model = "gemini-3-flash-preview" }`)
- [ ] Selectors by node ID work (e.g., `#review { reasoning_effort = "high" }`)
- [ ] Specificity order: universal < shape < class < ID
- [ ] Stylesheet properties are overridden by explicit node attributes

### 11.11 Transforms and Extensibility

- [ ] AST transforms can modify the Graph between parsing and validation
- [ ] Transform interface: `transform(graph) -> graph`
- [ ] Built-in variable expansion transform replaces `$goal` in prompts
- [ ] Custom transforms can be registered and run in order
- [ ] HTTP server mode (if implemented): POST /run starts pipeline, GET /status checks state, POST /answer submits human input

### 11.12 Cross-Feature Parity Matrix

Run this validation matrix -- each cell must pass:

| Test Case                                        | Pass |
|--------------------------------------------------|------|
| Parse a simple linear pipeline (start -> A -> B -> done) | [ ] |
| Parse a pipeline with graph-level attributes (goal, label) | [ ] |
| Parse multi-line node attributes                 | [ ] |
| Validate: missing start node -> error            | [ ] |
| Validate: missing exit node -> error             | [ ] |
| Validate: orphan node -> warning                 | [ ] |
| Execute a linear 3-node pipeline end-to-end      | [ ] |
| Execute with conditional branching (success/fail paths) | [ ] |
| Execute with retry on failure (max_retries=2)    | [ ] |
| Goal gate blocks exit when unsatisfied            | [ ] |
| Goal gate allows exit when all satisfied          | [ ] |
| Wait.human presents choices and routes on selection | [ ] |
| Edge selection: condition match wins over weight  | [ ] |
| Edge selection: weight breaks ties for unconditional edges | [ ] |
| Edge selection: lexical tiebreak as final fallback | [ ] |
| Context updates from one node are visible to the next | [ ] |
| Checkpoint save and resume produces same result   | [ ] |
| Stylesheet applies model override to nodes by shape | [ ] |
| Prompt variable expansion ($goal) works           | [ ] |
| Parallel fan-out and fan-in complete correctly    | [ ] |
| Custom handler registration and execution works   | [ ] |
| Pipeline with 10+ nodes completes without errors  | [ ] |

### 11.13 Integration Smoke Test

End-to-end test with a real LLM callback:

```
-- Test pipeline: plan -> implement -> review -> done
DOT = """
digraph test_pipeline {
    graph [goal="Create a hello world Python script"]

    start       [shape=Mdiamond]
    plan        [shape=box, prompt="Plan how to create a hello world script for: $goal"]
    implement   [shape=box, prompt="Write the code based on the plan", goal_gate=true]
    review      [shape=box, prompt="Review the code for correctness"]
    done        [shape=Msquare]

    start -> plan
    plan -> implement
    implement -> review [condition="outcome=success"]
    implement -> plan   [condition="outcome=fail", label="Retry"]
    review -> done      [condition="outcome=success"]
    review -> implement [condition="outcome=fail", label="Fix"]
}
"""

-- 1. Parse
graph = parse_dot(DOT)
ASSERT graph.goal == "Create a hello world Python script"
ASSERT LENGTH(graph.nodes) == 5
ASSERT LENGTH(edges_total(graph)) == 6

-- 2. Validate
lint_results = validate(graph)
ASSERT no error-severity results in lint_results

-- 3. Execute with LLM callback
context = Context()
outcome = run_pipeline(graph, context, llm_callback = real_llm_callback)

-- 4. Verify
ASSERT outcome.status == "success"
ASSERT "implement" in outcome.completed_nodes
ASSERT artifacts_exist(logs_root, "plan", ["prompt.md", "response.md", "status.json"])
ASSERT artifacts_exist(logs_root, "implement", ["prompt.md", "response.md", "status.json"])
ASSERT artifacts_exist(logs_root, "review", ["prompt.md", "response.md", "status.json"])

-- 5. Verify goal gate
ASSERT goal_gate_satisfied(graph, outcome, "implement")

-- 6. Verify checkpoint
checkpoint = load_checkpoint(logs_root)
ASSERT checkpoint.current_node == "done"
ASSERT "plan" IN checkpoint.completed_nodes
ASSERT "implement" IN checkpoint.completed_nodes
ASSERT "review" IN checkpoint.completed_nodes
```

---

## Appendix A: Complete Attribute Reference

### Graph Attributes

| Key                     | Type     | Default | Description |
|-------------------------|----------|---------|-------------|
| `goal`                  | String   | `""`    | Pipeline-level goal description |
| `label`                 | String   | `""`    | Display name for the graph |
| `model_stylesheet`      | String   | `""`    | CSS-like LLM model/provider stylesheet |
| `default_max_retry`     | Integer  | `50`    | Global retry ceiling |
| `default_fidelity`      | String   | `""`    | Default context fidelity mode |
| `retry_target`          | String   | `""`    | Node to jump to on unsatisfied exit |
| `fallback_retry_target` | String   | `""`    | Secondary jump target |
| `stack.child_dotfile`   | String   | `""`    | Path to child DOT file for supervision |
| `stack.child_workdir`   | String   | cwd     | Working directory for child run |
| `tool_hooks.pre`        | String   | `""`    | Shell command before each tool call |
| `tool_hooks.post`       | String   | `""`    | Shell command after each tool call |

### Node Attributes

| Key                     | Type     | Default       | Description |
|-------------------------|----------|---------------|-------------|
| `label`                 | String   | node ID       | Display name |
| `shape`                 | String   | `"box"`       | Graphviz shape (determines handler type) |
| `type`                  | String   | `""`          | Explicit handler type override |
| `prompt`                | String   | `""`          | LLM prompt (supports `$goal` expansion) |
| `max_retries`           | Integer  | `0`           | Additional retry attempts |
| `goal_gate`             | Boolean  | `false`       | Must succeed before pipeline exit |
| `retry_target`          | String   | `""`          | Jump target on failure |
| `fallback_retry_target` | String   | `""`          | Secondary jump target |
| `fidelity`              | String   | inherited     | Context fidelity mode |
| `thread_id`             | String   | derived       | Session reuse key |
| `class`                 | String   | `""`          | Stylesheet class names (comma-separated) |
| `timeout`               | Duration | unset         | Max execution time |
| `llm_model`             | String   | inherited     | LLM model override |
| `llm_provider`          | String   | auto-detected | LLM provider override |
| `reasoning_effort`      | String   | `"high"`      | Reasoning depth: low/medium/high |
| `auto_status`           | Boolean  | `false`       | Auto-generate SUCCESS if no status written |
| `allow_partial`         | Boolean  | `false`       | Accept PARTIAL_SUCCESS on retry exhaustion |

### Edge Attributes

| Key            | Type     | Default | Description |
|----------------|----------|---------|-------------|
| `label`        | String   | `""`    | Display caption and routing key |
| `condition`    | String   | `""`    | Boolean guard expression |
| `weight`       | Integer  | `0`     | Priority for edge selection (higher wins) |
| `fidelity`     | String   | unset   | Override fidelity for target node |
| `thread_id`    | String   | unset   | Override thread ID for target node |
| `loop_restart` | Boolean  | `false` | Restart pipeline with fresh log directory |

---

## Appendix B: Shape-to-Handler-Type Mapping

| Shape           | Handler Type        | Default Behavior |
|-----------------|---------------------|------------------|
| `Mdiamond`      | `start`             | No-op entry point |
| `Msquare`       | `exit`              | No-op exit point (goal gate check in engine) |
| `box`           | `codergen`          | LLM task (default for all nodes) |
| `hexagon`       | `wait.human`        | Blocks for human selection |
| `diamond`       | `conditional`       | Pass-through; engine evaluates edge conditions |
| `component`     | `parallel`          | Concurrent branch execution |
| `tripleoctagon` | `parallel.fan_in`   | Consolidate parallel results |
| `parallelogram` | `tool`              | External tool execution |
| `house`         | `stack.manager_loop`| Supervisor polling loop |

---

## Appendix C: Status File Contract

Each non-terminal node writes a `status.json` file in its stage directory. This file drives routing decisions and provides an audit trail.

```
{
    "outcome": "success | retry | fail | partial_success",
    "preferred_next_label": "<edge label or empty>",
    "suggested_next_ids": ["<node_id>", ...],
    "context_updates": {
        "key": "value",
        "nested.key": "value"
    },
    "notes": "Human-readable execution summary"
}
```

| Field                  | Type            | Required | Description |
|------------------------|-----------------|----------|-------------|
| `outcome`              | String (enum)   | Yes      | Outcome status. Drives routing and goal checks. |
| `preferred_next_label` | String          | No       | Edge label to prioritize for next transition. |
| `suggested_next_ids`   | List of Strings | No       | Fallback target node IDs if no label match. |
| `context_updates`      | Map             | No       | Key-value pairs merged into the run context. |
| `notes`                | String          | No       | Human-readable log entries. |

When `auto_status=true` on a node and no `status.json` was written by the handler, the engine synthesizes: `{"outcome": "success", "notes": "auto-status: handler completed without writing status"}`.

---

## Appendix D: Error Categories

Every error during pipeline execution falls into one of three categories:

**Retryable errors** are transient failures where re-execution may succeed. Examples: LLM rate limits, network timeouts, temporary service unavailability. The engine retries these automatically per the node's retry policy.

**Terminal errors** are permanent failures where re-execution will not help. Examples: invalid prompt, missing required context, authentication failures. The engine does not retry these; it immediately routes to the failure path.

**Pipeline errors** are structural failures in the pipeline itself. Examples: no start node, unreachable nodes, invalid conditions. These are detected during validation (before execution) when possible. Runtime detection causes immediate pipeline termination.
