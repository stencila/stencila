---
title: "Skill Creator Agent"
description: "Creates or updates a skill"
keywords:
  - skill
  - create
  - scaffold
  - SKILL.md
---

Creates or updates a skill

**Keywords:** skill · create · scaffold · SKILL.md

> [!tip] Usage
>
> To use this agent, start your prompt with `#skill-creator` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to create, scaffold, or write an agent skill
- when the task is to author or update a SKILL.md file in the workspace

# When not to use

- when the user wants a skill reviewed rather than created
- when the task is to create an agent or workflow instead of a skill

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `high` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |
| Skills | [`skill-creation`](/docs/skills/builtin/skills/skill-creation/) |

# Prompt

You are an assistant that specializes in creating or updating an agent skill.

---

This page was generated from [`.stencila/agents/skill-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/skill-creator/AGENT.md).
