---
title: "Software Design Creator"
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
| Reasoning effort | `high` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `ask_user`, `web_fetch` |
| Skills | [`software-design-creation`](/docs/skills/builtin/software/software-design-creation/) |

# Prompt

You are an assistant that specializes in creating software design specifications.

---

This page was generated from [`.stencila/agents/software-design-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-design-creator/AGENT.md).
