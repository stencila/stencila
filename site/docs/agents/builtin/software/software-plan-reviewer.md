---
title: "Software Plan Reviewer Agent"
description: "Reviews software delivery plans for quality, correctness, completeness, and feasibility"
keywords:
  - software plan
  - plan review
  - delivery plan
  - implementation plan
  - task breakdown
  - phasing
  - sequencing
  - testing strategy
  - TDD slices
  - risks
  - definition of done
  - critique
  - audit
  - software-plan-review
---

Reviews software delivery plans for quality, correctness, completeness, and feasibility

**Keywords:** software plan · plan review · delivery plan · implementation plan · task breakdown · phasing · sequencing · testing strategy · TDD slices · risks · definition of done · critique · audit · software-plan-review

> [!tip] Usage
>
> To use this agent, start your prompt with `#software-plan-reviewer` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when the user asks to review, audit, or critique a software delivery plan, implementation plan, or phased roadmap
- when the user wants feedback on task breakdown, sequencing, testing strategy, risks, or definition of done in an existing plan

# When not to use

- when the main task is to create a new delivery plan or draft an initial implementation plan
- when the main task is to write production code or review source code instead of evaluating a plan artifact
- when the main task is to review a design specification rather than a delivery plan

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Providers | `openai`, `anthropic`, `any` |
| Reasoning effort | `high` |
| Tools | `read_file`, `glob`, `grep` |
| Skills | [`software-plan-review`](/docs/skills/builtin/software/software-plan-review/) |

# Prompt

You are an assistant that specializes in reviewing software delivery plans for quality, correctness, completeness, and feasibility.

---

This page was generated from [`.stencila/agents/software-plan-reviewer/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-plan-reviewer/AGENT.md).
