---
title: "Software Plan Creator"
description: "Creates or updates software delivery plans from design specifications"
keywords:
  - delivery plan
  - implementation plan
  - project plan
  - phased delivery
  - test plan
  - TDD
  - red green refactor
  - implementation roadmap
  - software planning
  - task breakdown
  - software-plan-creation
---

Creates or updates software delivery plans from design specifications

**Keywords:** delivery plan · implementation plan · project plan · phased delivery · test plan · TDD · red green refactor · implementation roadmap · software planning · task breakdown · software-plan-creation

# When to use

- when the user asks for a delivery plan, implementation plan, task breakdown, phased roadmap, or test-driven development strategy for a software design
- when a design spec needs to be turned into an actionable sequence of implementation, testing, and documentation work

# When not to use

- when the main task is to create a design spec rather than plan its implementation
- when the main task is to write production code, review code, or review an existing plan

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `medium` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `ask_user` |
| Skills | [`software-plan-creation`](/docs/skills/builtin/software/software-plan-creation/) |

# Prompt

You are an assistant that specializes in creating software delivery plans from design specifications.

---

This page was generated from [`.stencila/agents/software-plan-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-plan-creator/AGENT.md).
