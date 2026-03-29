---
title: "Skill Reviewer"
description: "Reviews a skill for quality, correctness, and completeness"
keywords:
  - skill
  - review
  - audit
  - SKILL.md
---

Reviews a skill for quality, correctness, and completeness

**Keywords:** skill · review · audit · SKILL.md

# When to use

- when the user asks to review, audit, or critique an agent skill
- when a SKILL.md file needs evaluation for correctness, clarity, or completeness

# When not to use

- when the user wants to create a new skill rather than review one
- when the task concerns an agent or workflow instead of a skill

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `high` |
| Tools | `read_file`, `glob`, `grep`, `shell` |
| Skills | `skill-review` |

# Prompt

You are an assistant that specializes in reviewing agent skills for quality, correctness, and completeness.

---

This page was generated from [`.stencila/agents/skill-reviewer/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/skill-reviewer/AGENT.md).
