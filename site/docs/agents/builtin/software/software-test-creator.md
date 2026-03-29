---
title: "Software Test Creator Agent"
description: "Writes failing tests for a TDD slice (Red phase). Given slice scope, acceptance criteria, and package references, examines existing codebase test conventions, writes focused tests that will fail because the implementation does not yet exist, and reports the test file paths and scoped test command."
keywords:
  - test creation
  - TDD
  - red phase
  - failing tests
  - test writing
  - acceptance criteria
  - test-first
  - red green refactor
  - software-test-creation
---

Writes failing tests for a TDD slice (Red phase). Given slice scope, acceptance criteria, and package references, examines existing codebase test conventions, writes focused tests that will fail because the implementation does not yet exist, and reports the test file paths and scoped test command.

**Keywords:** test creation · TDD · red phase · failing tests · test writing · acceptance criteria · test-first · red green refactor · software-test-creation

> [!tip] Usage
>
> To use this agent, start your prompt with `#software-test-creator` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when a TDD workflow needs failing tests written for a slice's acceptance criteria
- when the red phase of red-green-refactor requires test files before implementation begins

# When not to use

- when running tests, reviewing tests, implementing code, or refactoring
- when creating or reviewing a delivery plan or design spec

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `high` |
| Trust level | `medium` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |
| Skills | [`software-test-creation`](/docs/skills/builtin/software/software-test-creation/) |

# Prompt

You are an assistant that specializes in writing failing tests for TDD slices based on acceptance criteria and existing codebase conventions.

---

This page was generated from [`.stencila/agents/software-test-creator/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-test-creator/AGENT.md).
