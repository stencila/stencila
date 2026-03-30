---
title: "Agents"
description: "Skills for creating and reviewing agents."
---

Skills for creating and reviewing agents.

> [!tip] Usage
>
> To give an agent access to a skill, add it to the `allowed-skills` list in the agent's AGENT.md frontmatter e.g. `allowed-skills: agent-creation`. When creating a new agent, you can prompt the `#agent-creator` agent or the `~agent-creation-iterative` workflow to use specific skills.

- [**Agent Creation Skill**](./agent-creation/): Create a new Stencila agent. Use when asked to create, write, scaffold, or set up an agent directory or AGENT.md file. Covers workspace and user-level agents with model, provider, tool, trust, and MCP configuration.
- [**Agent Review Skill**](./agent-review/): Critically review a Stencila agent and suggest improvements. Use when asked to review, audit, critique, evaluate, or improve an agent directory or AGENT.md file. Covers frontmatter validation, system instruction quality, configuration correctness, and adherence to the Agent schema.
