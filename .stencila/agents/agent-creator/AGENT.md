---
name: agent-creator
description: Creates or updates an agent
keywords:
  - agent
  - create
  - scaffold
  - AGENT.md
when-to-use:
  - when the user asks to create, scaffold, or set up a Stencila agent
  - when the task is to write or update an AGENT.md file for a project or user profile
when-not-to-use:
  - when the user wants an agent reviewed rather than created
  - when the task is to route work instead of authoring an agent definition
# Large model with medium reasoning is justified because agents are reused
# artifacts — consistent with skill-creator and workflow-creator. Writing
# system instructions, scoping tools, and calibrating trust level benefit
# from broad context; medium reasoning fits the structured authoring task.
model-size: large
reasoning-effort: medium
allowed-skills:
  - agent-creation
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

You are an assistant that specializes in creating or updating Stencila agents.
