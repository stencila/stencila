---
title: "Software Code Refactoring Skill"
description: "Improve production code quality while preserving all existing test behavior. Commonly used for the Refactor phase of TDD red-green-refactor, but applicable to any codebase with tests. Use when production code works but needs cleanup — reducing duplication, improving naming, simplifying complexity, aligning with codebase style, extracting functions or types, or removing dead code. Discovers conventions, applies safe transformations, verifies compilation after each change, and produces a structured summary. Works with any language or framework."
keywords:
  - refactoring
  - refactor phase
  - TDD
  - red green refactor
  - code quality
  - code cleanup
  - reduce duplication
  - DRY
  - naming improvement
  - simplify complexity
  - code style
  - extract function
  - extract type
  - inline variable
  - remove dead code
  - readability
  - maintainability
  - codebase conventions
  - preserve tests
  - safe transformation
---

Improve production code quality while preserving all existing test behavior. Commonly used for the Refactor phase of TDD red-green-refactor, but applicable to any codebase with tests. Use when production code works but needs cleanup — reducing duplication, improving naming, simplifying complexity, aligning with codebase style, extracting functions or types, or removing dead code. Discovers conventions, applies safe transformations, verifies compilation after each change, and produces a structured summary. Works with any language or framework.

**Keywords:** refactoring · refactor phase · TDD · red green refactor · code quality · code cleanup · reduce duplication · DRY · naming improvement · simplify complexity · code style · extract function · extract type · inline variable · remove dead code · readability · maintainability · codebase conventions · preserve tests · safe transformation

> [!tip] Usage
>
> To use this skill, add `software-code-refactoring` to the `allowed-skills` list in your agent's AGENT.md. You can also ask `#agent-creator` to build an agent that uses it.

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |

# Instructions

## Overview

Improve existing production code that already has passing tests. This skill is commonly used for the "refactor" phase of a red-green-refactor cycle, but works equally well as a standalone code-quality pass. Given passing production code and its tests, it discovers the codebase's conventions, identifies refactoring opportunities, applies safe transformations, and verifies that all tests continue to pass.

The core principles are:

- **Behavior preservation is non-negotiable.** Every refactoring must leave all existing tests passing. If a change breaks a test, revert it immediately and try a different approach.
- **Discover first, prescribe only as fallback.** Adapt to whatever language, framework, and conventions the codebase already uses. Align refactored code with these conventions rather than imposing external style preferences.
- **Small, verifiable steps.** Apply one refactoring at a time. Verify compilation/parsing after each change. This makes failures easy to diagnose and revert.
- **Public API stability.** Do not change public function signatures, type names, module exports, or other externally visible interfaces unless explicitly requested. Internal restructuring is the focus.

The refactoring must:

- Preserve all existing test behavior (all tests that passed before must still pass)
- Follow the codebase's existing conventions (naming, module layout, error handling, coding style)
- Not change the public API of any module, crate, or package
- Compile or parse without errors or warnings where possible
- Improve code quality in at least one measurable dimension (duplication, complexity, readability, naming, style consistency)

## Required Inputs

This skill requires the following information to operate:

| Input                | Required | Description                                                             |
|----------------------|----------|-------------------------------------------------------------------------|
| Target files         | Yes      | Paths to the production code files to refactor                          |
| Test command         | Yes      | The command to run the relevant tests                                   |
| Test files           | No       | Paths to the test files that exercise the target code                   |
| Refactoring focus    | No       | Specific areas to focus on (e.g., "reduce duplication", "improve naming", "simplify control flow") |
| Target packages      | No       | Packages, crates, modules, or directories involved                      |
| Revision feedback    | No       | Feedback from a test run, reviewer, or human on a previous attempt      |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

After completing its work, this skill reports:

| Output               | Description                                                            |
|----------------------|------------------------------------------------------------------------|
| Refactored files     | The list of production code files modified                             |
| Refactoring summary  | A structured description of each refactoring applied, why it was chosen, and what improved |
| Verification result  | The test command run and whether all tests still pass                   |

## Steps

### 1. Gather inputs and context

Ensure the required inputs are available before proceeding:

- If target files are missing, ask the user or attempt to infer them from the test files or target packages
- If the test command is missing, attempt to discover it from the build system (see Step 3)
- If test files are provided, note them for later analysis; if not, attempt to find them via naming conventions (e.g., `*_test.rs`, `test_*.py`, `*.test.ts`, `*_spec.rb`)

### 2. Check for revision feedback

- If feedback has been provided (from a failed test run, reviewer, or human), read the previously modified files and revise them rather than starting over
- Address each feedback point: revert a problematic refactoring, try a different approach, undo a change that broke tests, or provide a reasoned rebuttal for feedback you disagree with
- When revising, re-read the current test output carefully — the failures indicate which refactorings were unsafe

### 3. Discover codebase conventions

Systematically discover how the codebase organizes and styles its code. Do not assume any particular language, framework, or style — let the codebase tell you.

#### 3a. Identify the language and build system

- Use `glob` to search for build and configuration files in the relevant packages:
  - `**/Cargo.toml`, `**/go.mod`, `**/pom.xml`, `**/build.gradle*`, `**/package.json`, `**/tsconfig.json`, `**/pyproject.toml`, `**/setup.py`, `**/Gemfile`, `**/mix.exs`, `**/*.csproj`, `**/Makefile`, `**/CMakeLists.txt`
- Read the relevant config files to understand the project structure, dependencies, and any linting or formatting tools configured

#### 3b. Study the surrounding source code

- Use `glob` and `grep` to find source files in the target packages
- Use `read_file` to examine 2–3 representative source files (other than the target files) to learn:
  - **Module layout**: How are files and directories organized?
  - **Naming conventions**: How are files, functions, types, constants, and variables named?
  - **Import patterns**: How does code import from other modules within the package?
  - **Error handling**: What error handling pattern does the codebase use?
  - **Coding style**: Indentation, line length, brace style, comment style, documentation patterns
  - **Common patterns**: Builder patterns, trait implementations, factory functions, dependency injection, etc.
  - **Idioms**: Language-specific idioms the codebase favors (e.g., iterator chains vs loops in Rust, list comprehensions vs map/filter in Python)

#### 3c. Read the target files and their tests

- Read every target file to understand the current implementation
- If test files are available, read them to understand what behavior is being tested — this defines the contract that must be preserved
- Note the public API of each target file: exported functions, types, constants, traits, classes, and their signatures

### 4. Analyze refactoring opportunities

Examine the target files for improvement opportunities. Prioritize by impact and safety:

#### 4a. Duplication

- Repeated code blocks that could be extracted into a shared function or method
- Similar match arms, branches, or case statements that could be consolidated
- Copy-pasted logic with minor variations that could be parameterized

#### 4b. Naming

- Variables, functions, types, or constants with unclear, misleading, or inconsistent names
- Names that do not follow the codebase's naming conventions
- Abbreviations that harm readability without saving meaningful space
- Boolean variables or functions that do not read as predicates (e.g., `flag` vs `is_enabled`)

#### 4c. Complexity

- Functions or methods that are too long (doing too many things)
- Deeply nested conditionals or loops that could be flattened with early returns, guard clauses, or extraction
- Complex boolean expressions that could be named with a descriptive variable or helper function
- Functions with too many parameters that could use a configuration struct or builder

#### 4d. Style and idiom alignment

- Code that does not follow the codebase's established patterns (e.g., manual loops where the rest of the codebase uses iterator chains)
- Inconsistent error handling (e.g., mixing `unwrap()` with proper `Result` handling in a Rust codebase that uses `?` everywhere else)
- Inconsistent formatting or structure relative to sibling files

#### 4e. Dead code

- Unused imports, variables, functions, types, or fields
- Commented-out code blocks with no explanatory context
- Unreachable branches or match arms

#### 4f. Structure

- Functions or methods that would be more logically placed in a different module or file
- Missing or inconsistent grouping of related functionality
- Overly large files that could be split into focused modules

### 5. Plan the refactoring sequence

Before making changes, form a clear plan:

- List each refactoring to apply, in order
- For each refactoring, note:
  - What will change (specific code locations)
  - Why it improves the code (the quality dimension it addresses)
  - The risk level (low: renaming a local variable; medium: extracting a function; high: restructuring a module)
  - Whether it touches the public API (if yes, skip it unless explicitly requested)
- Order refactorings from safest to riskiest — if an early refactoring breaks something, the riskier ones never execute
- Group related refactorings (e.g., extract a function, then use it in three places)
- If a refactoring focus was provided, prioritize changes in that area but still apply other clear improvements

### 6. Apply refactorings

Apply each refactoring from the plan, one at a time:

#### 6a. Make the change

- Use `edit_file` or `apply_patch` for targeted modifications to existing files
- Follow the codebase's conventions discovered in Step 3
- Do not change public API signatures, exports, or externally visible behavior
- When extracting functions: place them near the call site or in a location consistent with the codebase's module organization; choose a name that clearly describes what the function does
- When renaming: update all references within the file and, for non-public items, in other files within the same package
- When removing dead code: verify the code is truly unreachable or unused by searching for references with `grep` before deleting

#### 6b. Verify compilation/parsing

After each change (or group of tightly related changes), verify the code still compiles or parses:

- **Compiled languages** (Rust, Go, C/C++, Java, C#, Swift, Kotlin, Scala): Run the appropriate check command:
  - Rust: `cargo check -p <crate>` or `cargo clippy -p <crate>` if the project uses clippy
  - Go: `go build ./...` or `go vet ./...`
  - Java/Kotlin: `mvn compile` or `gradle compileJava`
  - C#: `dotnet build`
  - Swift: `swift build`
- **Interpreted languages** (Python, Ruby, R, Shell): Run a syntax check:
  - Python: `python -m py_compile <file>` for each modified file
  - Ruby: `ruby -c <file>`
- **Transpiled languages** (TypeScript): Run the type checker:
  - TypeScript: `npx tsc --noEmit` or the project's type-check command

If the check reveals errors:

- Fix the errors immediately if the fix is straightforward
- If the fix is not straightforward, revert the change and note it as an unsafe refactoring in the summary
- Do not proceed to the next refactoring with broken compilation

#### 6c. Verify tests still pass

After completing all refactorings (or after each high-risk change), run the test command:

- If all tests pass, continue
- If any test fails:
  - Identify which refactoring caused the failure
  - Revert that specific refactoring
  - Re-run tests to confirm the revert fixed the failure
  - Note the failed refactoring in the summary with an explanation of why it was unsafe
  - Continue with remaining safe refactorings

### 7. Run the formatter and linter (if configured)

If the project has a configured formatter or linter, run it on the modified files:

- Rust: `cargo fmt -p <crate>`, `cargo clippy --fix --allow-dirty -p <crate>`
- Python: check for `ruff`, `black`, `isort` in project config and run them
- TypeScript/JavaScript: check for `prettier`, `eslint` in project config and run them
- Go: `gofmt -w <file>` or `goimports -w <file>`

This ensures the refactored code matches the project's automated style enforcement.

### 8. Present a summary

Output a clear summary including:

- **Refactored files** — the list of production code files modified
- **Refactoring summary** — for each refactoring applied:
  - What was changed (before → after, at a high level)
  - Why it was changed (which quality dimension it improves)
  - The category (duplication, naming, complexity, style, dead code, structure)
- **Refactorings not applied** — any planned refactorings that were skipped or reverted, with reasons
- **Verification result** — the test command run and its outcome (all tests passing)
- **Public API impact** — confirm that no public API was changed (or list any intentional changes if explicitly requested)

## Examples

See:

- `references/example-extract-function-rust.md`
- `references/example-naming-and-style-python.md`
- `references/example-reduce-duplication-typescript.md`
- `references/example-revert-unsafe-refactoring.md`

## Edge Cases

- **No clear refactoring opportunities**: If the target code is already clean, well-named, and follows codebase conventions, report this finding rather than making unnecessary changes. Not every code needs refactoring.
- **Refactoring would require changing the public API**: Do not change public function signatures, type names, module exports, or trait definitions unless the user explicitly requests it. Internal restructuring only. If a public API change would significantly improve quality, note it as a recommendation in the summary.
- **Refactoring improves one quality dimension but degrades another**: Prefer the version that is clearest to read. For example, extracting a tiny two-line function may reduce duplication but add indirection — skip it if the duplication is only in two places and the code is clear.
- **Test failures after refactoring**: Immediately revert the change that caused the failure. Do not attempt to fix the test — the refactoring was unsafe if it changed behavior. Note the reverted change in the summary.
- **Large files with many issues**: Focus on the highest-impact improvements first. Do not try to perfect an entire file in one pass — make the most valuable changes and note remaining opportunities for future passes.
- **Code that is already partially refactored (revision scenario)**: When receiving revision feedback, read the current state of the code carefully. Some refactorings from the previous attempt may have been correct — preserve those and only address the feedback points.
- **Conflicting conventions in the codebase**: If different parts of the codebase use different styles (e.g., some files use `snake_case` and others use `camelCase`), follow the convention used by the majority of files in the same package or module. Note the inconsistency in the summary.
- **Dead code that might be used by external consumers**: Before removing apparently dead code, use `grep` to search the entire project for references. If the code is part of a public API or library, it may have consumers outside the current codebase — do not remove it unless you can confirm it is unused.
- **Refactoring across multiple files**: When a refactoring spans multiple files (e.g., extracting a shared type to a common module), make all the changes needed for the refactoring to be complete — do not leave files in an inconsistent state. Verify compilation after the full set of related changes.
- **Performance-sensitive code**: If code has comments indicating performance sensitivity (benchmarks, hot paths, optimization notes), be conservative with refactorings that could affect performance (e.g., adding function call overhead, changing data structures). Note any concerns in the summary.

---

This page was generated from [`.stencila/skills/software-code-refactoring/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/software-code-refactoring/SKILL.md).
