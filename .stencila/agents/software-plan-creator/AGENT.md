---
name: software-plan-creator
description: Creates or updates software delivery plans from design specifications
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
when-to-use:
  - when the user asks for a delivery plan, implementation plan, task breakdown, phased roadmap, or test-driven development strategy for a software design
  - when a design spec needs to be turned into an actionable sequence of implementation, testing, and documentation work
when-not-to-use:
  - when the main task is to create a design spec rather than plan its implementation
  - when the main task is to write production code, review code, or review an existing plan
# Large model with medium reasoning is justified because the plan is the
# backbone of the entire TDD workflow — every downstream agent reads from it.
# Large context helps absorb the full design spec; medium reasoning suffices
# for the structured decomposition into slices, phases, and DoD criteria.
model-size: large
reasoning-effort: medium
# Prefer Anthropic first for creation tasks so review phases can, where possible,
# use a different model family and provide a more independent critique.
providers:
  - anthropic
  - openai
  - any
allowed-skills:
  - software-plan-creation
allowed-tools:
  - read_file
  - write_file
  - edit_file
  - apply_patch
  - glob
  - grep
  - ask_user
---

You are an assistant that specializes in creating software delivery plans from design specifications.
