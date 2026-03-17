---
name: software-test-creation
description: Write failing tests for a TDD slice based on acceptance criteria, codebase conventions, and slice metadata from workflow context. Use when a workflow needs the "red" phase of red-green-refactor — creating tests that define expected behavior before implementation exists. Discovers codebase test conventions first, writes test files that fail because the implementation does not exist yet, and stores test metadata back in workflow context. Works with any language or test framework.
keywords:
  - test creation
  - test writing
  - TDD
  - red green refactor
  - red phase
  - failing tests
  - test-first
  - test-first development
  - acceptance criteria
  - unit tests
  - integration tests
  - test conventions
  - test scaffolding
  - slice testing
  - workflow slice
  - workflow context
  - codebase conventions
  - test framework discovery
  - write failing tests
  - acceptance test derivation
allowed-tools: read_file write_file edit_file apply_patch glob grep shell ask_user
---

## Overview

Write failing tests for a TDD slice as part of a red-green-refactor workflow. This skill handles the "red" phase: it reads slice metadata from workflow context, discovers the codebase's existing test conventions, writes test files that compile or parse but fail because the implementation does not yet exist, and stores test metadata in workflow context for downstream agents.

The core principle is **discover first, prescribe only as fallback**. The agent must adapt to whatever language, framework, and conventions the codebase already uses. Language-specific defaults are provided only for the case where no existing test conventions can be found.

The tests must:

- Cover the slice's acceptance criteria
- Follow the codebase's existing test conventions (framework, assertion style, directory layout, naming)
- Compile or parse successfully (no syntax errors)
- Fail because the code under test does not exist or does not yet behave correctly
- Be minimal — test only what the slice requires, not more

## Context Keys

These are the workflow context keys this skill reads and writes:

| Key                         | Direction | Type   | Description                                                 |
| --------------------------- | --------- | ------ | ----------------------------------------------------------- |
| `current_slice`             | Read      | String | Name or identifier of the current slice                     |
| `slice.scope`               | Read      | String | Concise description of what the slice covers                |
| `slice.acceptance_criteria` | Read      | String | Acceptance criteria for the slice                           |
| `slice.packages`            | Read      | String | Packages, crates, modules, or directories involved          |
| `slice.test_files`          | Write     | String | Comma-separated list of test file paths created or modified |
| `slice.test_command`        | Write     | String | Command to run the tests (e.g., `cargo test -p auth`)       |

## Steps

### 1. Read slice metadata from workflow context

- Call `workflow_get_context` for keys: `current_slice`, `slice.scope`, `slice.acceptance_criteria`, `slice.packages`
- If `current_slice` is missing, report the error and stop — there is no slice to test
- If `slice.acceptance_criteria` is missing or empty, check the delivery plan in `.stencila/plans/` for criteria related to the current slice and use those
- If no matching plan file exists in `.stencila/plans/`, no criteria can be matched confidently, or the repository does not use plan files at all, stop and report that acceptance criteria are required before tests can be written; use `ask_user` if clarification is available in the current environment
- Parse `slice.packages` into a list of packages, crates, or directories to focus exploration

### 2. Check for reviewer or human feedback

- Call `workflow_get_context` with key `human.feedback` and check the previous workflow node output for revision notes
- If feedback exists, read the previously written test files (from `slice.test_files` in context) and revise them rather than starting from scratch
- Address each feedback point: fix incorrect assertions, add missing test cases, adjust naming, restructure, or provide a reasoned rebuttal for feedback you disagree with

### 3. Discover codebase test conventions

This is the most important step. Systematically discover how the codebase organizes and runs tests. Do not assume any particular language or framework — let the codebase tell you.

#### 3a. Identify the language and build system

- Use `glob` to search for build and configuration files in the relevant packages:
  - `**/Cargo.toml`, `**/go.mod`, `**/pom.xml`, `**/build.gradle*`, `**/build.sbt`, `**/Package.swift`, `**/package.json`, `**/tsconfig.json`, `**/pyproject.toml`, `**/setup.py`, `**/Gemfile`, `**/mix.exs`, `**/*.csproj`, `**/DESCRIPTION` (R), `**/Makefile`, `**/CMakeLists.txt`
- Read the relevant config files to understand the project structure and any test-specific configuration

#### 3b. Find existing test files

- Use `glob` to search broadly for test files:
  - `**/*test*`, `**/*spec*`, `**/tests/**`, `**/__tests__/**`, `**/test/**`, `**/spec/**`, `**/src/test/**`
- Narrow down to the packages listed in `slice.packages`

#### 3c. Study existing test conventions

- Use `read_file` to examine 2–3 representative test files from the relevant packages to learn:
  - **Test framework**: What assertions, decorators, attributes, or macros are used?
  - **File layout**: Are tests inline, in a sibling `tests/` directory, or in a parallel source tree?
  - **Naming**: How are test files and test functions named?
  - **Import patterns**: How does existing test code import the code under test?
  - **Setup/teardown**: Are there fixtures, helpers, setup methods, or builder patterns?
  - **Assertion style**: Which assertion functions or macros are preferred?

#### 3d. Determine the test command

- Check `Makefile`, CI config files (`.github/workflows/*.yml`, `.gitlab-ci.yml`), `package.json` scripts, or other build system files for the canonical test command
- Record the command for use in Step 7 and Step 8

#### 3e. Synthesize a convention summary

Before proceeding, form a clear mental model of:

- Language and test framework
- Test file location convention
- Test naming convention
- Assertion style
- Test command

If no existing tests are found in the package or its siblings, consult `references/fallback-test-conventions.md`.

### 4. Understand the code structure the tests will target

- Use `glob` and `grep` to examine the source files in `slice.packages`
- Identify what types, functions, methods, or modules the slice will introduce or modify
- For new code that does not exist yet, determine where it will live based on:
  - The slice scope description
  - The package's existing module structure
  - Naming conventions in the codebase
- Note the public API surface the tests need to exercise — function signatures, struct fields, enum variants, trait methods, etc.

### 5. Design the test cases

For each acceptance criterion in the slice:

- Derive one or more test cases that verify the criterion
- Each test should be focused: one logical assertion per test function (though multiple `assert` calls are fine if they verify the same behavior)
- Name tests descriptively, following the naming conventions discovered in Step 3:
  - Good: `test_parse_returns_malformed_token_error_for_empty_string`
  - Bad: `test1`, `test_parse`
- Include both positive cases (expected behavior) and negative cases (error handling, edge cases) when the acceptance criteria require them
- If the slice has N acceptance criteria, aim for at least N test cases — more if a criterion has multiple interesting inputs

### 6. Write the test files

Write the actual test code using the conventions discovered in Step 3:

- **File placement**: Follow the codebase's convention for where tests go. If the package uses inline test modules, add tests to the relevant source file. If the package uses separate test files, create them in the conventional location.
- **Imports and setup**: Import the types and functions under test as they will exist once the implementation is created. In some ecosystems this means the test run fails with unresolved import or symbol errors; in others the imports resolve but assertions fail at runtime. Both are acceptable red-phase outcomes if they directly reflect the missing or incomplete implementation.
- **Compilation/parse requirement**: The test file itself must be syntactically valid. In interpreted languages this usually means the file parses cleanly. In compiled languages this usually means the test source is well-formed even if compilation fails because referenced items do not yet exist. Do not leave behind syntax errors, malformed test structure, or broken framework usage.
- **Framework conventions**: Use the same test framework, assertion macros/functions, and patterns as existing tests in the package.
- **Framework installation**: If you must use a fallback framework because no existing test setup is present, install or declare the minimum required test framework package(s) as part of generating the failing tests. This is part of the skill's responsibility: the red-phase tests should be runnable enough to fail for the intended implementation reason, not simply because the test framework itself is missing.
- **Test organization**: Group related tests logically. Use descriptive module names or test class names where the convention calls for it.
- **Dependencies**: If the tests require a library that is not yet in the project's dependencies (for example, a test-only or dev dependency), add the minimal dependency changes needed for the chosen test framework to run, but do not add unrelated implementation dependencies. Note any dependency changes in the summary.

### 7. Verify the tests are valid and failing

- Run the test command determined in Step 3d to confirm:
  - **Syntax/parse validity**: The test files have no syntax errors
  - **Expected failure**: Tests fail because the implementation does not exist, not because of test bugs
- If the project has a syntax-check or compile-check command (e.g., `cargo check --tests`, `python -m py_compile`, `npx tsc --noEmit`), use that first for a faster feedback loop
- If the tests fail for unexpected reasons (syntax errors, wrong imports for existing code, framework misconfiguration), fix those issues before proceeding
- The tests should fail cleanly — with "not found", "does not exist", "cannot resolve", or assertion failure messages — not with cryptic errors

### 8. Store test metadata in workflow context

- Call `workflow_set_context` with key `slice.test_files` and the comma-separated list of test file paths that were created or modified
- Call `workflow_set_context` with key `slice.test_command` and the test command determined in Step 3d. If no canonical command was found, construct one from the language's standard test runner using `references/fallback-test-conventions.md`.

### 9. Present a summary

Output a clear summary including:

- The slice name and scope
- Number of test cases written
- Test file paths
- Test command
- Which acceptance criteria each test covers
- Expected failure mode (why the tests fail)
- Any ambiguities encountered in the acceptance criteria

## Examples

See:

- `references/example-python-pytest.md`
- `references/example-r-testthat.md`
- `references/example-typescript-vitest.md`
- `references/example-rust-inline-tests.md`

## Edge Cases

- **No existing tests in the package**: If the package has no test files to learn conventions from, look at sibling packages or the project root for test configuration. If no conventions are found anywhere, use `references/fallback-test-conventions.md`, selecting the row that matches the detected language. Note in the summary that no existing tests were found and conventions were inferred from the fallback table.
- **Multiple test frameworks detected**: If the package uses more than one test framework (e.g., both pytest and unittest, or both jest and vitest), prefer the one used by the majority of test files. Note the choice in the summary.
- **Tests need fixtures or setup**: If the acceptance criteria require test data, mock objects, or setup code, create minimal fixtures following the package's existing fixture patterns. Do not create elaborate fixture infrastructure beyond what the tests need.
- **Slice targets code across multiple packages**: Write tests in each relevant package following that package's conventions. Store all test file paths in `slice.test_files` and construct a combined test command.
- **Acceptance criteria are vague**: Derive the most concrete tests possible from the criteria. Add a comment in the test file noting which criterion is vague and what interpretation was used. Flag the ambiguity in the summary. If the ambiguity is severe enough to risk wasted effort, use `ask_user` to request clarification before writing tests.
- **Implementation partially exists**: If some of the code under test already exists (perhaps from a previous slice), write tests that import existing code correctly and only fail for the parts that are new. Verify that existing tests still pass.
- **Test file already exists at the target path**: Read the existing file and extend it rather than overwriting. Add new test functions alongside existing ones. Do not remove or modify existing tests unless feedback specifically requests it.
- **Cannot determine test command**: If the project uses a non-standard test runner or build system, use `grep` to search for test scripts in `Makefile`, `package.json`, `pyproject.toml`, CI configuration, or similar files. Store whatever command is most appropriate for running the specific tests written.
- **Cyclic dependency in test imports**: If the test needs to import from a module that imports from the module being tested, restructure the test to avoid the cycle — typically by testing through the public API rather than internal modules.
- **Language not in the fallback table**: If the codebase uses a language not listed in `references/fallback-test-conventions.md`, search for that language's most common test framework conventions online or infer from the build system, and note in the summary what conventions were chosen and why.
