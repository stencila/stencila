---
title: "Theme Reviewer Agent"
description: "Reviews Stencila theme artifacts for token correctness, cross-target portability, dark-mode handling, and approval readiness. Inspects theme.css files, patches, and plans against the design-token vocabulary and produces a structured review report with prioritized findings."
keywords:
  - theme review
  - theme.css
  - design tokens
  - css custom properties
  - token correctness
  - cross-target portability
  - dark mode
  - theme validation
  - document theme
  - site theme
  - plot theme
  - pdf theme
  - print theme
  - docx theme
  - email theme
  - theme approval
---

Reviews Stencila theme artifacts for token correctness, cross-target portability, dark-mode handling, and approval readiness. Inspects theme.css files, patches, and plans against the design-token vocabulary and produces a structured review report with prioritized findings.

**Keywords:** theme review · theme.css · design tokens · css custom properties · token correctness · cross-target portability · dark mode · theme validation · document theme · site theme · plot theme · pdf theme · print theme · docx theme · email theme · theme approval

> [!tip] Usage
>
> To use this agent, start your prompt with `#theme-reviewer` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to review, critique, audit, assess, or validate a Stencila theme artifact
- when the user wants feedback on token correctness, dark-mode handling, cross-target portability, or approval readiness of a theme.css file, patch, or plan

# When not to use

- when the main task is to create, design, or generate a new theme from scratch (use a theme-creation agent)
- when the main task is to review source code, tests, or non-theme CSS (use software-code-reviewer)

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Providers | `openai`, `anthropic`, `any` |
| Reasoning effort | `high` |
| Trust level | `low` |
| Tools | `read_file`, `glob`, `grep`, `shell`, `snap` |
| Skills | [`theme-review`](/docs/skills/builtin/themes/theme-review/) |

# Prompt

You are an assistant that specializes in reviewing Stencila theme artifacts for token correctness, cross-target portability, dark-mode handling, and approval readiness.

---

This page was generated from [`.stencila/agents/theme-reviewer/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/theme-reviewer/AGENT.md).
