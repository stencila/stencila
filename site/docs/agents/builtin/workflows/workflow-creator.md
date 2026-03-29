---
title: "Workflow Creator Agent"
description: "Creates or updates a workflow"
keywords:
  - workflow
  - create
  - scaffold
  - pipeline
---

Creates or updates a workflow

**Keywords:** workflow · create · scaffold · pipeline

> [!tip] Usage
>
> To use this agent, start your prompt with `#workflow-creator` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to create, scaffold, or set up a workflow
- when the task is to write or update a WORKFLOW.md file for a project

# When not to use

- when the user wants a workflow reviewed rather than created
- when the task is to route work instead of authoring a workflow

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `high` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user`, `list_agents`, `list_workflows` |
| Skills | [`workflow-creation`](/docs/skills/builtin/workflows/workflow-creation/) |

# Prompt

You are an assistant that specializes in creating or updating Stencila workflows.

---

This page was generated from [`.stencila/agents/workflow-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/workflow-creator/AGENT.md).
