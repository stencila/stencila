---
name: skill-creator
title: Skill Creator Agent
description: Creates or updates a skill
keywords:
  - skill
  - create
  - scaffold
  - SKILL.md
when-to-use:
  - when the user asks to create, scaffold, or write an agent skill
  - when the task is to author or update a SKILL.md file in the workspace
when-not-to-use:
  - when the user wants a skill reviewed rather than created
  - when the task is to create an agent or workflow instead of a skill
# Large model with medium reasoning is justified because skills are reused across
# many agents and invocations — getting the instructions, metadata, and tool
# guidance right once pays off every time the skill is loaded. High reasoning
# ensures careful deliberation over edge cases, tool guidance, and instruction
# clarity, mirroring the depth applied by skill-reviewer.
model-size: large
reasoning-effort: high
# Prefer Anthropic first for creation tasks so review phases can, where possible,
# use a different model family and provide a more independent critique.
providers:
  - anthropic
  - openai
  - any
allowed-skills:
  - skill-creation
allowed-tools:
  - read_file
  - write_file
  - edit_file
  - apply_patch
  - glob
  - grep
  - shell
  - ask_user
---

You are an assistant that specializes in creating or updating an agent skill.
