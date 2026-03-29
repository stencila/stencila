---
title: "Software Implementation Skill"
description: "Write the minimal production code needed to make failing TDD tests pass (Green phase of red-green-refactor). Use when failing tests exist and production code must be written or modified to satisfy them. Reads and interprets failing test output, discovers codebase conventions (module layout, naming, import patterns, error handling, coding style), writes focused production code that satisfies test expectations without over-engineering, integrates new code with existing modules, types, and APIs, handles revision feedback from failed test runs, and verifies implementation compiles or parses before handing off to test execution. Works with any language or framework."
keywords:
  - implementation
  - production code
  - green phase
  - TDD
  - red green refactor
  - make tests pass
  - write code
  - code writing
  - satisfy tests
  - fix failing tests
  - minimal implementation
  - codebase conventions
  - module layout
  - coding style
  - integrate code
  - compile check
  - parse check
  - not test creation
  - not test writing
  - not refactoring
  - not test execution
---

Write the minimal production code needed to make failing TDD tests pass (Green phase of red-green-refactor). Use when failing tests exist and production code must be written or modified to satisfy them. Reads and interprets failing test output, discovers codebase conventions (module layout, naming, import patterns, error handling, coding style), writes focused production code that satisfies test expectations without over-engineering, integrates new code with existing modules, types, and APIs, handles revision feedback from failed test runs, and verifies implementation compiles or parses before handing off to test execution. Works with any language or framework.

**Keywords:** implementation · production code · green phase · TDD · red green refactor · make tests pass · write code · code writing · satisfy tests · fix failing tests · minimal implementation · codebase conventions · module layout · coding style · integrate code · compile check · parse check · not test creation · not test writing · not refactoring · not test execution

> [!tip] Usage
>
> To use this skill, add `software-implementation` to the `allowed-skills` list in your agent's AGENT.md. You can also ask `#agent-creator` to build an agent that uses it.

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |

# Instructions

## Overview

Write the minimal production code needed to make failing TDD tests pass. This skill handles the "green" phase of a red-green-refactor cycle: given failing test output, test file paths, and slice context, it discovers the codebase's conventions, writes or modifies production code so the tests pass, and verifies the code compiles or parses cleanly.

The core principles are:

- **Discover first, prescribe only as fallback.** Adapt to whatever language, framework, and conventions the codebase already uses.
- **Minimal and sufficient.** Write exactly the code needed to make the failing tests pass — no more, no less. Do not add features, optimizations, or abstractions beyond what the tests require.
- **Tests are the specification.** The failing test output defines what the code must do. Do not second-guess the tests or add untested behavior.

The implementation must:

- Make all specified failing tests pass
- Follow the codebase's existing conventions (naming, module layout, error handling, coding style)
- Compile or parse without errors or warnings where possible
- Integrate cleanly with existing modules, types, and APIs
- Not break any existing tests

## Required Inputs

This skill requires the following information to operate:

| Input                | Required | Description                                                             |
|----------------------|----------|-------------------------------------------------------------------------|
| Test failure output  | Yes      | The output from the failing test run (compiler errors, assertion failures, import errors) |
| Test files           | Yes      | Paths to the test files that need to pass                               |
| Test command         | Yes      | The command to run the scoped tests                                     |
| Slice scope          | No       | Description of what the slice covers (provides context for design decisions) |
| Acceptance criteria  | No       | The criteria the tests verify (provides context for intent)             |
| Target packages      | No       | Packages, crates, modules, or directories involved                      |
| Slice name           | No       | Name or identifier of the current slice                                 |
| Revision feedback    | No       | Feedback from a test run, reviewer, or human on a previous attempt      |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

After completing its work, this skill reports:

| Output               | Description                                                            |
|----------------------|------------------------------------------------------------------------|
| Implementation files | The list of production code file paths created or modified             |
| Implementation summary | A brief description of what was implemented and why                  |

## Steps

### 1. Gather inputs and context

Ensure the required inputs are available before proceeding:

- If test failure output is missing, report the error and stop — there is nothing to implement against
- If test files are missing, attempt to extract them from the test failure output (most test runners include file paths in their output)
- If the test command is missing, attempt to infer it from the test failure output or discover it from the build system (see Step 3)
- Parse target packages into a list of packages, crates, or directories to focus exploration

### 2. Check for revision feedback

- If feedback has been provided (from a failed test run, reviewer, or human), read the previously written implementation files and revise them rather than starting from scratch
- Address each feedback point: fix incorrect logic, add missing branches, adjust types, restructure code, or provide a reasoned rebuttal for feedback you disagree with
- When revising, re-read the current test failure output carefully — it may differ from the original failures if the previous implementation was partially correct

### 3. Discover codebase conventions

Systematically discover how the codebase organizes production code. Do not assume any particular language, framework, or style — let the codebase tell you.

#### 3a. Identify the language and build system

- Use `glob` to search for build and configuration files in the relevant packages:
  - `**/Cargo.toml`, `**/go.mod`, `**/pom.xml`, `**/build.gradle*`, `**/build.sbt`, `**/Package.swift`, `**/package.json`, `**/tsconfig.json`, `**/pyproject.toml`, `**/setup.py`, `**/Gemfile`, `**/mix.exs`, `**/*.csproj`, `**/DESCRIPTION` (R), `**/Makefile`, `**/CMakeLists.txt`
- Read the relevant config files to understand the project structure and dependencies

#### 3b. Study the existing source code

- Use `glob` and `grep` to find source files in the target packages
- Use `read_file` to examine 2–3 representative source files from the relevant packages to learn:
  - **Module layout**: How are files and directories organized? Is there a `src/` directory, flat layout, or nested modules?
  - **Naming conventions**: How are files, functions, types, constants, and variables named?
  - **Import patterns**: How does code import from other modules within the package? Are there re-exports?
  - **Error handling**: Does the codebase use exceptions, Result types, error codes, or a specific error library?
  - **Coding style**: Indentation, line length, brace style, comment style, documentation patterns
  - **Common patterns**: Builder patterns, trait implementations, factory functions, dependency injection, etc.
  - **Public API surface**: How does the package expose functionality? Through a `lib.rs`, `mod.rs`, `__init__.py`, `index.ts`, or similar entry point?

#### 3c. Understand the test expectations

- Read the test files listed in the inputs to understand exactly what the tests expect:
  - What types, functions, methods, or modules are imported?
  - What signatures are expected (parameter types, return types)?
  - What behavior is asserted (input → output mappings, error conditions, side effects)?
  - What struct fields, enum variants, or trait implementations are assumed?
- Cross-reference with the test failure output to confirm which specific expectations are unmet

### 4. Plan the implementation

Before writing code, form a clear plan:

- List every type, function, method, or module the tests need that does not yet exist
- For each item, determine:
  - Which file it belongs in (based on codebase conventions from Step 3b)
  - Its signature (from the test imports and assertions in Step 3c)
  - Its behavior (from the test assertions)
  - How it integrates with existing code (imports, trait implementations, module re-exports)
- Identify any existing code that must be modified (adding fields to structs, variants to enums, methods to impl blocks, entries to module declarations)
- Order the changes so each file can be written or modified in a logical sequence (e.g., types before functions that use them, modules before re-exports)

### 5. Write the implementation

Write the production code following the plan from Step 4 and the conventions from Step 3:

- **File placement**: Place new files according to the codebase's module layout convention. If adding to an existing file, insert code at a logical location (near related functions, at the end of an impl block, etc.)
- **Naming**: Follow the codebase's naming conventions exactly — casing, prefixes, suffixes, abbreviation style
- **Signatures**: Match the signatures the tests expect precisely. Do not add extra parameters, change return types, or alter visibility unless the tests require it.
- **Logic**: Write the simplest correct logic that makes the tests pass. Resist the urge to handle cases the tests do not cover or to add optimizations the tests do not require.
- **Error handling**: Use the codebase's established error handling pattern. If the tests expect specific error types or messages, implement those exactly.
- **Integration points**: If the new code must be registered, re-exported, or declared in a parent module, make those changes too:
  - Rust: add `mod` and `pub use` declarations in parent modules, update `lib.rs` or `mod.rs`
  - Python: update `__init__.py` imports if the package uses explicit re-exports
  - TypeScript/JS: update `index.ts` barrel exports if the package uses them
  - Go: ensure the package name matches the directory convention
- **Dependencies**: If the implementation requires a new dependency (crate, package, module), add it to the appropriate manifest file (`Cargo.toml`, `package.json`, `pyproject.toml`, etc.). Add only what is strictly needed.

### 6. Verify the implementation compiles or parses

Before reporting completion, verify the implementation is syntactically and structurally valid:

- **Compiled languages** (Rust, Go, C/C++, Java, C#, Swift, Kotlin, Scala): Run the appropriate check command:
  - Rust: `cargo check -p <crate>` or `cargo clippy -p <crate>` if the project uses clippy
  - Go: `go build ./...` or `go vet ./...`
  - Java/Kotlin: `mvn compile` or `gradle compileJava`
  - C#: `dotnet build`
  - Swift: `swift build`
  - C/C++: `cmake --build build` or the project's build command
- **Interpreted languages** (Python, Ruby, R, Shell): Run a syntax check:
  - Python: `python -m py_compile <file>` for each modified file
  - Ruby: `ruby -c <file>`
  - R: `Rscript -e "parse('<file>')"`
- **Transpiled languages** (TypeScript): Run the type checker:
  - TypeScript: `npx tsc --noEmit` or the project's type-check command

If the check reveals errors:

- Fix the errors immediately
- Re-run the check to confirm the fix
- Repeat until the implementation compiles or parses cleanly
- Do not proceed to the summary with known compilation or parse errors

### 7. Present a summary

Output a clear summary including:

- The slice name and scope (if provided)
- **Implementation files** — the list of production code files created or modified
- **Implementation summary** — a brief description of what was implemented:
  - New types, functions, or modules created
  - Existing code modified and how
  - Integration changes (module declarations, re-exports, dependency additions)
- The verification command run and its result (clean compilation/parse or any remaining warnings)
- Any design decisions made and their rationale (e.g., "chose to implement X as an enum variant rather than a separate struct because the existing codebase uses this pattern for similar cases")
- Any concerns or caveats (e.g., "this implementation is minimal and will likely need refactoring for production use beyond the current test cases")

## Examples

See:

- `references/example-rust-new-function.md`
- `references/example-python-new-module.md`
- `references/example-typescript-new-component.md`
- `references/example-revision-after-partial.md`

## Edge Cases

- **Tests expect code in a file that already exists**: Read the existing file carefully. Add the new code without modifying or removing existing functionality. Place new functions, types, or methods at a logical location relative to existing code.
- **Tests expect modifications to existing types**: If a test asserts on a new field, variant, or method of an existing type, add only that field/variant/method. Do not refactor or restructure the existing type beyond what is needed.
- **Test expectations are ambiguous**: If the test assertions could be satisfied by multiple implementations, choose the simplest one. If the ambiguity is severe enough to risk wasted effort (e.g., the test checks a string but does not constrain the format), use `ask_user` if clarification is available, otherwise pick the most conventional approach and note the ambiguity in the summary.
- **Tests expect interaction with external systems**: Implement the code to satisfy the test's mocked or stubbed expectations. Do not introduce real external dependencies (network calls, file I/O, database connections) unless the tests explicitly set up and exercise those.
- **Circular dependency risk**: If the new code would create a circular import or dependency, restructure the implementation to avoid it — typically by extracting shared types to a common module or by depending on traits/interfaces instead of concrete implementations. Note the restructuring in the summary.
- **Implementation requires a design decision not covered by tests**: When multiple valid approaches exist (e.g., eager vs lazy evaluation, mutable vs immutable, sync vs async), prefer the approach used by similar code in the codebase. If no precedent exists, choose the simpler approach and note the decision.
- **Some tests were already passing**: Do not modify code that makes existing tests pass. Only add or change code for the currently failing tests. Verify that previously passing tests still pass after your changes.
- **Test failure is caused by a test bug, not missing implementation**: If after careful analysis the test itself appears to have a bug (e.g., asserting an impossible condition, importing from a path that contradicts the project structure), note the suspected test issue in the summary rather than writing contorted implementation code. Do not modify test files — that is outside this skill's scope.
- **Multiple languages or packages involved**: If the slice spans multiple packages (e.g., a Rust backend and a TypeScript frontend), implement changes in each package following that package's conventions. Verify each package compiles or parses independently.
- **Compilation succeeds but with warnings**: Fix warnings that are straightforward (unused imports, unused variables from scaffolding). Note any warnings that cannot be resolved without the full test suite passing (e.g., unused functions that will be called by code in a future slice).
- **Implementation needs a new dependency**: Add only the minimum required dependency. Prefer dependencies already used elsewhere in the project. Note any new dependency additions in the summary so reviewers can evaluate them.

---

This page was generated from [`.stencila/skills/software-implementation/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/software-implementation/SKILL.md).
