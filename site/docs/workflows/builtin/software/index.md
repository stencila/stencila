---
title: "Software Engineering"
description: "Workflows for the design, planning, implementation, testing, review, and delivery of software."
---

Workflows for the design, planning, implementation, testing, review, and delivery of software.

- [**Software Design Iterative Workflow**](./software-design-iterative/): Create and iteratively refine a software design specification using the `software-design-creator` and `software-design-reviewer` agents, with human review, optional commit, and revision loops until accepted
- [**Software Plan Iterative Workflow**](./software-plan-iterative/): Create and iteratively refine a software delivery plan using the `software-plan-creator` and `software-plan-reviewer` agents, with human review, optional commit, and revision loops until accepted
- [**Software Refactor Iterative Workflow**](./software-refactor-iterative/): Iteratively refactor part or all of a software project using the `software-code-refactorer` and `software-code-reviewer` agents, with test verification after each pass, human approval, and optional commit before completion
- [**Software Delivery Test-Driven Development (TDD) Workflow**](./software-delivery-tdd/): Execute a software delivery plan using test-driven development with Red-Green-Refactor cycles, agent-driven scoped test execution, human approval after each slice, and a bounded delivery closeout phase with final human review once all slices are complete
- [**Software Delivery Full Workflow**](./software-delivery-full/): End-to-end software feature pipeline that composes design, planning, and TDD delivery workflows into a single orchestrated sequence
- [**Code Review Parallel Workflow**](./code-review-parallel/): Run three independent code reviews in parallel using Anthropic, OpenAI, and Google models via the `software-code-reviewer` agent, then synthesize a unified prioritized findings report highlighting reviewer agreement and disagreement
