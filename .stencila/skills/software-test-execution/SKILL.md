---
name: software-test-execution
description: Run scoped tests for a TDD slice, determine the appropriate test framework and command, and report structured pass/fail results. Use when tests need to be executed after writing, implementing, or refactoring code. Reads test metadata, discovers the test framework if needed, executes the scoped test command, parses output into structured results, and reports whether tests passed or failed. Handles compilation errors, missing dependencies, timeouts, and works with any language and test framework.
keywords:
  - test execution
  - test runner
  - run tests
  - test results
  - pass fail
  - TDD
  - red green refactor
  - scoped tests
  - test command
  - test framework discovery
  - compilation errors
  - test output parsing
  - test verification
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

Execute scoped tests for a TDD slice and report structured results. This skill runs only the tests relevant to the current slice, parses the output, and reports a clear pass/fail result with details.

This skill does not write or modify any code or test files. It only reads, executes, and reports.

## Required Inputs

This skill requires the following information to operate:

| Input          | Required | Description                                                          |
|----------------|----------|----------------------------------------------------------------------|
| Test command   | No       | Command to run the scoped tests (discovered if not provided)         |
| Test files     | No       | List of test file paths (used to scope the command if needed)        |
| Slice scope    | No       | Description of what the slice covers (for context in the report)     |
| Target packages| No       | Packages, crates, or directories involved (for command discovery)    |
| Slice name     | No       | Name or identifier of the current slice (for the report header)      |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

After completing its work, this skill reports:

| Output       | Description                                                    |
|--------------|----------------------------------------------------------------|
| Result       | Whether the tests passed or failed (Pass / Fail)               |
| Report       | A structured report with counts, failure details, and notes    |

## Steps

### 1. Gather test metadata

Ensure the available inputs are collected:

- The test command is the primary input — if present, use it directly
- Test files and target packages are used as fallbacks for constructing a test command if no command is provided
- Slice name and scope provide context for the report

### 2. Determine the test command

If a test command is provided and non-empty, use it. Otherwise, discover the command:

#### 2a. Identify the build system

- Use `glob` to search for build files in the target directories:
  - `Cargo.toml`, `go.mod`, `package.json`, `pyproject.toml`, `setup.py`, `pom.xml`, `build.gradle*`, `Gemfile`, `mix.exs`, `Makefile`
- Read the relevant config files to understand the test configuration

#### 2b. Check for canonical test commands

- Look in `Makefile` for `test:` targets
- Check `package.json` `scripts.test` for JS/TS projects
- Check CI config files (`.github/workflows/*.yml`) for test commands
- Consult `references/framework-detection.md` for the mapping from build file to test command

#### 2c. Construct a scoped command

- If test files list specific files, scope the command to those files
- If target packages name specific packages, scope to those packages
- Never run the full project test suite when scoping is possible
- See `references/framework-detection.md` for scoping patterns per framework

### 3. Execute the test command

- Run the test command via `shell` with an appropriate timeout (default 120 seconds; increase to 300 seconds for compiled languages like Rust, Go, or C++ where compilation may be slow)
- Capture both stdout and stderr — test output may appear on either stream
- Record the exit code

#### Timeout handling

- If the command times out, report it as a failure with an explanation that the test suite exceeded the time limit
- Suggest possible causes: infinite loops, network-dependent tests, excessive test scope

#### Compilation or collection errors

- If the output contains compilation errors (e.g., `error[E...]` in Rust, `SyntaxError` in Python, `Cannot find module` in JS), report this as a failure
- Include the compilation error details in the structured output so the upstream agent can fix them
- Do not attempt to fix the errors — this skill only executes and reports

#### Missing dependencies

- If tests fail because a test framework or dependency is not installed (e.g., `ModuleNotFoundError: No module named 'pytest'`, `error: no matching package named`), report this as a failure
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
3. At least one test actually ran (zero tests executed is suspicious and should be flagged as a potential issue, but still reported as Pass if exit code is 0)

The result is **Fail** if:

1. The exit code is non-zero, OR
2. Any test failures appear in the output, OR
3. A compilation or collection error prevented tests from running

When the exit code and output disagree (e.g., exit code 0 but failures in output), trust the output — some frameworks have bugs or configurations that mask failures in exit codes.

### 6. Report structured results

Output a clear structured report:

````
## Test Results: [PASS | FAIL]

**Slice**: <slice name>
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

## Examples

### Example 1: Rust — tests pass

Test command: `cargo test -p my-auth`, scope: "token validation"

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

Report: PASS, 4 passed, 0 failed. Result: Pass.

### Example 2: Python — tests fail

Test command: `pytest tests/test_parser.py`, scope: "CSV parser error handling"

```
$ pytest tests/test_parser.py
FAILED tests/test_parser.py::test_empty_file_raises_error - ImportError: cannot import name 'parse_csv'
FAILED tests/test_parser.py::test_malformed_row_skipped - ImportError: cannot import name 'parse_csv'
====== 0 passed, 2 failed in 0.12s ======
```

Report: FAIL, 0 passed, 2 failed. Failures are import errors because the implementation does not exist yet — expected in Red phase. Result: Fail.

### Example 3: No test command provided — discovery needed

Test command: absent. Target packages: "frontend". Test files: "frontend/src/components/__tests__/Button.test.tsx"

Discovery:
- `glob` finds `frontend/package.json`
- Read `package.json` → `"scripts": { "test": "vitest run" }`
- Construct scoped command: `cd frontend && npx vitest run src/components/__tests__/Button.test.tsx`

Execute, parse, report as normal.

### Example 4: Compilation error

Test command: `cargo test -p my-codec`

```
$ cargo test -p my-parser
error[E0433]: failed to resolve: could not find `Parser` in `parser`
 --> src/lib.rs:15:22
   |
15 |     use crate::parser::Parser;
   |                        ^^^^^^^ not found in `parser`
```

Report: FAIL, 0 passed, 0 failed (compilation error prevented tests from running). Include the full compiler error. Result: Fail.

## Edge Cases

- **No test command provided and no build system detected**: Search for a `Makefile` with a test target or CI config with a test step. If nothing is found, report the failure clearly — "Could not determine how to run tests for this project" — and report Fail.
- **Test command runs but produces no output**: Some frameworks produce no output on success. If exit code is 0 and no failures are detected, report Pass with a note that no detailed output was available. If exit code is non-zero with no output, report Fail and suggest checking stderr or increasing verbosity.
- **Flaky tests**: If a test sometimes passes and sometimes fails, report the observed result and note in the report that the test may be flaky (e.g., if the failure is timing-related or network-dependent). Do not re-run automatically — let the upstream agent decide.
- **Very large test output**: If the output exceeds what can be reasonably included in a report (thousands of lines), summarize: include the summary line, all failure details, and the first few passing tests. Truncate with a note about the total count.
- **Multiple test commands needed**: If test files span multiple packages that require different test commands (e.g., both a Rust crate and a Python package), run each command separately and combine the results. The overall result is Pass only if all commands pass.
- **Tests pass but with warnings**: Warnings do not affect the Pass/Fail result. Include notable warnings in the Notes section of the report (e.g., deprecation warnings, compiler warnings about unused code).
- **Zero tests executed**: If the test runner reports 0 tests run and exits with code 0, report as Pass but flag this prominently — it usually means the test filter matched nothing, which may indicate stale test file paths or an incorrect scoping pattern.
- **Test command not found**: If the `shell` execution fails because the test runner binary is not installed (e.g., `command not found: pytest`), report Fail with the specific error so the upstream agent can install the dependency.
- **Workspace-level vs package-level test commands**: Some projects run tests from the workspace root (e.g., `cargo test -p <crate>` from the repo root) while others require `cd <dir> && <command>`. Check whether the test command includes a directory change or package selector, and if it fails with a "not found" error, try running from the workspace root with a package flag.
