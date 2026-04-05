---
title: "Figure Creator Agent"
description: "Creates or updates figures in Stencila Markdown documents, including simple image figures, executable figures, multi-panel layouts, captions, and SVG annotation overlays"
keywords:
  - figure
  - figure creation
  - image figure
  - executable figure
  - chart
  - plot
  - caption
  - subfigure
  - multi-panel
  - grid layout
  - overlay
  - annotation
  - callout
  - arrow
  - scale bar
  - SVG
  - stencila markdown
  - smd
---

Creates or updates figures in Stencila Markdown documents, including simple image figures, executable figures, multi-panel layouts, captions, and SVG annotation overlays

**Keywords:** figure · figure creation · image figure · executable figure · chart · plot · caption · subfigure · multi-panel · grid layout · overlay · annotation · callout · arrow · scale bar · SVG · stencila markdown · smd

> [!tip] Usage
>
> To use this agent, start your prompt with `#figure-creator` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to create, insert, update, or redesign a figure in a Stencila Markdown document
- when the task involves figure captions, subfigure layouts, executable figures, or SVG overlay annotations
- when the user wants a figure plan or figure specification before editing a document

# When not to use

- when the user wants a figure reviewed or audited without modifying it
- when the task is general image editing outside Stencila Markdown figure authoring
- when the user needs broader document writing rather than figure-focused work

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Providers | `anthropic`, `openai`, `any` |
| Reasoning effort | `high` |
| Tools | `read_file`, `write_file`, `apply_patch`, `glob`, `grep`, `snap`, `ask_user` |
| Skills | [`figure-creation`](/docs/skills/builtin/documents/figure-creation/) |

# Prompt

You are an assistant that specializes in creating or updating figures in Stencila Markdown documents using figure syntax, subfigure layouts, executable outputs, and SVG overlay annotations.

---

This page was generated from [`.stencila/agents/figure-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/figure-creator/AGENT.md).
