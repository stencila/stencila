---
name: software-test-review
description: Evaluate the quality of TDD tests against slice acceptance criteria, codebase conventions, and Red-phase execution results, producing a structured review with Accept/Revise routing. Use when a workflow needs to review tests written during the Red phase of red-green-refactor — checking coverage of acceptance criteria, conformance with codebase test conventions, test quality (naming, assertions, isolation, readability), edge-case and error-path coverage, and whether Red-phase failures indicate correctly missing implementation. Reads slice metadata and test execution output from workflow context, discovers codebase conventions independently, and routes via Accept or Revise labeled edges.
keywords:
  - test review
  - test quality
  - test critique
  - TDD
  - red phase review
  - acceptance criteria coverage
  - test conventions
  - test naming
  - test assertions
  - test isolation
  - test readability
  - edge cases
  - error paths
  - Red-phase validation
  - workflow routing
  - Accept Revise
  - slice testing
  - workflow context
  - codebase conventions
  - test conformance
  - not test creation
  - not test writing
  - not test execution
  - not implementation
  - not refactoring
allowed-tools: read_file glob grep
---

## Overview

Review tests written during the Red phase of a TDD workflow. This skill is used by agents operating at a test-review node after tests have been created and executed. It reads slice metadata and test execution results from workflow context, independently discovers the codebase's test conventions, reads the test files, and evaluates them across multiple quality dimensions. The output is a structured review report that routes the workflow via `Accept` or `Revise` labeled edges.

This skill does not write or modify any code or test files. It only reads, evaluates, and reports.

The review answers one central question: **Are these tests good enough to drive the Green phase of implementation?** Tests that are accepted should be a reliable specification of the slice's expected behavior. Tests that need revision should come back with specific, actionable feedback so the test-creation agent can fix them efficiently.

## Context Keys

| Key                         | Direction | Type   | Description                                                 |
| --------------------------- | --------- | ------ | ----------------------------------------------------------- |
| `current_slice`             | Read      | String | Name or identifier of the current slice                     |
| `slice.scope`               | Read      | String | Concise description of what the slice covers                |
| `slice.acceptance_criteria` | Read      | String | Acceptance criteria for the slice                           |
| `slice.packages`            | Read      | String | Packages, crates, modules, or directories involved          |
| `slice.test_files`          | Read      | String | Comma-separated list of test file paths created or modified |
| `slice.test_command`        | Read      | String | Command used to run the tests                               |

The skill also reads the output from the most recent workflow node (the test execution node) to obtain the structured test results including pass/fail counts and failure details.

## Route Labels

| Label    | When to use                                                        |
| -------- | ------------------------------------------------------------------ |
| `Accept` | Tests are good enough to drive the Green phase                     |
| `Revise` | Tests need changes — feedback is provided for the test creator     |

## Steps

### 1. Read slice metadata from workflow context

- Call `workflow_get_context` for keys: `current_slice`, `slice.scope`, `slice.acceptance_criteria`, `slice.packages`, `slice.test_files`, `slice.test_command`
- If `current_slice` or `slice.test_files` is missing, report the error and route `Revise` — there is nothing to review
- If `slice.acceptance_criteria` is missing, check the delivery plan in `.stencila/plans/` for criteria related to the current slice and use those
- Parse `slice.test_files` into a list of file paths for reading

### 2. Read test execution results

- Call `workflow_get_output` to obtain the structured test execution report from the previous node
- Extract: number of tests passed, number failed, failure messages, and any notes about compilation errors or unexpected failures
- If no execution results are available, note this as a limitation — the review will assess test quality without execution feedback

### 3. Read the test files

- Use `read_file` to load each test file listed in `slice.test_files`
- If a file does not exist, flag it immediately — this indicates a problem with the test creation step

### 4. Discover codebase test conventions

Independently discover the codebase's test conventions to evaluate conformance. Do not assume conventions from the test files being reviewed — those files may deviate from the codebase's norms.

#### 4a. Find existing test files in the relevant packages

- Use `glob` to search for test files in the directories listed in `slice.packages`:
  - `**/*test*`, `**/*spec*`, `**/tests/**`, `**/__tests__/**`, `**/test/**`
- Exclude the test files being reviewed from the convention sample

#### 4b. Study 2–3 representative existing test files

- Use `read_file` to examine existing tests and learn:
  - **Test framework**: What assertions, decorators, attributes, or macros are used?
  - **File layout**: Are tests inline, in a sibling `tests/` directory, or in a parallel source tree?
  - **Naming**: How are test files and test functions named?
  - **Import patterns**: How does existing test code import the code under test?
  - **Assertion style**: Which assertion functions or macros are preferred?
  - **Setup/teardown**: Are there fixtures, helpers, or builder patterns?

#### 4c. If no existing tests are found

- Broaden the search to sibling packages or the project root
- If still nothing is found, note this in the review — convention conformance cannot be assessed and only intrinsic quality dimensions apply

### 5. Evaluate acceptance-criteria coverage

For each acceptance criterion in `slice.acceptance_criteria`:

- Determine whether at least one test directly verifies the criterion
- Check that the test's assertion actually exercises the criterion, not just a superficially related behavior
- Flag any acceptance criteria that have no corresponding test
- Flag tests that do not map to any acceptance criterion (over-testing is a mild concern; missing coverage is a serious one)

Produce a coverage matrix:

| Acceptance Criterion | Covered By | Status |
|---|---|---|
| Criterion text | `test_function_name` | ✅ Covered / ❌ Missing / ⚠️ Weak |

A criterion is **Weak** if a test exists but does not adequately verify the criterion (e.g., it asserts the wrong property, uses a trivial input, or only tests the happy path when the criterion includes error handling).

### 6. Evaluate codebase convention conformance

Compare the test files against the conventions discovered in Step 4:

- **File placement**: Are test files in the expected location?
- **Naming**: Do test file names and test function names follow the codebase convention?
- **Framework usage**: Are the same test framework and assertion style used?
- **Import patterns**: Do imports follow the existing convention?
- **Organization**: Are tests grouped and structured consistently with existing tests?

Convention deviations are lower severity than coverage gaps but should still be flagged because they create maintenance burden and inconsistency.

### 7. Evaluate test quality

Assess each test across these dimensions:

#### 7a. Naming

- Test names should describe the behavior being tested, not the implementation detail
- Good: `test_parse_returns_malformed_token_error_for_empty_string`
- Bad: `test1`, `test_parse`, `test_it_works`
- Names should be specific enough that a failure message tells you what broke without reading the test body

#### 7b. Assertions

- Each test should have meaningful assertions that verify observable behavior
- Assertions should test the right property (return value, side effect, error type — whatever the criterion requires)
- Avoid tests that assert only that code runs without error unless that is the specific criterion
- Check for over-assertion: testing internal implementation details that may change creates brittle tests

#### 7c. Isolation

- Each test should be independent — no test should depend on the outcome or side effects of another test
- Tests should set up their own state rather than relying on shared mutable state
- If fixtures or setup code is shared, it should be clearly defined and not create hidden coupling

#### 7d. Readability

- Tests should be understandable without reading the implementation
- Arrange-Act-Assert or Given-When-Then structure should be discernible
- Magic numbers and cryptic test data should be avoided or explained with named constants or comments
- The purpose of each test should be clear from its name and body together

#### 7e. Minimality

- Tests should verify what the slice requires, not more
- Avoid testing behavior that belongs to other slices or to code that already exists and is already tested
- Each test function should focus on one logical behavior

### 8. Evaluate edge-case and error-path coverage

- Check whether the acceptance criteria imply edge cases (empty input, boundary values, null/None, maximum sizes, concurrent access, etc.)
- Check whether error paths specified in the criteria have corresponding tests
- Flag missing edge-case tests when the criteria clearly require them
- Do not penalize the absence of edge-case tests that go beyond the acceptance criteria — test scope should match slice scope

### 9. Validate Red-phase failure mode

Using the test execution results from Step 2:

- **All tests should fail**: In the Red phase, tests should fail because the implementation does not exist or is incomplete, not because of test bugs
- **Correct failure reasons**: Check that failure messages indicate missing functions, unresolved imports, unimplemented methods, or assertion failures against stub/missing behavior — not syntax errors, framework misconfiguration, or import errors for existing code
- **No tests should pass**: If some tests pass during Red phase, they may be testing existing behavior rather than new slice behavior — flag this as a concern (it may be acceptable if the slice extends existing code, but it deserves scrutiny)
- **Compilation/parse errors**: If the test runner failed due to compilation or parse errors, this is a significant issue — the tests have syntax or structural problems that must be fixed before they can serve as a specification

If no execution results are available, skip this step and note the limitation.

### 10. Produce the structured review report

Follow the Report Format below.

### 11. Route the workflow

Route `Accept` when:

- All acceptance criteria are covered by at least one test (no ❌ Missing in the coverage matrix)
- There are no High-severity findings
- Red-phase failures (if available) indicate correctly missing implementation, not test bugs

Route `Revise` when:

- Any acceptance criterion is missing test coverage, OR
- Any High-severity finding exists (test bugs, syntax errors, serious quality problems), OR
- Red-phase failures indicate test defects rather than missing implementation

When routing `Revise`, the review report serves as feedback for the test-creation agent. Make findings specific and actionable so the test creator knows exactly what to fix.

When in doubt, route `Revise` — it is safer to improve tests before driving implementation than to accept weak tests that lead to incorrect Green-phase code.

## Report Format

### Overall Assessment

One to three sentences summarizing the test suite's quality and the most important finding. State the routing decision (Accept or Revise) and the primary reason.

### Strengths

A short bullet list of what the tests do well. Recognizing strengths helps the test creator know what to preserve during revision.

### Acceptance-Criteria Coverage

The coverage matrix from Step 5. This is the most important section — missing coverage is the primary reason for Revise routing.

### Findings

Group findings under these headings when relevant (omit headings with no findings):

- **Convention conformance** — deviations from codebase test conventions
- **Test quality** — naming, assertions, isolation, readability, minimality issues
- **Edge cases and error paths** — missing coverage for edge cases or error handling
- **Red-phase validation** — problems with how tests fail (or unexpectedly pass)

For each finding:

- Indicate severity as **High**, **Medium**, or **Low**
- Describe the issue precisely, referencing the specific test function or file
- Explain why it matters

Severity guidelines:

- **High**: Missing acceptance-criteria coverage, test bugs, syntax/compilation errors, tests that pass when they should fail, tests that fail for the wrong reason
- **Medium**: Convention violations, weak assertions, poor naming, missing edge-case tests that the criteria imply, tests that over-assert on implementation details
- **Low**: Style inconsistencies, minor naming improvements, opportunities to simplify setup, tests that go slightly beyond slice scope

### Recommendations

A numbered list of concrete improvements in priority order. Each recommendation should say what to change, where, and why. When useful, suggest specific test names, assertion patterns, or restructuring approaches.

## Examples

### Example 1: Accept — well-covered tests with minor style issues

Slice scope: "Token validation for auth module"
Acceptance criteria: `AuthToken` struct has `sub`, `exp`, `iat`, `roles` fields; `AuthError::MalformedToken` returned for empty string; expired tokens rejected

Test execution: 3 tests, all failed with `error[E0433]: failed to resolve: could not find AuthToken in auth` — correct Red-phase behavior.

Review:

> ### Overall Assessment
>
> The test suite covers all three acceptance criteria with clear, well-named tests that fail correctly in the Red phase. **Accept** — tests are ready to drive the Green phase.
>
> ### Strengths
>
> - Each acceptance criterion maps to a dedicated test
> - Test names describe the expected behavior precisely
> - Failure messages indicate missing implementation, not test bugs
>
> ### Acceptance-Criteria Coverage
>
> | Acceptance Criterion | Covered By | Status |
> |---|---|---|
> | `AuthToken` has required fields | `test_auth_token_has_required_fields` | ✅ Covered |
> | `MalformedToken` for empty string | `test_parse_returns_malformed_token_for_empty_string` | ✅ Covered |
> | Expired tokens rejected | `test_expired_token_is_rejected` | ✅ Covered |
>
> ### Findings
>
> **Convention conformance**
> - **Low**: `test_auth_token_has_required_fields` uses `assert!` for field existence checks, while existing tests in the package use `assert_eq!` for struct field verification. Consistency is preferred but not blocking.
>
> ### Recommendations
>
> 1. Consider switching field assertions to `assert_eq!` to match existing conventions in the `auth` crate — this is minor and does not block acceptance

### Example 2: Revise — missing coverage and test bugs

Slice scope: "CSV parser error handling"
Acceptance criteria: `parse_csv` raises `CsvError::EmptyFile` for empty input; `parse_csv` skips malformed rows and returns partial results; `parse_csv` raises `CsvError::EncodingError` for non-UTF-8 input

Test execution: 2 tests failed with `SyntaxError: invalid syntax` — test file has Python syntax errors.

Review:

> ### Overall Assessment
>
> The test suite is missing coverage for one of three acceptance criteria, and the test file has a syntax error that prevents execution. **Revise** — fix the syntax error and add the missing encoding-error test.
>
> ### Strengths
>
> - Test names are descriptive and follow the project's `test_<behavior>_<condition>` convention
> - The empty-file test has a clear assertion against the specific error type
>
> ### Acceptance-Criteria Coverage
>
> | Acceptance Criterion | Covered By | Status |
> |---|---|---|
> | `EmptyFile` for empty input | `test_parse_csv_raises_empty_file_for_empty_input` | ✅ Covered |
> | Skip malformed rows, return partial | `test_parse_csv_skips_malformed_rows` | ⚠️ Weak |
> | `EncodingError` for non-UTF-8 | — | ❌ Missing |
>
> ### Findings
>
> **Red-phase validation**
> - **High**: Test file has a syntax error on line 23 — missing closing parenthesis in the `test_parse_csv_skips_malformed_rows` function. Tests fail due to `SyntaxError`, not due to missing implementation.
>
> **Test quality**
> - **Medium**: `test_parse_csv_skips_malformed_rows` only checks that the function returns a list, but does not assert the partial results contain the valid rows. The assertion should verify the content of the returned data, not just its type.
>
> **Edge cases and error paths**
> - **High**: No test for `CsvError::EncodingError` when given non-UTF-8 input. This is an explicit acceptance criterion.
>
> ### Recommendations
>
> 1. Fix the syntax error on line 23 of `tests/test_parser.py` — add the missing closing parenthesis
> 2. Add a test `test_parse_csv_raises_encoding_error_for_non_utf8` that passes binary non-UTF-8 data and asserts `CsvError.EncodingError` is raised
> 3. Strengthen `test_parse_csv_skips_malformed_rows` to assert the returned list contains only the valid rows, not just that it is a list

## Edge Cases

- **No test execution results available**: Review the tests on their quality dimensions alone. Skip the Red-phase validation step. Note in the report that execution feedback was unavailable. The review can still route `Accept` or `Revise` based on coverage and quality alone.
- **Test files do not exist**: If `slice.test_files` lists files that cannot be found, route `Revise` immediately with a clear finding that the test files are missing.
- **No existing tests in the codebase for convention discovery**: Note that convention conformance cannot be assessed. Evaluate tests on intrinsic quality dimensions only. Do not penalize convention deviations when no conventions could be discovered.
- **Acceptance criteria are vague**: Interpret the criteria as concretely as possible and evaluate coverage against that interpretation. Note which criteria were ambiguous and what interpretation was used. If the vagueness makes it impossible to judge coverage, flag this as a finding but do not automatically route `Revise` — the tests may be reasonable given the available information.
- **Tests intentionally pass during Red phase**: Some tests in a slice that extends existing code may pass because the existing code already satisfies part of the criterion. If the slice scope explains this, note it as acceptable. If it is unexpected, flag it for scrutiny.
- **Very large test files**: For test files with many tests, focus the review on the tests relevant to the current slice's acceptance criteria. Existing tests in the file that predate the current slice are outside the review scope.
- **Multiple test files across packages**: Review each file against its own package's conventions. Produce a single unified report covering all files.
- **Test creation agent added tests beyond acceptance criteria**: Mild over-testing is acceptable. Flag it as Low severity only if the extra tests risk confusion or maintenance burden. Do not route `Revise` solely for minor over-testing.
