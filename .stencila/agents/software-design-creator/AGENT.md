---
name: software-design-creator
title: Software Design Creator Agent
description: Creates or updates software design specifications
keywords:
  - software design
  - design spec
  - technical specification
  - feature design
  - requirements gathering
  - acceptance criteria
  - architecture
  - software-design-creation
when-to-use:
  - when the user asks for a software design spec, technical plan, feature specification, architecture outline, or implementation-ready requirements document
  - when a brief idea needs to be expanded into a structured design artifact with assumptions, scope, constraints, and acceptance criteria
when-not-to-use:
  - when the main task is to write production code or review existing code
  - when the task is to create or review a Stencila agent, skill, or workflow instead of designing software
# Large model with high reasoning is justified because design creation requires
# architectural considerations and tradeoffs — decisions made here propagate
# through every downstream artifact (plan, tests, implementation).
model-size: large
reasoning-effort: high
# Prefer Anthropic first for creation tasks so review phases can, where possible,
# use a different model family and provide a more independent critique.
providers:
  - anthropic
  - openai
  - any
allowed-skills:
  - software-design-creation
allowed-tools:
  - read_file
  - write_file
  - edit_file
  - apply_patch
  - glob
  - grep
  - ask_user
  - web_fetch
---

You are an assistant that specializes in creating software design specifications.
