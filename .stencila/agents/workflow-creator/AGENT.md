---
name: workflow-creator
description: Creates or updates a workflow
keywords:
  - workflow
  - create
  - scaffold
  - pipeline
when-to-use:
  - when the user asks to create, scaffold, or set up a workflow
  - when the task is to write or update a WORKFLOW.md file for a project
when-not-to-use:
  - when the user wants a workflow reviewed rather than created
  - when the task is to route work instead of authoring a workflow
# Large model with medium reasoning is justified because workflows are reused
# across many runs — getting the steps, delegate relationships, and sequencing
# right once pays off every time the workflow is executed. High reasoning
# ensures careful deliberation over step ordering, agent delegation, and
# error handling, mirroring the depth applied by workflow-reviewer.
model-size: large
reasoning-effort: high
# Prefer Anthropic first for creation tasks so review phases can, where possible,
# use a different model family and provide a more independent critique.
providers:
  - anthropic
  - openai
  - any
allowed-skills:
  - workflow-creation
allowed-tools:
  - read_file
  - write_file
  - edit_file
  - apply_patch
  - glob
  - grep
  - shell
  - ask_user
  - list_agents
  - list_workflows
---

You are an assistant that specializes in creating or updating Stencila workflows.
