---
title: "Software Engineering"
description: "Agents for the design, planning, implementation, testing, review, and delivery of software."
---

Agents for the design, planning, implementation, testing, review, and delivery of software.

> [!tip] Usage
>
> You can use these agents in the Stencila TUI by selecting one with the `/agent` command, or by starting your prompt with `#agent-name` e.g. `#software-design-creator`. Agents can also be referenced by name in workflow node definitions.

- [**Software Design Creator Agent**](./software-design-creator/): Creates or updates software design specifications
- [**Software Design Reviewer Agent**](./software-design-reviewer/): Reviews software design specifications for quality, correctness, completeness, feasibility, and architecture
- [**Software Plan Creator Agent**](./software-plan-creator/): Creates or updates software delivery plans from design specifications
- [**Software Plan Reviewer Agent**](./software-plan-reviewer/): Reviews software delivery plans for quality, correctness, completeness, and feasibility
- [**Software Slice Selector Agent**](./software-slice-selector/): Reads a software delivery plan, marks the just-completed slice or slice batch (if any), updates the completed slices list, selects the next unfinished execution unit based on phase ordering and dependency constraints, and reports whether more slices remain. Combines slice completion tracking with next-work selection in a single step and may normalize overly narrow plans by combining adjacent compatible slices.
- [**Software Test Creator Agent**](./software-test-creator/): Writes failing tests for a TDD slice (Red phase). Given slice scope, acceptance criteria, and package references, examines existing codebase test conventions, writes focused tests that will fail because the implementation does not yet exist, and reports the test file paths and scoped test command.
- [**Software Test Reviewer Agent**](./software-test-reviewer/): Reviews tests written during the Red phase of a TDD slice, evaluating acceptance-criteria coverage, codebase convention conformance, test quality, edge-case handling, and Red-phase failure correctness. Given slice metadata and test execution results, produces a structured review report with an Accept or Revise recommendation.
- [**Software Test Executor Agent**](./software-test-executor/): Executes scoped tests for a TDD slice and reports structured pass/fail results. Given the test command, test files, and slice scope, discovers the test framework if no command is provided, runs only the tests relevant to the current slice, parses output, and reports a structured pass/fail result.
- [**Software Implementor Agent**](./software-implementor/): Implements the minimum production code necessary to make failing tests pass (Green phase of TDD). Given slice scope, acceptance criteria, target packages, and test file references, examines failing test output, discovers codebase conventions, and writes focused implementation code that satisfies test expectations without over-engineering. Handles iterative feedback from failed test runs.
- [**Software Code Reviewer Agent**](./software-code-reviewer/): Reviews source code for correctness, quality, security, style, and maintainability. Discovers codebase conventions, analyzes code against them, and produces a structured review report with prioritized findings and actionable recommendations. Works with any language or framework.
- [**Software Code Refactorer Agent**](./software-code-refactorer/): Refactors production code to improve quality while keeping all tests passing. Discovers codebase conventions, applies safe transformations (duplication removal, naming improvements, complexity reduction, convention alignment), and verifies the code still compiles and all tests pass. Commonly used for the Refactor phase of TDD, but works equally well as a standalone code-quality improvement pass on any codebase with tests. Handles iterative feedback from failed test runs.
- [**Software Delivery Completer Agent**](./software-delivery-completer/): Verifies plan-level Definition of Done and completion criteria after all execution slices are finished, performs bounded minor closeout work (formatting, lint, generated files, small documentation or glue fixes), runs final verification commands, and produces a structured completion report. Reports clearly when substantial unfinished work remains rather than beginning a new implementation cycle. Used as the final delivery stage after slice-by-slice TDD execution.
