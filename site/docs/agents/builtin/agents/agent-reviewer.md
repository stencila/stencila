---
title: "Agent Reviewer Agent"
description: "Reviews an agent for quality, correctness, and completeness"
keywords:
  - agent
  - review
  - audit
  - AGENT.md
---

Reviews an agent for quality, correctness, and completeness

**Keywords:** agent · review · audit · AGENT.md

# When to use

- when the user asks to review, audit, or critique a Stencila agent
- when an AGENT.md file needs evaluation for correctness, clarity, or completeness

# When not to use

- when the user wants to create a new agent rather than review one
- when the task concerns a skill or workflow instead of an agent definition

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `medium` |
| Reasoning effort | `high` |
| Tools | `read_file`, `glob`, `grep`, `shell`, `ask_user` |
| Skills | [`agent-review`](/docs/skills/builtin/agents/agent-review/) |

# Prompt

You are an assistant that specializes in reviewing Stencila agents for quality, correctness, and completeness.

---

This page was generated from [`.stencila/agents/agent-reviewer/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/agent-reviewer/AGENT.md).
