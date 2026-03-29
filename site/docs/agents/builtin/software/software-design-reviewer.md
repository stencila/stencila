---
title: "Software Design Reviewer Agent"
description: "Reviews software design specifications for quality, correctness, completeness, feasibility, and architecture"
keywords:
  - software design
  - design review
  - design spec review
  - technical plan
  - technical specification
  - architecture review
  - requirements review
  - acceptance criteria
  - feasibility
  - critique
  - audit
  - software-design-review
---

Reviews software design specifications for quality, correctness, completeness, feasibility, and architecture

**Keywords:** software design · design review · design spec review · technical plan · technical specification · architecture review · requirements review · acceptance criteria · feasibility · critique · audit · software-design-review

# When to use

- when the user asks to review, audit, or critique a software design spec, technical plan, architecture proposal, or implementation plan
- when the user wants feedback on quality, correctness, clarity, completeness, feasibility, requirements, or acceptance criteria in an existing design document

# When not to use

- when the main task is to create a new software design spec or draft an initial technical plan
- when the main task is to write production code or review source code instead of evaluating a design artifact

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `high` |
| Tools | `read_file`, `glob`, `grep` |
| Skills | [`software-design-review`](/docs/skills/builtin/software/software-design-review/) |

# Prompt

You are an assistant that specializes in reviewing software design specifications.

---

This page was generated from [`.stencila/agents/software-design-reviewer/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-design-reviewer/AGENT.md).
