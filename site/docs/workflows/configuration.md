---
title: Configuration Reference
description: Full reference for WORKFLOW.md frontmatter and graph-level properties.
---

This page documents all properties available in the YAML frontmatter of a `WORKFLOW.md` file and the graph-level attributes of the DOT pipeline.

## Frontmatter Properties

### `name`

**Type:** `string` — **Required**

The name of the workflow. Must be lowercase kebab-case: 1–64 characters, only lowercase alphanumeric characters and hyphens, no leading/trailing/consecutive hyphens.

```yaml
name: code-review
```

### `description`

**Type:** `string` — **Required**

A brief description of the workflow. Must be non-empty.

```yaml
description: Automated code review with human approval gate
```

### `goal`

**Type:** `string`

The high-level goal for the pipeline. Expanded as `$goal` in node prompt templates. Can be overridden at runtime with `stencila workflows run --goal`.

```yaml
goal: Review recent literature on CRISPR gene editing
```

The goal can also be set as a graph-level attribute in the DOT source (see below). When set in both places, the frontmatter value takes precedence unless overridden at runtime.

## Graph-Level Attributes

These attributes are set on the `graph` declaration inside the DOT pipeline:

```dot
digraph my_workflow {
    graph [
        goal="Analyze experimental results",
        overrides="* { model: claude-sonnet-4-5; }",
        default_max_retry=3
    ]
    ...
}
```

### `goal`

**Type:** `string` — Default: `""`

Pipeline-level goal. Expanded as `$goal` in node prompts.

```dot
graph [goal="Systematic review of renewable energy storage"]
```

### `overrides`

**Type:** `string` — Default: `""`

CSS-like rules for per-node agent overrides. The primary way to configure models is through [agent definitions](../agents/configuration) referenced via the `agent` attribute on each node. The overrides mechanism provides a supplementary way to bulk-override agent properties across many nodes at once.

Supported properties: `model`, `provider`, `reasoning_effort`, `trust_level`, `max_turns`.

```dot
graph [overrides="
    * { model: claude-sonnet-4-5; provider: anthropic; }
    .analysis { model: claude-opus-4-6; }
    #review { model: o3; provider: openai; reasoning_effort: high; }
"]
```

Specificity order: `*` (universal) < `.class` < `#node_id`. Override values take precedence over agent defaults but are themselves overridden by explicit `agent.*` node attributes. See [Pipelines — Overrides](pipelines#overrides) for details.

### `default_max_retry`

**Type:** `integer` — Default: `0`

Global retry ceiling for nodes that omit `max_retries`.

```dot
graph [default_max_retry=3]
```

### `retry_target`

**Type:** `string` — Default: `""`

Node ID to jump to if the pipeline reaches the exit node with unsatisfied goal gates.

```dot
graph [retry_target="Design"]
```

### `fallback_retry_target`

**Type:** `string` — Default: `""`

Secondary jump target if `retry_target` is missing or invalid.

### `default_fidelity`

**Type:** `string` — Default: `""`

Default context fidelity mode for LLM sessions. Controls how context values are compressed or summarized.

Valid values: `full`, `truncate`, `compact`, `summary:low`, `summary:medium`, `summary:high`.

```dot
graph [default_fidelity="compact"]
```

## Node Attributes

Node attributes are set on individual nodes in the DOT pipeline. See [Pipelines — Common attributes](pipelines#common-attributes) for the full reference, including `prompt`, `agent`, `agent.model`, `agent.provider`, `agent.reasoning-effort`, `agent.trust-level`, `agent.max-turns`, `shape`, `max_retries`, `goal_gate`, `timeout`, and `class`. The `overrides` graph attribute provides bulk overrides that target nodes by ID or class.

## Edge Attributes

Edge attributes control transitions between nodes. See [Pipelines — Edge attributes](pipelines#attributes) for the full reference, including `label`, `condition`, and `weight`.
