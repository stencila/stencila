---
title: Using Workflows
description: How to list, inspect, run, and manage workflows from the CLI.
---

## Listing Workflows

See all discovered workflows (`stencila workflows` is shorthand for `stencila workflows list`):

```sh
stencila workflows
```

This shows workflows from `.stencila/workflows/` with their names, descriptions, and goals.

Some workflows may also be marked as **ephemeral**, meaning they were created as temporary workflows and can later be either saved or discarded.

Output as JSON or YAML:

```sh
stencila workflows list --as json
```

## Inspecting Workflows

View the full definition of a workflow:

```sh
stencila workflows show code-review
stencila workflows show code-review --as json
```

## Running Workflows

Run a workflow by name:

```sh
stencila workflows run code-review
```

### Goal Override

Override the workflow's goal for a single run:

```sh
stencila workflows run code-review --goal "Review the authentication module"
```

The `$goal` variable in all node prompts expands to this value instead of the goal defined in the WORKFLOW.md.

### Verbose Output

By default, workflow runs show a compact progress view with spinners. Use `--verbose` for detailed output showing agent names, full prompts, and full responses:

```sh
stencila workflows run code-review --verbose
```

### Dry Runs

Preview the workflow configuration and pipeline without executing it:

```sh
stencila workflows run code-review --dry-run
```

This shows the workflow name, description, goal, referenced agents, pipeline DOT source, and overrides — useful for verifying the workflow is configured correctly before a real run.

## Human-in-the-Loop

When a pipeline reaches a human review gate (`shape=human` or `shape=hexagon`), it pauses and presents choices derived from the node's outgoing edge labels. For example:

```dot
Review -> Publish [label="[A] Approve"]
Review -> Design  [label="[R] Revise"]
```

At the CLI, you select an option by typing the accelerator key (`A` or `R`). The pipeline then continues along the chosen edge. See [Pipelines — Human-in-the-loop](pipelines#human-in-the-loop) for details on accelerator key formats and auto-derivation.

## Workflow Lifecycle

A typical workflow lifecycle:

1. **Create** — `stencila workflows create my-workflow "Description"`
2. **Edit** — modify the WORKFLOW.md to define your pipeline and reference agents
3. **Validate** — `stencila workflows validate my-workflow`
4. **Dry run** — `stencila workflows run my-workflow --dry-run`
5. **Run** — `stencila workflows run my-workflow`
6. **Iterate** — adjust the pipeline, agents, or goal and re-run

## Managing Ephemeral Workflows

Ephemeral workflows behave like normal workflows when listing, showing, validating, and running them. The difference is lifecycle management: they are intended to be temporary until you explicitly save them.

Save an ephemeral workflow to keep it:

```sh
stencila workflows save my-workflow
```

Discard an ephemeral workflow to remove it entirely:

```sh
stencila workflows discard my-workflow
```

If a workflow is not ephemeral, these commands will fail rather than changing or deleting a permanent workflow.
