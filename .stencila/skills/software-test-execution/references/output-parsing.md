# Test Output Parsing Reference

Guidance for parsing test output from common frameworks to extract structured results.

## Common Result Patterns

### Python (pytest)

```
===== 3 passed, 2 failed, 1 skipped in 0.42s =====
```

- **Passed**: `.` in short output, or `PASSED` in verbose
- **Failed**: `F` in short output, or `FAILED` in verbose
- **Skipped**: `s` in short output, or `SKIPPED` in verbose
- **Summary line**: `=====` footer with counts
- **Failure details**: Between `FAILURES` header and the summary
- **Collection errors**: `ERROR collecting <file>` — usually import/syntax errors

### JavaScript/TypeScript (vitest, jest)

```
Tests: 2 failed, 3 passed, 5 total
```

- **Passed**: `✓` or `√` prefix
- **Failed**: `✕` or `×` prefix, or `FAIL` label
- **Skipped**: `○` prefix or `skipped` label
- **Summary line**: `Tests:` line with counts
- **Failure details**: Stack trace printed inline after the failing test

### Go (`go test`)

```
FAIL
ok   package/name   0.005s
FAIL package/other  0.003s
```

- **Passed**: `--- PASS: <TestName>` or `ok` line for the package
- **Failed**: `--- FAIL: <TestName>` or `FAIL` line for the package
- **Skipped**: `--- SKIP: <TestName>`
- **Compilation errors**: `# package/name` followed by error messages

### Ruby (RSpec)

```
3 examples, 1 failure, 1 pending
```

- **Passed**: `.` in short output
- **Failed**: `F` in short output
- **Pending**: `*` in short output
- **Summary line**: `N examples, N failures` footer

### Rust (`cargo test`)

```
test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

- **Passed**: `test <name> ... ok`
- **Failed**: `test <name> ... FAILED`
- **Skipped/ignored**: `test <name> ... ignored`
- **Summary line**: `test result: ok` or `test result: FAILED`
- **Failure details**: Printed between `---- <name> stdout ----` and the next test or summary
- **Compilation errors**: Output starts with `error[E...]` before any test runs

### Java (JUnit via Maven/Gradle)

```
Tests run: 5, Failures: 2, Errors: 0, Skipped: 1
```

- **Summary line**: `Tests run:` with counts
- **Failure details**: Stack traces in the `Failures:` section

## Determining Overall Pass/Fail

The overall result is **Pass** if and only if:

1. The test command exits with code 0, AND
2. No test failures are reported in the output, AND
3. At least one test was actually executed (a suite with 0 tests run is suspicious and should be flagged)

The overall result is **Fail** if:

1. The test command exits with a non-zero code, OR
2. Any test failures are reported, OR
3. A compilation or collection error prevented tests from running

## Structured Result Format

When reporting results, use this structure:

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

<failure detail / assertion message>

```

### Passed Tests
1. `<test_name>`
2. `<test_name>`

### Notes
- <any observations about flaky tests, warnings, unexpected behavior>
````
