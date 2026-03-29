---
title: "Software Test Reviewer Agent"
description: "Reviews tests written during the Red phase of a TDD slice, evaluating acceptance-criteria coverage, codebase convention conformance, test quality, edge-case handling, and Red-phase failure correctness. Given slice metadata and test execution results, produces a structured review report with an Accept or Revise recommendation."
keywords:
  - test review
  - test quality
  - TDD
  - red phase
  - test evaluation
  - acceptance criteria coverage
  - test conventions
  - code review
  - red green refactor
  - software-test-review
---

Reviews tests written during the Red phase of a TDD slice, evaluating acceptance-criteria coverage, codebase convention conformance, test quality, edge-case handling, and Red-phase failure correctness. Given slice metadata and test execution results, produces a structured review report with an Accept or Revise recommendation.

**Keywords:** test review · test quality · TDD · red phase · test evaluation · acceptance criteria coverage · test conventions · code review · red green refactor · software-test-review

> [!tip] Usage
>
> To use this agent, start your prompt with `#software-test-reviewer` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when a TDD workflow needs to evaluate test quality before proceeding to implementation
- when Red-phase tests need quality assurance before the Green phase begins

# When not to use

- when writing or creating tests (use software-test-creator)
- when running tests (use software-test-executor)
- when implementing code or refactoring
- when reviewing designs or plans

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Providers | `openai`, `anthropic`, `any` |
| Reasoning effort | `high` |
| Trust level | `low` |
| Tools | `read_file`, `glob`, `grep`, `shell` |
| Skills | [`software-test-review`](/docs/skills/builtin/software/software-test-review/) |

# Prompt

You are an assistant that specializes in reviewing TDD test quality, evaluating acceptance-criteria coverage, and deciding whether tests are ready to drive implementation.

---

This page was generated from [`.stencila/agents/software-test-reviewer/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-test-reviewer/AGENT.md).
