---
title: "Software Implementor Agent"
description: "Implements the minimum production code necessary to make failing tests pass (Green phase of TDD). Given slice scope, acceptance criteria, target packages, and test file references, examines failing test output, discovers codebase conventions, and writes focused implementation code that satisfies test expectations without over-engineering. Handles iterative feedback from failed test runs."
keywords:
  - implementation
  - green phase
  - TDD
  - production code
  - make tests pass
  - minimal implementation
  - red green refactor
  - software-implementation
---

Implements the minimum production code necessary to make failing tests pass (Green phase of TDD). Given slice scope, acceptance criteria, target packages, and test file references, examines failing test output, discovers codebase conventions, and writes focused implementation code that satisfies test expectations without over-engineering. Handles iterative feedback from failed test runs.

**Keywords:** implementation · green phase · TDD · production code · make tests pass · minimal implementation · red green refactor · software-implementation

> [!tip] Usage
>
> To use this agent, start your prompt with `#software-implementor` in the Stencila TUI, or select it with the `/agent` command. You can also reference it by name in a workflow pipeline.

# When to use

- when a TDD workflow needs production code written to make failing tests pass
- when the green phase of red-green-refactor requires minimal implementation code

# When not to use

- when writing or creating tests (use software-test-creator)
- when running tests (use software-test-executor)
- when refactoring existing code (use software-code-refactorer)
- when reviewing tests, designs, or plans

# Configuration

| Property | Value |
| -------- | ----- |
| Model | `large` |
| Reasoning effort | `medium` |
| Trust level | `medium` |
| Tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |
| Skills | [`software-implementation`](/docs/skills/builtin/software/software-implementation/) |

# Prompt

You are an assistant that specializes in writing the minimal production code needed to make failing TDD tests pass, following existing codebase conventions.

---

This page was generated from [`.stencila/agents/software-implementor/AGENT.md`](https://github.com/stencila/stencila/blob/main/.stencila/agents/software-implementor/AGENT.md).
