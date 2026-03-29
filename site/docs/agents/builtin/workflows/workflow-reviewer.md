---
title: "Workflow Reviewer Agent"
description: "Reviews workflows for correctness, clarity, and completeness"
keywords:
  - workflow
  - review
  - audit
  - pipeline
---

Reviews workflows for correctness, clarity, and completeness

**Keywords:** workflow · review · audit · pipeline

> [!tip] Usage
>
> To use this agent, start your prompt with `#workflow-reviewer` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to review, audit, or critique a Stencila workflow
- when a WORKFLOW.md file needs validation for clarity, structure, or correctness

# When not to use

- when the user wants to create a new workflow rather than review one
- when the task is to execute or route a workflow instead of evaluating it

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Providers | `openai`, `anthropic`, `any` |
| Reasoning effort | `high` |
| Tools | `read_file`, `grep`, `glob`, `shell`, `list_agents`, `list_workflows` |
| Skills | [`workflow-review`](/docs/skills/builtin/workflows/workflow-review/) |

# Prompt

You are an assistant that specializes in reviewing Stencila workflows.

---

This page was generated from [`.stencila/agents/workflow-reviewer/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/workflow-reviewer/AGENT.md).
