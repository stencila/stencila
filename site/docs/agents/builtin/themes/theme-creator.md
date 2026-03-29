---
title: "Theme Creator Agent"
description: "Creates or updates Stencila theme CSS files using semantic tokens, module tokens, and the theme CLI"
keywords:
  - theme
  - theme.css
  - css
  - design system
  - tokens
  - semantic tokens
  - document theme
  - site theme
  - plot tokens
  - print tokens
  - branding
  - styling
  - dark mode
  - fonts
---

Creates or updates Stencila theme CSS files using semantic tokens, module tokens, and the theme CLI

**Keywords:** theme · theme.css · css · design system · tokens · semantic tokens · document theme · site theme · plot tokens · print tokens · branding · styling · dark mode · fonts

> [!tip] Usage
>
> To use this agent, start your prompt with `#theme-creator` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to create, update, or patch a theme.css file
- when the task involves choosing or customizing Stencila design tokens
- when the user wants to style documents, sites, plots, or PDF output
- when the user needs help with theme planning, direction, or token selection

# When not to use

- when the user wants to review an existing theme rather than create or modify one
- when the task is about general CSS unrelated to Stencila themes or tokens
- when the user needs to implement application logic rather than visual styling

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `high` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `snap`, `ask_user` |
| Skills | [`theme-creation`](/docs/skills/builtin/themes/theme-creation/) |

# Prompt

You are an assistant that specializes in creating or updating Stencila theme CSS files using semantic tokens, module tokens, and the theme CLI.

---

This page was generated from [`.stencila/agents/theme-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/theme-creator/AGENT.md).
