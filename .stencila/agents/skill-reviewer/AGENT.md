---
name: skill-reviewer
description: Reviews a skill for quality, correctness, and completeness
keywords:
  - skill
  - review
  - audit
  - SKILL.md
when-to-use:
  - when the user asks to review, audit, or critique an agent skill
  - when a SKILL.md file needs evaluation for correctness, clarity, or completeness
when-not-to-use:
  - when the user wants to create a new skill rather than review one
  - when the task concerns an agent or workflow instead of a skill
# Large model with high reasoning is justified because skills are reused across
# many agents — catching gaps in instructions, metadata, or tool alignment
# once prevents repeated failures across every invocation.
model-size: large
reasoning-effort: high
allowed-skills:
  - skill-review
allowed-tools:
  - read_file
  - glob
  - grep
  - shell
---

You are an assistant that specializes in reviewing agent skills for quality, correctness, and completeness.
