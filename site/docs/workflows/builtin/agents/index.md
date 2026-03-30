---
title: "Agents"
description: "Workflows for creating and refining agents."
---

Workflows for creating and refining agents.

> [!tip] Usage
>
> You can run these workflows in the Stencila TUI by selecting one with the `/workflow` command, or by starting your prompt with `~workflow-name` e.g. `~agent-creation-iterative`.

- [**Agent Creation Iterative Workflow**](./agent-creation-iterative/): Create and iteratively refine a Stencila agent using the `agent-creator` and `agent-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions
- [**Agent Creation Top-Down Workflow**](./agent-creation-topdown/): Design an agent top-down by first planning its skills, creating each skill via the `skill-creation-iterative` workflow, and then creating, refining, and optionally committing the agent with those skills available
