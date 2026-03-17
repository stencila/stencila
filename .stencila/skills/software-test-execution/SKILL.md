---
name: software-test-execution
description: Run scoped tests for a TDD slice, determine the appropriate test framework and command, report structured results, and route the workflow based on pass/fail outcomes. Use when a workflow needs to execute tests after the Red, Green, or Refactor phase — confirming tests fail in Red, pass in Green, and still pass after Refactor. Reads slice metadata from workflow context, discovers the test framework if needed, executes the scoped test command, parses output into structured results, and routes via Pass/Fail labeled edges. Handles compilation errors, missing dependencies, timeouts, and works with any language and test framework.
keywords:
  - test execution
  - test runner
  - run tests
  - test results
  - pass fail
  - test routing
  - TDD
  - red green refactor
  - scoped tests
  - test command
  - test framework discovery
  - compilation errors
  - test output parsing
  - workflow routing
  - test verification
  - slice testing
  - workflow context
  - cargo test
  - pytest
  - vitest
  - go test
  - not test creation
  - not test writing
  - not implementation
  - not refactoring
allowed-tools: read_file glob grep shell
---

## Overview

Execute scoped tests for a TDD slice and report structured results. This skill is used by agents operating at `RunTestsRed`, `RunTestsGreen`, and `RunTestsRefactor` nodes in a TDD workflow. It reads test metadata from workflow context, discovers the test command if not already stored, runs only the tests relevant to the current slice, parses the output, and routes the workflow via `Pass` or `Fail` labeled edges.

This skill does not write or modify any code or test files. It only reads, executes, and reports.

## Context Keys

| Key                 | Direction | Type   | Description                                                 |
| ------------------- | --------- | ------ | ----------------------------------------------------------- |
| `slice.test_command`| Read      | String | Command to run the scoped tests (e.g., `cargo test -p my-crate`)|
| `slice.test_files`  | Read      | String | Comma-separated list of test file paths                     |
| `slice.scope`       | Read      | String | Concise description of what the slice covers                |
| `slice.packages`    | Read      | String | Packages, crates, modules, or directories involved          |
| `current_slice`     | Read      | String | Name or identifier of the current slice                     |

## Route Labels

| Label  | When to use                                        |
| ------ | -------------------------------------------------- |
| `Pass` | All tests passed (exit code 0, no failures)        |
| `Fail` | Any test failed, or tests could not run            |

## Steps

### 1. Read slice metadata from workflow context

- Call `workflow_get_context` for keys: `slice.test_command`, `slice.test_files`, `slice.scope`, `slice.packages`, `current_slice`
- `current_slice` and `slice.scope` provide context for understanding what the tests cover
- `slice.test_command` is the primary input — if present, use it directly
- `slice.test_files` and `slice.packages` are used as fallbacks for constructing a test command if `slice.test_command` is absent

### 2. Determine the test command

If `slice.test_command` is present and non-empty, use it. Otherwise, discover the command:

#### 2a. Identify the build system

- Use `glob` to search for build files in the directories listed in `slice.packages`:
  - `Cargo.toml`, `go.mod`, `package.json`, `pyproject.toml`, `setup.py`, `pom.xml`, `build.gradle*`, `Gemfile`, `mix.exs`, `Makefile`
- Read the relevant config files to understand the test configuration

#### 2b. Check for canonical test commands

- Look in `Makefile` for `test:` targets
- Check `package.json` `scripts.test` for JS/TS projects
- Check CI config files (`.github/workflows/*.yml`) for test commands
- Consult `references/framework-detection.md` for the mapping from build file to test command

#### 2c. Construct a scoped command

- If `slice.test_files` lists specific files, scope the command to those files
- If `slice.packages` names specific packages, scope to those packages
- Never run the full project test suite when scoping is possible
- See `references/framework-detection.md` for scoping patterns per framework

### 3. Execute the test command

- Run the test command via `shell` with an appropriate timeout (default 120 seconds; increase to 300 seconds for compiled languages like Rust, Go, or C++ where compilation may be slow)
- Capture both stdout and stderr — test output may appear on either stream
- Record the exit code

#### Timeout handling

- If the command times out, report it as a `Fail` with an explanation that the test suite exceeded the time limit
- Suggest possible causes: infinite loops, network-dependent tests, excessive test scope

#### Compilation or collection errors

- If the output contains compilation errors (e.g., `error[E...]` in Rust, `SyntaxError` in Python, `Cannot find module` in JS), report this as a `Fail`
- Include the compilation error details in the structured output so the upstream agent can fix them
- Do not attempt to fix the errors — this skill only executes and reports

#### Missing dependencies

- If tests fail because a test framework or dependency is not installed (e.g., `ModuleNotFoundError: No module named 'pytest'`, `error: no matching package named`), report this as a `Fail`
- Include the missing dependency in the output so the upstream agent can address it

### 4. Parse the test output

Analyze the combined stdout/stderr output to extract:

- **Number of tests passed**
- **Number of tests failed**
- **Number of tests skipped or ignored**
- **Names and failure details for each failing test** (assertion messages, expected vs actual values, stack traces)
- **Names of passing tests** (when available in the output)

Use `references/output-parsing.md` for framework-specific parsing patterns. If the output format is unfamiliar, extract what you can and include the raw output in the report.

### 5. Determine the overall result

The result is **Pass** if and only if:

1. The exit code is 0
2. No test failures are reported in the output
3. At least one test actually ran (zero tests executed is suspicious and should be flagged as a potential issue, but still routed as Pass if exit code is 0)

The result is **Fail** if:

1. The exit code is non-zero, OR
2. Any test failures appear in the output, OR
3. A compilation or collection error prevented tests from running

When the exit code and output disagree (e.g., exit code 0 but failures in output), trust the output — some frameworks have bugs or configurations that mask failures in exit codes.

### 6. Report structured results

Output a clear structured report:

````
## Test Results: [PASS | FAIL]

**Slice**: <current_slice>
**Command**: `<the command that was run>`
**Exit code**: <code>
**Duration**: <time if available>

### Summary
- Passed: N
- Failed: N
- Skipped: N

### Failed Tests
1. `<test_name>` — <brief failure reason>
   ```
   <assertion message or error detail>
   ```

### Passed Tests
1. `<test_name>`

### Notes
- <any observations: warnings, slow tests, suspicious patterns>
````

If no tests failed, omit the "Failed Tests" section. If the list of passing tests is very long (>20), summarize rather than listing each one.

### 7. Route the workflow

- If the result is **Pass**, call `workflow_set_route` with label `Pass`
- If the result is **Fail**, call `workflow_set_route` with label `Fail`

The route must always be set. If you are unsure about the result, default to `Fail` — it is safer to retry than to proceed with broken code.

## Examples

### Example 1: Rust — tests pass in Green phase

Context: `slice.test_command` = `cargo test -p my-auth`, `slice.scope` = "token validation"

```
$ cargo test -p my-auth
   Compiling my-auth v0.1.0
    Finished test target(s)
     Running unittests src/lib.rs
running 4 tests
test token::tests::test_valid_token_parses ... ok
test token::tests::test_expired_token_rejected ... ok
test token::tests::test_malformed_token_returns_error ... ok
test token::tests::test_missing_roles_uses_empty_vec ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Report: PASS, 4 passed, 0 failed. Route: `Pass`.

### Example 2: Python — tests fail in Red phase

Context: `slice.test_command` = `pytest tests/test_parser.py`, `slice.scope` = "CSV parser error handling"

```
$ pytest tests/test_parser.py
FAILED tests/test_parser.py::test_empty_file_raises_error - ImportError: cannot import name 'parse_csv'
FAILED tests/test_parser.py::test_malformed_row_skipped - ImportError: cannot import name 'parse_csv'
====== 0 passed, 2 failed in 0.12s ======
```

Report: FAIL, 0 passed, 2 failed. Failures are import errors because the implementation does not exist yet — expected in Red phase. Route: `Fail`.

### Example 3: No test command in context — discovery needed

Context: `slice.test_command` is absent, `slice.packages` = "frontend", `slice.test_files` = "frontend/src/components/__tests__/Button.test.tsx"

Discovery:
- `glob` finds `frontend/package.json`
- Read `package.json` → `"scripts": { "test": "vitest run" }`
- Construct scoped command: `cd frontend && npx vitest run src/components/__tests__/Button.test.tsx`

Execute, parse, report, and route as normal.

### Example 4: Compilation error

Context: `slice.test_command` = `cargo test -p my-codec`

```
$ cargo test -p my-parser
error[E0433]: failed to resolve: could not find `Parser` in `parser`
 --> src/lib.rs:15:22
   |
15 |     use crate::parser::Parser;
   |                        ^^^^^^^ not found in `parser`
```

Report: FAIL, 0 passed, 0 failed (compilation error prevented tests from running). Include the full compiler error. Route: `Fail`.

## Edge Cases

- **`slice.test_command` is absent and no build system is detected**: Search for a `Makefile` with a test target or CI config with a test step. If nothing is found, report the failure clearly — "Could not determine how to run tests for this project" — and route `Fail`.
- **Test command runs but produces no output**: Some frameworks produce no output on success. If exit code is 0 and no failures are detected, report Pass with a note that no detailed output was available. If exit code is non-zero with no output, report Fail and suggest checking stderr or increasing verbosity.
- **Flaky tests**: If a test sometimes passes and sometimes fails, report the observed result and note in the report that the test may be flaky (e.g., if the failure is timing-related or network-dependent). Do not re-run automatically — let the upstream agent decide.
- **Very large test output**: If the output exceeds what can be reasonably included in a report (thousands of lines), summarize: include the summary line, all failure details, and the first few passing tests. Truncate with a note about the total count.
- **Multiple test commands needed**: If `slice.test_files` spans multiple packages that require different test commands (e.g., both a Rust crate and a Python package), run each command separately and combine the results. The overall result is Pass only if all commands pass.
- **Tests pass but with warnings**: Warnings do not affect the Pass/Fail routing. Include notable warnings in the Notes section of the report (e.g., deprecation warnings, compiler warnings about unused code).
- **Zero tests executed**: If the test runner reports 0 tests run and exits with code 0, route as `Pass` but flag this prominently — it usually means the test filter matched nothing, which may indicate stale `slice.test_files` or an incorrect scoping pattern.
- **Test command not found**: If the `shell` execution fails because the test runner binary is not installed (e.g., `command not found: pytest`), report Fail with the specific error so the upstream agent can install the dependency.
- **Workspace-level vs package-level test commands**: Some projects run tests from the workspace root (e.g., `cargo test -p <crate>` from the repo root) while others require `cd <dir> && <command>`. Check whether the test command includes a directory change or package selector, and if it fails with a "not found" error, try running from the workspace root with a package flag.
