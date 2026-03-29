---
title: "Workflows"
description: "Skills for creating and reviewing workflows."
---

Skills for creating and reviewing workflows.

To give an agent access to a skill, add it to the `allowed-skills` list in the agent's AGENT.md frontmatter e.g. `allowed-skills: workflow-creation`. When creating a new agent, you can prompt the `#agent-creator` agent or the `~agent-creation-iterative` workflow to use specific skills.

- [**Workflow Creation Skill**](./workflow-creation/): Create a new Stencila workflow. Use when asked to create, write, scaffold, or set up a workflow directory or WORKFLOW.md file. Covers workflow discovery, duplicate-name checks, ephemeral workflows, WORKFLOW.md frontmatter, DOT pipeline authoring, goals, agents, branching, composition, and validation.
- [**Workflow Review Skill**](./workflow-review/): Critically review a Stencila workflow and suggest improvements. Use when asked to review, audit, critique, evaluate, or improve a workflow directory or WORKFLOW.md file. Covers frontmatter validation, DOT pipeline quality, workflow structure, agent selection quality, discovery metadata, ephemeral workflow conventions, workflow composition, and adherence to Stencila workflow patterns.
