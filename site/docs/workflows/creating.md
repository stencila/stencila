---
title: Creating Workflows
description: How to create multi-stage AI workflows with WORKFLOW.md files.
---

## Quick Start

Create a new workflow with the CLI:

```sh
stencila workflows create my-workflow "A multi-stage data pipeline"
```

This creates `.stencila/workflows/my-workflow/WORKFLOW.md` in your workspace with a template pipeline you can edit.

## Permanent vs Ephemeral Workflows

Most hand-authored workflows are **permanent** workflows that remain in `.stencila/workflows/` until you edit or delete them.

Stencila also supports **ephemeral** workflows for temporary or agent-created pipelines. An ephemeral workflow is stored in the same location as any other workflow, but its directory is marked with a `.gitignore` file containing `*`. This makes it easy to create a workflow, run it immediately, and either keep or discard it later.

Use ephemeral workflows when:

- an agent creates a workflow on your behalf
- you want to try a short-lived workflow before deciding to keep it
- you want a workflow that should not be committed or retained by default

See [Using Workflows](using#managing-ephemeral-workflows) for how to save or discard ephemeral workflows.

## The WORKFLOW.md File

A workflow is a directory containing a `WORKFLOW.md` file. The file has two parts:

1. **YAML frontmatter** — metadata (name, description, goal)
2. **Markdown body** — a DOT pipeline in a `` ```dot `` fenced code block, plus optional documentation

Here is a minimal example:

````markdown
---
name: lit-review
description: Search and summarize recent literature
---

```dot
digraph lit_review {
    graph [goal="Review recent literature on CRISPR gene editing"]

    Start -> Search -> Summarize -> Draft -> End

    Search    [prompt="Search for recent papers on: $goal"]
    Summarize [prompt="Summarize the key findings across the papers"]
    Draft     [prompt="Draft a literature review from the summaries"]
}
```
````

And a fully configured example with agents, branching, and human review:

````markdown
---
name: code-review
description: Automated code review with human approval gate
---

# Code Review Workflow

This workflow implements, tests, and reviews code changes.

```dot
digraph code_review {
    graph [goal="Implement and review the feature"]

    Start -> Design -> Build -> Test
    Test -> Review       [label="Pass", condition="outcome=success"]
    Test -> Build        [label="Fail", condition="outcome!=success"]
    Review -> End        [label="[A] Approve"]
    Review -> Design     [label="[R] Revise"]

    Design [agent="code-planner", prompt="Design the solution for: $goal"]
    Build  [agent="code-engineer", prompt="Implement the design"]
    Test   [agent="code-tester", prompt="Run tests and validate"]
    Review [shape=human]
}
```
````

Markdown content outside the `` ```dot `` block serves as human-readable documentation for the workflow. Only the first `` ```dot `` block is extracted as the pipeline definition.

## Workflow Names

Workflow names follow the same rules as [agent names](../agents/creating#agent-names) — **lowercase kebab-case**:

- 1–64 characters
- Only lowercase alphanumeric characters and hyphens
- No leading, trailing, or consecutive hyphens

By convention, names describe the workflow's purpose:

| Name | Purpose |
| ---- | ------- |
| `code-review` | Review code changes |
| `test-and-deploy` | Run tests then deploy |
| `lit-review` | Literature search and review |
| `plan-implement-validate` | Design, build, and validate |

The workflow's directory name must match the `name` field in the frontmatter.

## Directory Structure

Workflow definitions live in `.stencila/workflows/` in the workspace. Each workflow gets its own subdirectory:

```
.stencila/
  workflows/
    code-review/
      WORKFLOW.md
    test-and-deploy/
      WORKFLOW.md
    lit-review/
      WORKFLOW.md
```

An ephemeral workflow has the same structure, plus the temporary marker file:

```
.stencila/
  workflows/
    draft-review/
      .gitignore    # contains: *
      WORKFLOW.md
```

## Improving Discoverability and Delegation

When a manager agent decides which workflow to delegate to, it uses the workflow's `description`, `keywords`, `when-to-use`, and `when-not-to-use` fields. Adding these fields improves delegation accuracy.

```yaml
---
name: code-review
description: Automated code review with human approval gate
keywords:
  - code
  - review
  - testing
  - approval
when-to-use:
  - when the user wants an automated code review pipeline
  - when changes need testing and human approval before merging
when-not-to-use:
  - when the user wants a quick one-shot code review without a pipeline
  - when the task is about writing new code rather than reviewing it
---
```

Each `when-to-use` and `when-not-to-use` entry should be a short, specific sentence. Avoid vague signals like "when appropriate" — be concrete about the scenarios that match or don't match.

## Referencing Agents

Pipeline nodes reference [agents](../agents/) by name using the `agent` attribute:

```dot
Build [agent="code-engineer", prompt="Implement the design"]
Test  [agent="code-tester", prompt="Run tests and validate"]
```

The engine resolves agent names using the standard agent discovery order (workspace agents first, then user-level, then CLI-detected). This means:

- **Shared workflows** can be committed to a repository and used by the whole team
- **Personal agents** (in `~/.config/stencila/agents/`) let each user configure their preferred model, provider, and API keys
- The same `code-engineer` node runs with different backing models depending on who runs the workflow

When a node has no `agent` attribute, the engine uses a default agent. Explicit node attributes (like `agent.model` or `agent.provider`) override the agent's defaults.

You can also override specific agent properties inline using `agent.*` dotted-key attributes:

```dot
Build [agent="code-engineer", agent.provider="openai", agent.model="o3"]
Test  [agent="code-tester", agent.reasoning-effort="high"]
```

See [Pipelines — Agent property overrides](pipelines#agent-property-overrides) for details.

## Setting a Goal

The `goal` field provides a high-level objective for the pipeline. It is expanded as `$goal` in node prompts:

```yaml
---
name: data-analysis
description: Analyze and report on experimental data
goal: Analyze climate data from 2020-2024
---
```

The goal can also be set as a graph-level attribute in the DOT source:

```dot
digraph analysis {
    graph [goal="Analyze climate data from 2020-2024"]
    ...
}
```

When running a workflow, the goal can be overridden from the command line with `--goal`.

## Validation

Validate a workflow definition before running it:

```sh
# Validate by name
stencila workflows validate code-review

# Validate by path
stencila workflows validate .stencila/workflows/code-review/

# Validate a WORKFLOW.md file directly
stencila workflows validate .stencila/workflows/code-review/WORKFLOW.md
```

Validation checks:

- Name format (kebab-case, 1–64 characters)
- Name matches directory name
- Description is non-empty
- Pipeline DOT syntax is valid (if present)

## Next Steps

Once you have a workflow, see [Using Workflows](using) to run it, or dive into [Pipelines](pipelines) for the full pipeline syntax reference covering nodes, edges, conditions, parallel execution, and more.
