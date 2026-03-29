---
title: "Software Test Review Skill"
description: "Evaluate the quality of TDD tests against slice acceptance criteria, codebase conventions, and Red-phase execution results, producing a structured review with Accept or Revise recommendations. Use when tests written during the Red phase of red-green-refactor need quality review — checking coverage of acceptance criteria, conformance with codebase test conventions, test quality (naming, assertions, isolation, readability, triviality), edge-case and error-path coverage, and whether Red-phase failures indicate correctly missing implementation. Flags trivial low-value tests that add more maintenance cost than testing value. Discovers codebase conventions independently and produces an actionable review report."
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
  - trivial tests
  - low-value tests
  - maintenance cost
  - test value
  - over-testing
  - tautological tests
  - edge cases
  - error paths
  - Red-phase validation
  - codebase conventions
  - test conformance
  - not test creation
  - not test writing
  - not test execution
  - not implementation
  - not refactoring
---

Evaluate the quality of TDD tests against slice acceptance criteria, codebase conventions, and Red-phase execution results, producing a structured review with Accept or Revise recommendations. Use when tests written during the Red phase of red-green-refactor need quality review — checking coverage of acceptance criteria, conformance with codebase test conventions, test quality (naming, assertions, isolation, readability, triviality), edge-case and error-path coverage, and whether Red-phase failures indicate correctly missing implementation. Flags trivial low-value tests that add more maintenance cost than testing value. Discovers codebase conventions independently and produces an actionable review report.

**Keywords:** test review · test quality · test critique · TDD · red phase review · acceptance criteria coverage · test conventions · test naming · test assertions · test isolation · test readability · trivial tests · low-value tests · maintenance cost · test value · over-testing · tautological tests · edge cases · error paths · Red-phase validation · codebase conventions · test conformance · not test creation · not test writing · not test execution · not implementation · not refactoring

> [!tip] Usage
>
> To use this skill, add `software-test-review` to the `allowed-skills` list in your agent's AGENT.md. You can also ask `#agent-creator` to build an agent that uses it.

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `glob`, `grep` |

# Instructions

## Overview

Review tests written during the Red phase of a TDD cycle. This skill evaluates tests across multiple quality dimensions: acceptance-criteria coverage, codebase convention conformance, test quality (including detection of trivial low-value tests), edge-case handling, and whether Red-phase failures indicate correctly missing implementation. The output is a structured review report with an Accept or Revise recommendation.

This skill does not write or modify any code or test files. It only reads, evaluates, and reports.

The review answers one central question: **Are these tests good enough to drive the Green phase of implementation?** Tests that are accepted should be a reliable specification of the slice's expected behavior. Tests that need revision should come back with specific, actionable feedback so the test-creation agent can fix them efficiently.

A corollary question is equally important: **Does every test earn its keep?** Each test carries ongoing maintenance cost — it must be loaded, run, kept passing through refactors, and understood by future developers. A test is only worth that cost if it provides meaningful confidence that a real behavior works correctly. Trivial tests that merely restate the code, assert tautologies, or verify something the type system already guarantees add net-negative value: they cost time to maintain and provide no real safety net. This review must actively identify and flag such tests.

The reviewer must distinguish between acceptance criteria that should drive automated tests and criteria that are better satisfied by implementation or human inspection. Documentation requirements, doc comments, README edits, changelog entries, and similar non-executable deliverables are usually **not** reasons to demand source-inspecting unit tests. Their absence from the test suite is not automatically a coverage gap.

## Required Inputs

This skill requires the following information to operate:

| Input                | Required | Description                                                 |
|----------------------|----------|-------------------------------------------------------------|
| Slice name           | Yes      | Name or identifier of the current slice                     |
| Slice scope          | Yes      | Concise description of what the slice covers                |
| Acceptance criteria  | Yes      | The criteria the tests must verify                          |
| Target packages      | Yes      | Packages, modules, or directories involved                  |
| Test files           | Yes      | List of test file paths to review                           |
| Test command         | No       | Command used to run the tests                               |
| Test execution results | No     | Structured output from running the tests (pass/fail counts, failure details) |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

After completing its work, this skill reports:

| Output         | Description                                                              |
|----------------|--------------------------------------------------------------------------|
| Recommendation | Whether the tests should be accepted or revised (Accept / Revise)        |
| Review report  | Structured report with coverage matrix, findings, and recommendations    |

## Steps

### 1. Gather slice and test information

Ensure the required inputs are available:

- If the slice name or test files are missing, report the error and recommend Revise — there is nothing to review
- If acceptance criteria are missing, attempt to infer them from a delivery plan in `.stencila/plans/` as a standalone convenience — look for criteria related to the current slice name. In workflow use, the stage prompt should always provide acceptance criteria explicitly.
- Parse test files into a list of file paths for reading

### 2. Review test execution results

If test execution results are available:

- Extract: number of tests passed, number failed, failure messages, and any notes about compilation errors or unexpected failures
- If no execution results are available, note this as a limitation — the review will assess test quality without execution feedback

### 3. Read the test files

- Use `read_file` to load each test file listed in the inputs
- If a file does not exist, flag it immediately — this indicates a problem with the test creation step

### 4. Discover codebase test conventions

Independently discover the codebase's test conventions to evaluate conformance. Do not assume conventions from the test files being reviewed — those files may deviate from the codebase's norms.

#### 4a. Find existing test files in the relevant packages

- Use `glob` to search for test files in the target package directories:
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

For each acceptance criterion:

- First classify the criterion as either:
  - **Test-appropriate executable behavior** — behavior that should reasonably be verified by automated tests
  - **Non-test deliverable** — documentation, comments, prose updates, manual review items, naming cleanup, or similar requirements better checked outside the automated test suite
- Determine whether at least one test directly verifies each **test-appropriate executable behavior** criterion
- Check that the test's assertion actually exercises the criterion, not just a superficially related behavior
- Flag any **test-appropriate executable behavior** criteria that have no corresponding test
- Do **not** mark a documentation-only or other non-test-deliverable criterion as missing merely because there is no automated test for it
- Flag tests that read source files, comments, or documentation solely to prove prose was added as an inappropriate testing strategy unless the repository already has a strong convention for such checks and the criterion explicitly calls for them
- Flag tests that do not map to any acceptance criterion — over-testing adds maintenance burden without providing safety. Assess each unmapped test for triviality using the criteria in Step 7f

Produce a coverage matrix:

| Acceptance Criterion | Covered By | Status |
|---|---|---|
| Criterion text | `test_function_name` | ✅ Covered / ❌ Missing / ⚠️ Weak / ➖ Non-test deliverable |

A criterion is **Weak** if a test exists but does not adequately verify the criterion (e.g., it asserts the wrong property, uses a trivial input, or only tests the happy path when the criterion includes error handling). A criterion is **Non-test deliverable** when it is better satisfied by implementation and review than by an automated test.

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
- Treat tests that inspect source files or comments only to confirm documentation was written as brittle and generally inappropriate for TDD; normally raise this as at least a Medium-severity quality issue

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

#### 7f. Triviality / Cost-Value Balance

Every test has a maintenance cost: it must be kept loadable, execute successfully, stay green through refactors, and be understood by future developers. A test is only worth that cost if it provides meaningful confidence that the system behaves correctly under conditions that could realistically fail. **Actively look for and flag trivial tests that add more maintenance cost than testing value.**

Flag a test as trivial if it matches any of these anti-patterns:

- **Tautological assertions** — the test asserts something that is guaranteed to be true by construction. Examples:
  - Creating an object with a field set to `42` and then asserting the field is `42` (this tests the language's assignment, not the code)
  - Asserting that a constant equals its own literal value
  - Building a value and immediately checking it matches itself
- **Type-system-verified properties** — the test verifies something the compiler or type checker already enforces. Examples:
  - Asserting that a function with a declared return type actually returns that type (e.g., checking a function that returns `Optional[str]` produces an `Optional`)
  - Checking that a required field exists on an object when the language would already reject its absence at compile time or construction time
  - Asserting that a value is one of the known variants of a discriminated union or enum
- **Constructor echo tests** — the test creates an object with known inputs and asserts each field matches the input, without exercising any logic, transformation, validation, or default-value computation. If the constructor has no interesting behavior (no validation, no defaults, no derived fields), testing it is tautological
- **Getter/setter round-trips with no logic** — the test calls a setter and then a getter to confirm the value is stored, when the getter/setter pair is trivial (no validation, no transformation, no side effects)
- **Existence checks with no behavior** — the test imports a symbol or calls a function with no assertions, only to confirm it exists and does not raise an error. This is sometimes called a "smoke test" but provides minimal value when the language's tooling already verifies the symbol exists
- **String/display representation checks for trivial formats** — asserting that a `toString()`, `__str__()`, `Display`, or similar string-conversion output matches a hardcoded string when the implementation is a trivial format string with no conditional logic
- **Duplicate coverage** — the test covers exactly the same behavior and code path as another test in the same suite, with no meaningfully different inputs, edge cases, or assertions

When evaluating whether a test is trivial, ask: **"What bug would this test catch that would not already be caught by the compiler, type checker, linter, or another test?"** If the answer is "none" or "only if someone makes a typo in a trivial one-liner," the test is trivial.

Trivial tests are not just useless — they are actively harmful because they:
- Create false confidence that "we have N tests" when those tests catch nothing
- Must be updated during refactors, slowing down legitimate changes
- Clutter the test suite, making it harder to find and understand the tests that matter
- Waste CI time on every run
- Encourage a culture of testing quantity over quality

**Severity**: Flag individual trivial tests as **Medium**. If the majority of the test suite consists of trivial tests (more than half), elevate this to a **High** finding because the suite provides inadequate real coverage despite appearing to have tests. A suite full of trivial tests is worse than a suite with fewer but meaningful tests.

### 8. Evaluate edge-case and error-path coverage

- Check whether the acceptance criteria imply edge cases (empty input, boundary values, null/None, maximum sizes, concurrent access, etc.)
- Check whether error paths specified in the criteria have corresponding tests
- Flag missing edge-case tests when the criteria clearly require them
- Do not penalize the absence of edge-case tests that go beyond the acceptance criteria — test scope should match slice scope

### 9. Validate Red-phase failure mode

If test execution results are available:

- **All tests should fail**: In the Red phase, tests should fail because the implementation does not exist or is incomplete, not because of test bugs
- **Correct failure reasons**: Check that failure messages indicate missing functions, unresolved imports, unimplemented methods, or assertion failures against stub/missing behavior — not syntax errors, framework misconfiguration, or import errors for existing code
- **No tests should pass**: If some tests pass during Red phase, they may be testing existing behavior rather than new slice behavior — flag this as a concern (it may be acceptable if the slice extends existing code, but it deserves scrutiny)
- **Compilation/parse errors**: If the test runner failed due to compilation or parse errors, this is a significant issue — the tests have syntax or structural problems that must be fixed before they can serve as a specification

If no execution results are available, skip this step and note the limitation.

### 10. Produce the structured review report

Follow the Report Format below.

### 11. Make the recommendation

Recommend **Accept** when:

- All test-appropriate executable acceptance criteria are covered by at least one test (no ❌ Missing in the coverage matrix for criteria that should be tested)
- There are no High-severity findings
- Red-phase failures (if available) indicate correctly missing implementation, not test bugs

Recommend **Revise** when:

- Any test-appropriate executable acceptance criterion is missing test coverage, OR
- Any High-severity finding exists (test bugs, syntax errors, serious quality problems, a majority-trivial test suite), OR
- Red-phase failures indicate test defects rather than missing implementation

When recommending Revise, the review report serves as feedback for the test-creation agent. Make findings specific and actionable so the test creator knows exactly what to fix. For trivial tests, explicitly recommend removing or replacing them — do not just note they are trivial but leave them in place. Specify what meaningful test (if any) should take their place, or state clearly that the test should be deleted with no replacement.

When in doubt, recommend Revise — it is safer to improve tests before driving implementation than to accept weak tests that lead to incorrect Green-phase code.

## Report Format

### Overall Assessment

One to three sentences summarizing the test suite's quality and the most important finding. State the recommendation (Accept or Revise) and the primary reason.

### Strengths

A short bullet list of what the tests do well. Recognizing strengths helps the test creator know what to preserve during revision.

### Acceptance-Criteria Coverage

The coverage matrix from Step 5. This is the most important section — missing coverage is the primary reason for a Revise recommendation.

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

- **High**: Missing acceptance-criteria coverage, test bugs, syntax/compilation errors, tests that pass when they should fail, tests that fail for the wrong reason, a test suite where the majority of tests are trivial (more maintenance cost than testing value)
- **Medium**: Convention violations, weak assertions, poor naming, missing edge-case tests that the criteria imply, tests that over-assert on implementation details, tests that verify documentation or comments by reading source files when inspection would be more appropriate, individual trivial tests (tautological assertions, constructor echo tests, type-system-verified properties, getter/setter round-trips with no logic — see Step 7f)
- **Low**: Style inconsistencies, minor naming improvements, opportunities to simplify setup, tests that go slightly beyond slice scope

### Recommendations

A numbered list of concrete improvements in priority order. Each recommendation should say what to change, where, and why. When useful, suggest specific test names, assertion patterns, or restructuring approaches.

## Examples

### Example 1: Accept — well-covered tests with minor style issues

Slice scope: "Token validation for auth module"
Acceptance criteria: `AuthToken` type has `sub`, `exp`, `iat`, `roles` fields; `AuthError.MalformedToken` returned for empty string; expired tokens rejected

Test execution: 3 tests, all failed with resolution errors indicating `AuthToken` cannot be found — correct Red-phase behavior.

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
> - **Low**: `test_auth_token_has_required_fields` uses a loose truthiness check for field existence, while existing tests in the package use equality assertions for field verification. Consistency is preferred but not blocking.
>
> ### Recommendations
>
> 1. Consider switching field assertions to equality checks to match existing conventions in the `auth` package — this is minor and does not block acceptance

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

### Example 3: Revise — majority trivial tests inflating coverage

Slice scope: "Add `Config` class with validation for the settings module"
Acceptance criteria: `Config` constructor validates that `port` is 1–65535; `Config` constructor validates that `host` is non-empty; constructor raises `ConfigError.InvalidPort` and `ConfigError.InvalidHost` respectively

Test suite has 8 tests. Test execution: 8 tests, all failed with import/resolution errors indicating `Config` cannot be found — correct Red-phase behavior.

Review:

> ### Overall Assessment
>
> While all acceptance criteria are covered, 5 of 8 tests are trivial — they test constructor field assignment, type identity, and default values with no validation logic. The suite gives a false sense of thoroughness. **Revise** — remove the 5 trivial tests and keep the 3 meaningful validation tests.
>
> ### Strengths
>
> - The three validation tests (`test_new_rejects_port_zero`, `test_new_rejects_empty_host`, `test_new_accepts_valid_config`) are well-named and assert the right error types
> - Red-phase failures are correct — all failures indicate missing implementation
>
> ### Acceptance-Criteria Coverage
>
> | Acceptance Criterion | Covered By | Status |
> |---|---|---|
> | Port validated 1–65535 | `test_new_rejects_port_zero`, `test_new_rejects_port_above_65535` | ✅ Covered |
> | Host validated non-empty | `test_new_rejects_empty_host` | ✅ Covered |
> | `InvalidPort` error returned | `test_new_rejects_port_zero` | ✅ Covered |
> | `InvalidHost` error returned | `test_new_rejects_empty_host` | ✅ Covered |
>
> ### Findings
>
> **Test quality — Triviality**
> - **High**: 5 of 8 tests (63%) are trivial, making the majority of the suite maintenance cost with no testing value:
>   - **Medium**: `test_config_has_port_field` — creates a `Config` with `port: 8080` and asserts `config.port == 8080`. This is a tautological constructor echo test; it tests that assignment works, not that the code is correct.
>   - **Medium**: `test_config_has_host_field` — same pattern: sets `host` to `"localhost"` and asserts it equals `"localhost"`. No logic is exercised.
>   - **Medium**: `test_config_default_port` — asserts that a default-constructed `Config` has `port == 0`. Unless the default value has business significance specified in the acceptance criteria, this tests the default mechanism, not application logic.
>   - **Medium**: `test_config_is_debug` — asserts that the string representation of a `Config` contains `"Config"`. This tests the auto-generated debug/string representation, which is guaranteed by the language framework.
>   - **Medium**: `test_config_clone` — clones a `Config` and asserts the clone equals the original. This tests the language's copy/clone mechanism, not application logic.
>
> ### Recommendations
>
> 1. **Delete** `test_config_has_port_field`, `test_config_has_host_field`, `test_config_default_port`, `test_config_is_debug`, and `test_config_clone` — they catch no bugs that the language toolchain would miss and will need updating on every refactor
> 2. Keep `test_new_rejects_port_zero`, `test_new_rejects_port_above_65535`, `test_new_rejects_empty_host`, and `test_new_accepts_valid_config` — these test real validation logic
> 3. The resulting 3-test suite will have 100% meaningful coverage of the acceptance criteria with zero maintenance waste

## Edge Cases

- **No test execution results available**: Review the tests on their quality dimensions alone. Skip the Red-phase validation step. Note in the report that execution feedback was unavailable. The review can still recommend Accept or Revise based on coverage and quality alone.
- **Test files do not exist**: If the listed test files cannot be found, recommend Revise immediately with a clear finding that the test files are missing.
- **No existing tests in the codebase for convention discovery**: Note that convention conformance cannot be assessed. Evaluate tests on intrinsic quality dimensions only. Do not penalize convention deviations when no conventions could be discovered.
- **Acceptance criteria are vague**: Interpret the criteria as concretely as possible and evaluate coverage against that interpretation. Note which criteria were ambiguous and what interpretation was used. If the vagueness makes it impossible to judge coverage, flag this as a finding but do not automatically recommend Revise — the tests may be reasonable given the available information.
- **Acceptance criteria include documentation or prose requirements**: Mark those rows in the coverage matrix as `➖ Non-test deliverable` unless the project already has an established automated check for them and the criterion clearly intends that. Do not recommend Revise solely because there is no unit test asserting docs exist.
- **Tests intentionally pass during Red phase**: Some tests in a slice that extends existing code may pass because the existing code already satisfies part of the criterion. If the slice scope explains this, note it as acceptable. If it is unexpected, flag it for scrutiny.
- **Very large test files**: For test files with many tests, focus the review on the tests relevant to the current slice's acceptance criteria. Existing tests in the file that predate the current slice are outside the review scope.
- **Multiple test files across packages**: Review each file against its own package's conventions. Produce a single unified report covering all files.
- **Test creation agent added tests beyond acceptance criteria**: Extra tests that verify meaningful behavior beyond the acceptance criteria are acceptable — flag as Low severity only if they risk confusion. However, if the extra tests are trivial (see Step 7f), flag them at Medium or High severity as appropriate. A test that goes beyond the acceptance criteria *and* is trivial should be removed, not merely noted.
- **AI-generated test suites with inflated counts**: AI test generators commonly produce suites padded with trivial tests — constructor echo tests, type-verification tests, and tautological assertions — to give the appearance of thorough coverage. Be especially vigilant with AI-generated tests: scrutinize every test against the triviality criteria in Step 7f and do not accept a suite at face value just because it has many tests.
- **Tests were added only to prove docs exist**: Recommend Revise unless there is an explicit repository convention or plan requirement for an automated docs-presence check. Prefer feedback telling the creator to remove those tests and leave the documentation requirement to implementation and human review.

---

This page was generated from [`.stencila/skills/software-test-review/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/software-test-review/SKILL.md).
