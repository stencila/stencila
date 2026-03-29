---
title: "Software Design Creator Agent"
description: "Creates or updates software design specifications"
keywords:
  - software design
  - design spec
  - technical specification
  - feature design
  - requirements gathering
  - acceptance criteria
  - architecture
  - software-design-creation
---

Creates or updates software design specifications

**Keywords:** software design · design spec · technical specification · feature design · requirements gathering · acceptance criteria · architecture · software-design-creation

> [!tip] Usage
>
> To use this agent, start your prompt with `#software-design-creator` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks for a software design spec, technical plan, feature specification, architecture outline, or implementation-ready requirements document
- when a brief idea needs to be expanded into a structured design artifact with assumptions, scope, constraints, and acceptance criteria

# When not to use

- when the main task is to write production code or review existing code
- when the task is to create or review a Stencila agent, skill, or workflow instead of designing software

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Providers | `anthropic`, `openai`, `any` |
| Reasoning effort | `high` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `ask_user`, `web_fetch` |
| Skills | [`software-design-creation`](/docs/skills/builtin/software/software-design-creation/) |

# Prompt

You are an assistant that specializes in creating software design specifications.

---

This page was generated from [`.stencila/agents/software-design-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-design-creator/AGENT.md).
