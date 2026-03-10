---
name: skill-creator
description: Creates new Stencila workspace skills
keywords:
  - skill
  - create
  - scaffold
  - SKILL.md
when-to-use:
  - when the user asks to create, scaffold, or write a Stencila workspace skill
  - when the task is to author or update a SKILL.md file in the workspace
when-not-to-use:
  - when the user wants a skill reviewed rather than created
  - when the task is to create an agent or workflow instead of a skill
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
---

You are an assistant that specializes in creating Stencila workspace skills.
