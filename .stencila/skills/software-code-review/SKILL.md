---
name: software-code-review
title: Software Code Review Skill
description: Evaluate source code for correctness, quality, security, style conformance, and maintainability, producing a structured review report with findings and recommendations. Use when the user wants to review, critique, audit, evaluate, or inspect source code — checking for bugs, logic errors, unhandled error paths, security vulnerabilities, naming and readability issues, complexity, duplication, coupling, testability, and API design. Discovers codebase conventions independently and produces an actionable report with severity-graded findings grouped by category, prioritized recommendations, and open questions.
keywords:
  - code review
  - code audit
  - code quality
  - code inspection
  - bug detection
  - security review
  - vulnerability detection
  - error handling
  - style conformance
  - readability
  - complexity
  - maintainability
  - testability
  - API design
allowed-tools: read_file glob grep
---

## Overview

Review source code for correctness, quality, security, style conformance, and maintainability. This skill reads and evaluates code — it does not modify any files. The output is a structured review report with an overall assessment, strengths, severity-graded findings grouped by category, prioritized recommendations, and open questions.

Do not use this skill when the main task is to write code, refactor code, create tests, review tests, review a design, or review a delivery plan.

## Required Inputs

| Input               | Required | Description                                                          |
|---------------------|----------|----------------------------------------------------------------------|
| Target files        | Yes      | Paths to the source code files to review                             |
| Review focus        | No       | Specific areas to prioritize (e.g., "security", "error handling", "API design") |
| Target packages     | No       | Packages, crates, modules, or directories involved                   |
| Acceptance criteria | No       | Requirements or criteria the code should satisfy                     |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output          | Description                                                                   |
|-----------------|-------------------------------------------------------------------------------|
| Overall verdict | Summary assessment of the code's quality and readiness                        |
| Review report   | Structured report with strengths, findings, recommendations, and open questions |

## Steps

### 1. Gather inputs and context

Ensure the required inputs are available:

- If target files are missing, report the error — there is nothing to review
- If a review focus is specified, prioritize that area but still evaluate all categories
- If acceptance criteria are provided, check whether the code satisfies them

### 2. Read the target files

- Use `read_file` to load each target file
- If a file does not exist or cannot be read, flag it immediately as a finding
- Note the language, framework, and overall structure of each file

### 3. Discover codebase conventions

Independently discover the codebase's conventions to evaluate style conformance. Do not assume conventions from the target files themselves — those files may deviate from the codebase's norms.

#### 3a. Identify the language and build system

- Use `glob` to search for build and configuration files in the relevant packages:
  - `**/Cargo.toml`, `**/go.mod`, `**/package.json`, `**/tsconfig.json`, `**/pyproject.toml`, `**/setup.py`, `**/Gemfile`, `**/Makefile`, `**/CMakeLists.txt`
- Read relevant config files to understand the project structure, dependencies, and any linting or formatting tools configured

#### 3b. Study surrounding source code

- Use `glob` and `grep` to find source files in the target packages
- Use `read_file` to examine 2–3 representative source files (other than the target files) to learn:
  - **Naming conventions**: How are files, functions, types, constants, and variables named?
  - **Module layout**: How are files and directories organized?
  - **Import patterns**: How does code import from other modules?
  - **Error handling**: What error handling pattern does the codebase use?
  - **Coding style**: Indentation, line length, brace style, comment style, documentation patterns
  - **Common patterns**: Builder patterns, trait implementations, factory functions, dependency injection, etc.
  - **Idioms**: Language-specific idioms the codebase favors

#### 3c. If no surrounding code is found

- Broaden the search to sibling packages or the project root
- If still nothing is found, note this in the review — convention conformance will be assessed against general best practices for the language only

### 4. Evaluate correctness

Analyze the code for bugs, logic errors, and unhandled error paths:

#### 4a. Logic errors

- Off-by-one errors in loops, slices, or indexing
- Incorrect boolean logic (wrong operator, inverted condition, missing case)
- Unreachable code or dead branches that suggest a logic mistake
- Race conditions or incorrect ordering of operations
- Integer overflow, underflow, or truncation
- Null/None/nil dereferences or missing null checks where the type system does not prevent them

#### 4b. Error handling

- Unhandled error cases (swallowed errors, empty catch blocks, bare `unwrap()` in Rust, unchecked exceptions)
- Error messages that leak internal details or provide no useful information
- Missing validation of inputs, return values, or external data
- Resource leaks (unclosed files, connections, or handles)
- Inconsistent error handling strategy within the same module

#### 4c. Behavioral correctness

- Does the code do what its name, comments, and API contract suggest?
- Are there edge cases that would produce incorrect results (empty input, boundary values, large input, concurrent access)?
- Are type conversions safe, or could they lose precision or fail silently?

### 5. Evaluate security

Analyze the code for security vulnerabilities and unsafe patterns:

#### 5a. Injection

- SQL injection (string concatenation in queries instead of parameterized queries)
- Command injection (unsanitized input passed to shell commands)
- Path traversal (unsanitized file paths from user input)
- Cross-site scripting (XSS) if the code generates HTML or handles web content
- Template injection or format-string vulnerabilities

#### 5b. Credential and secret exposure

- Hardcoded secrets, API keys, tokens, or passwords
- Secrets logged to stdout, stderr, or log files
- Secrets passed as command-line arguments (visible in process listings)
- Sensitive data in error messages or stack traces

#### 5c. Unsafe input handling

- Missing input validation or sanitization
- Trusting user-supplied data for authorization decisions
- Deserialization of untrusted data without validation
- Buffer overflows or unbounded allocations from external input

#### 5d. Cryptography and authentication

- Use of weak or deprecated cryptographic algorithms
- Custom cryptography implementations instead of well-audited libraries
- Missing authentication or authorization checks
- Insecure default configurations

Flag security findings with appropriate severity — a hardcoded secret or SQL injection is High; a missing input length check may be Medium or Low depending on context.

### 6. Evaluate quality and style

Assess code quality and adherence to codebase conventions:

#### 6a. Naming

- Are names descriptive and consistent with the codebase conventions discovered in Step 3?
- Do function names describe what they do? Do variable names describe what they hold?
- Are abbreviations avoided unless they are well-established in the codebase?
- Do boolean variables and functions read as predicates?

#### 6b. Complexity

- Are functions or methods excessively long or doing too many things?
- Are there deeply nested conditionals or loops that could be flattened?
- Are there complex boolean expressions that should be extracted into named variables or helper functions?
- Are there functions with too many parameters?

#### 6c. Duplication

- Is there repeated code that could be extracted into a shared function or method?
- Are there copy-pasted blocks with minor variations that could be parameterized?
- Is there duplication across the target files that suggests a missing abstraction?

#### 6d. Readability

- Is the code understandable without extensive context?
- Is the control flow clear and easy to follow?
- Are magic numbers or cryptic constants explained with named constants or comments?
- Are comments accurate and helpful, or are they stale, misleading, or restating the obvious?

#### 6e. Convention alignment

- Does the code follow the naming, formatting, import, and structural conventions discovered in Step 3?
- Does error handling follow the codebase's established pattern?
- Does the code use the codebase's preferred idioms?

### 7. Evaluate maintainability

Assess the code's long-term maintainability:

#### 7a. Modularity

- Are responsibilities clearly separated?
- Does each function, method, or class have a single, well-defined purpose?
- Could the code be tested, reused, or replaced independently?

#### 7b. Coupling

- Is the code tightly coupled to external systems, global state, or implementation details of other modules?
- Are dependencies explicit (via parameters or constructors) or hidden (via global access or side effects)?
- Would changing one part of the code require changes in many other places?

#### 7c. Testability

- Can the code be unit-tested without elaborate setup?
- Are dependencies injectable or mockable?
- Are side effects isolated from business logic?
- Is there logic that is difficult to test because it is buried inside a large function or tightly coupled to I/O?

#### 7d. API design

- Are public interfaces clear, minimal, and hard to misuse?
- Are parameters and return types appropriate? Would callers need to do unnecessary work?
- Are optional or configuration parameters handled cleanly (builder pattern, options struct, default values)?
- Are error types informative and actionable for callers?

### 8. Check acceptance criteria (if provided)

If acceptance criteria were provided:

- For each criterion, assess whether the code satisfies it
- Flag criteria that appear unsatisfied or only partially satisfied
- Include a coverage matrix in the report

| Acceptance Criterion | Status | Notes |
|---|---|---|
| Criterion text | ✅ Satisfied / ❌ Not satisfied / ⚠️ Partially satisfied | Brief explanation |

### 9. Produce the structured review report

Follow the Report Format below.

## Report Format

### Overall Assessment

One to three sentences summarizing the code's quality and the most important findings. State the overall quality level and the primary area needing attention.

### Strengths

A short bullet list of what the code does well. Recognizing strengths helps the author know what to preserve.

### Acceptance-Criteria Coverage

Include this section only if acceptance criteria were provided. Use the coverage matrix from Step 8.

### Findings

Group findings under these headings when relevant (omit headings with no findings):

- **Correctness** — bugs, logic errors, unhandled error paths, edge-case failures
- **Security** — injection, credential exposure, unsafe input handling, cryptographic issues
- **Quality and style** — naming, complexity, duplication, readability, convention violations
- **Maintainability** — modularity, coupling, testability, API design issues

For each finding:

- Indicate severity as **High**, **Medium**, or **Low**
- Describe the issue precisely, referencing the specific file, function, and line range
- Explain why it matters

Severity guidelines:

- **High**: Bugs that would produce incorrect results, security vulnerabilities exploitable by an attacker, crashes or data loss, completely missing error handling for critical paths
- **Medium**: Logic that works but is fragile or likely to break under edge cases, security issues that require specific conditions to exploit, significant convention violations, high complexity that materially hinders understanding, poor API design that makes misuse easy
- **Low**: Style inconsistencies, minor naming improvements, small duplication, opportunities to simplify, documentation gaps, minor readability improvements

### Recommendations

A numbered list of concrete improvements in priority order. Each recommendation should say what to change, where, and why. When useful, suggest specific approaches, patterns, or restructuring strategies.

### Open Questions

List questions that should be answered to improve confidence in the review. Include this section only when such questions remain — for example, when the reviewer cannot determine whether a pattern is intentional without more context, or when the review focus was too narrow to fully assess a concern.

## Examples

### Example 1: Small function with a bug and a security issue

Target files: `src/auth/token.rs`

Review:

> ### Overall Assessment
>
> The token validation function has a critical logic error that accepts expired tokens and a hardcoded fallback secret. These must be fixed before the code is safe for production.
>
> ### Strengths
>
> - Clear function signature with appropriate error types
> - Good use of the codebase's `AuthError` enum for error reporting
> - Follows the module's existing import and naming conventions
>
> ### Findings
>
> **Correctness**
> - **High**: `validate_token` in `src/auth/token.rs:42-58` compares `token.exp > now` but should compare `token.exp < now` to detect expiration. Currently, all expired tokens are accepted and all valid tokens are rejected.
>
> **Security**
> - **High**: `src/auth/token.rs:15` contains a hardcoded fallback secret `"dev-secret-key"` used when the environment variable is not set. This would allow token forgery in any deployment that fails to set the variable.
> - **Medium**: `src/auth/token.rs:51` logs the full token payload at `debug` level, which could expose user claims in log files.
>
> **Quality and style**
> - **Low**: The function `validate_token` is 40 lines long. Extracting the claims-parsing logic into a helper would improve readability, consistent with how `parse_header` is structured in the same module.
>
> ### Recommendations
>
> 1. Fix the expiration check on line 45: change `token.exp > now` to `token.exp < now`
> 2. Remove the hardcoded fallback secret on line 15. Return an error when the environment variable is missing instead of falling back to an insecure default.
> 3. Redact or remove the debug log of the full token payload on line 51. Log only the token ID or a truncated hash if debugging information is needed.
> 4. Consider extracting the claims-parsing block (lines 30–45) into a `parse_claims` helper for consistency with the module's existing structure.

### Example 2: Multi-file review with convention and maintainability issues

Target files: `lib/handlers/user_handler.py`, `lib/handlers/order_handler.py`
Review focus: maintainability

Review:

> ### Overall Assessment
>
> Both handler files work correctly but share significant duplicated code and are tightly coupled to the database module through direct imports of internal functions. Refactoring the shared patterns into a base handler and using the public `db.query` API would substantially improve maintainability.
>
> ### Strengths
>
> - Both handlers follow the project's URL routing convention
> - Error responses use the standardized `ErrorResponse` format consistently
> - Input validation is present for all endpoints
>
> ### Findings
>
> **Quality and style**
> - **Medium**: `user_handler.py:12-35` and `order_handler.py:15-38` contain nearly identical authentication and permission-checking boilerplate. This pattern is repeated in both files and would need to be updated in multiple places if the auth logic changes.
> - **Low**: `order_handler.py:72` uses `camelCase` for the local variable `orderItems`, while the rest of the codebase uses `snake_case`. The codebase convention (confirmed in `lib/handlers/product_handler.py` and `lib/utils/helpers.py`) is `snake_case` throughout.
>
> **Maintainability**
> - **Medium**: Both handlers import `db._execute_raw_query` (a private function indicated by the underscore prefix) instead of using the public `db.query` interface. This creates tight coupling to the database module's internals and would break if the internal implementation changes.
> - **Medium**: `user_handler.py:88-120` contains a 32-line function `handle_update` that mixes validation, database access, cache invalidation, and response formatting. Separating these concerns would make each step independently testable.
> - **Low**: Neither handler has type annotations on function parameters or return values, while the three other handler files in `lib/handlers/` all use type annotations. Adding them would improve IDE support and catch type errors earlier.
>
> ### Recommendations
>
> 1. Extract the shared auth/permission boilerplate into a decorator or base handler class to eliminate duplication and centralize auth logic changes
> 2. Replace `db._execute_raw_query` imports with `db.query` in both handlers to depend on the public API
> 3. Break `handle_update` into smaller functions: `validate_update_request`, `apply_update`, `invalidate_user_cache`, to improve testability
> 4. Rename `orderItems` to `order_items` on line 72 of `order_handler.py` to match the codebase's `snake_case` convention
> 5. Add type annotations to function signatures in both handlers for consistency with the rest of `lib/handlers/`

## Edge Cases

- **Partial code or code snippets**: Review what is provided. Note any limitations caused by missing context (e.g., cannot assess error handling without seeing the caller, cannot assess security without seeing how input arrives). Do not refuse to review — provide the best assessment possible with available information.
- **Multi-file reviews**: Review each file against the same package's conventions. Produce a single unified report covering all files. Note cross-file issues such as duplicated patterns, inconsistent styles between files, or coupling between the reviewed files.
- **Code without tests**: Do not penalize the code for lacking tests in the correctness or quality findings — that is a testability observation under maintainability. Focus on the code itself. Note in the maintainability section whether the code's structure would make it easy or hard to test.
- **Generated code**: If the code appears to be generated (e.g., by a code generator, ORM, or protocol buffer compiler), note this and limit the review to correctness and security. Style, naming, and structural critiques are usually not actionable for generated code since the generator, not the code, should be changed.
- **Performance-sensitive code**: If code has comments indicating performance sensitivity (benchmarks, hot paths, optimization notes) or if the review focus includes performance, be conservative about recommending changes that could affect performance (e.g., adding function call overhead, changing data structures). Note performance concerns separately when the current code sacrifices readability for speed.
- **Very large files**: Focus on the highest-impact findings. Summarize patterns (e.g., "the same missing-null-check pattern appears in 12 functions") rather than listing every instance individually. Prioritize findings that affect correctness and security over style observations.
- **Code in unfamiliar languages**: If the language is not well-known, focus on universal concerns (logic errors, security, naming clarity, structural complexity) and note that language-specific idiom assessment may be limited.
- **Review focus specified**: When a specific focus is provided, give that area extra depth but still scan all categories. A security-focused review should still flag an obvious bug; a quality-focused review should still flag an obvious vulnerability.
- **Acceptance criteria provided but code is incomplete**: Flag unmet criteria as findings but distinguish between "the code does not implement this" and "the code implements this incorrectly."
- **No convention baseline discoverable**: If no surrounding code exists to establish conventions, evaluate against general best practices for the language and note the limitation. Do not invent conventions that may not apply.
