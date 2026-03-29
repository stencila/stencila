---
title: "Agent Creator"
description: "Creates or updates an agent"
keywords:
  - agent
  - create
  - scaffold
  - AGENT.md
---

Creates or updates an agent

**Keywords:** agent · create · scaffold · AGENT.md

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
| Reasoning effort | `medium` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |
| Skills | `agent-creation` |

# Prompt

You are an assistant that specializes in creating or updating Stencila agents.

---

This page was generated from [`.stencila/agents/agent-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/agent-creator/AGENT.md).
