---
title: Builtin Workflows
description: Builtin workflows that ship with Stencila.
---

Builtin workflows ship with Stencila and are available in every workspace without additional configuration.

> [!tip] Usage
>
> You can run these workflows in the Stencila TUI by selecting one with the `/workflow` command, or by starting your prompt with `~workflow-name` e.g. `~software-delivery-tdd implement the feature described in DESIGN.md`.

## Themes

Workflows for creating and refining Stencila themes.

- [**Theme Creation Iterative Workflow**](./themes/theme-creation-iterative/) — Create and iteratively refine a Stencila theme using the `theme-creator` and `theme-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions

## Software Engineering

Workflows for the design, planning, implementation, testing, review, and delivery of software.

- [**Software Design Iterative Workflow**](./software/software-design-iterative/) — Create and iteratively refine a software design specification using the `software-design-creator` and `software-design-reviewer` agents, with human review, optional commit, and revision loops until accepted
- [**Software Plan Iterative Workflow**](./software/software-plan-iterative/) — Create and iteratively refine a software delivery plan using the `software-plan-creator` and `software-plan-reviewer` agents, with human review, optional commit, and revision loops until accepted
- [**Software Refactor Iterative Workflow**](./software/software-refactor-iterative/) — Iteratively refactor part or all of a software project using the `software-code-refactorer` and `software-code-reviewer` agents, with test verification after each pass, human approval, and optional commit before completion
- [**Software Delivery Test-Driven Development (TDD) Workflow**](./software/software-delivery-tdd/) — Execute a software delivery plan using test-driven development with Red-Green-Refactor cycles, agent-driven scoped test execution, human approval after each slice, and a bounded delivery closeout phase with final human review once all slices are complete
- [**Software Delivery Full Workflow**](./software/software-delivery-full/) — End-to-end software feature pipeline that composes design, planning, and TDD delivery workflows into a single orchestrated sequence
- [**Code Review Parallel Workflow**](./software/code-review-parallel/) — Run three independent code reviews in parallel using Anthropic, OpenAI, and Google models via the `software-code-reviewer` agent, then synthesize a unified prioritized findings report highlighting reviewer agreement and disagreement

## Workflows

Workflows for creating, refining, and dynamically running workflows.

- [**Workflow Creation Iterative Workflow**](./workflows/workflow-creation-iterative/) — Create and iteratively refine a Stencila workflow using the `workflow-creator` and `workflow-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions
- [**Workflow Creation Top-Down Workflow**](./workflows/workflow-creation-topdown/) — Design a workflow top-down by first planning its structure and dependencies, creating each dependency via the appropriate child workflow, then building and refining the parent workflow with those dependencies available
- [**Workflow Create and Run Workflow**](./workflows/workflow-create-run/) — Generate an ephemeral workflow tailored to a user's goal and immediately execute it, enabling single-delegation dynamic workflow orchestration

## Agents

Workflows for creating and refining agents.

- [**Agent Creation Iterative Workflow**](./agents/agent-creation-iterative/) — Create and iteratively refine a Stencila agent using the `agent-creator` and `agent-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions
- [**Agent Creation Top-Down Workflow**](./agents/agent-creation-topdown/) — Design an agent top-down by first planning its skills, creating each skill via the `skill-creation-iterative` workflow, and then creating, refining, and optionally committing the agent with those skills available

## Skills

Workflows for creating and refining agent skills.

- [**Skill Creation Iterative Workflow**](./skills/skill-creation-iterative/) — Create and iteratively refine a Stencila skill using the `skill-creator` and `skill-reviewer` agents, with human review, optional commit, and revision loops until accepted
