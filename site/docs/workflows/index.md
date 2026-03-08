---
title: Workflows
description: Documentation for Stencila's multi-stage AI workflows.
---

Workflows orchestrate multi-stage AI tasks as directed graphs. Where an [agent](../agents/) handles a single task — reading files, running commands, reasoning about code — a workflow chains multiple agents together into a pipeline with branching, retries, parallel execution, and human-in-the-loop gates.

A workflow is a directory containing a `WORKFLOW.md` file that defines the pipeline using [Graphviz DOT syntax](https://graphviz.org/doc/info/lang.html). Each node in the graph is a stage — an LLM call, a shell command, a human review gate, or a conditional branch — and edges define the flow between them.

## How Workflows Work

A workflow pipeline is a directed acyclic graph (with optional retry loops). The execution engine walks the graph from `Start` to `End`, running each node's handler and selecting the next edge based on conditions, outcomes, and edge weights.

```
Start → Search → Screen → Analyze → CheckQuality
                                        ↓ Pass       ↓ Fail
                                      Review       Analyze (retry)
                                        ↓ Approve
                                      Publish → End
```

Each node can:

- **Call an agent** — run a Stencila agent with a prompt, receiving the agent's full tool-calling capabilities
- **Execute a shell command** — run a build, test suite, or script directly
- **Pause for human input** — present choices and wait for a decision
- **Route conditionally** — branch based on the outcome of the previous stage

Nodes communicate through a shared key-value context. After each node completes, its outcome and any context updates are available to subsequent nodes via prompt variable expansion and edge conditions.

## Workflows vs Agents

| | Agent | Workflow |
| --- | --- | --- |
| **Scope** | Single task | Multi-stage pipeline |
| **Definition** | `AGENT.md` | `WORKFLOW.md` with DOT pipeline |
| **Execution** | Agentic loop (model → tools → model → …) | Graph traversal (node → edge → node → …) |
| **Location** | `.stencila/agents/` or `~/.config/stencila/agents/` | `.stencila/workflows/` |
| **Reuse** | Referenced by name from workflow nodes | Composes agents into larger processes |

Workflows reference agents by name. A node like `Build [agent="code-engineer", prompt="Implement the design"]` tells the engine to run the `code-engineer` agent for that stage. This separation means a shared workflow committed to a repository can be executed with different agent configurations by different users — one user might have a `code-engineer` agent backed by Anthropic, another by OpenAI.

## Workflow Discovery

Workflows are discovered from `.stencila/workflows/` in the workspace:

```
.stencila/
  workflows/
    code-review/
      WORKFLOW.md
    test-and-deploy/
      WORKFLOW.md
```

Each workflow gets its own subdirectory. The directory name must match the `name` field in the WORKFLOW.md frontmatter.

## Next Steps

- [Creating Workflows](creating) — create and configure your own workflows
- [Using Workflows](using) — run workflows from the CLI
- [Configuration Reference](configuration) — full reference for `WORKFLOW.md` properties
- [Pipelines](pipelines) — detailed pipeline syntax: nodes, edges, conditions, patterns, and graph attributes
