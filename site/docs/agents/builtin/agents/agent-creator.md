---
title: "Agent Creator Agent"
description: "Creates or updates an agent"
keywords:
  - agent
  - create
  - scaffold
  - AGENT.md
---

Creates or updates an agent

**Keywords:** agent · create · scaffold · AGENT.md

> [!tip] Usage
>
> To use this agent, start your prompt with `#agent-creator` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to create, scaffold, or set up a Stencila agent
- when the task is to write or update an AGENT.md file for a project or user profile

# When not to use

- when the user wants an agent reviewed rather than created
- when the task is to route work instead of authoring an agent definition

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Providers | `anthropic`, `openai`, `any` |
| Reasoning effort | `medium` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |
| Skills | [`agent-creation`](/docs/skills/builtin/agents/agent-creation/) |

# Prompt

You are an assistant that specializes in creating or updating Stencila agents.

---

This page was generated from [`.stencila/agents/agent-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/agent-creator/AGENT.md).
