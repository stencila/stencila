---
name: software-delivery-tdd
description: Execute a software delivery plan slice-by-slice using test-driven development with Red-Green-Refactor cycles, agent-driven scoped test execution, iterative review, and human approval after each completed slice
goal-hint: Provide the delivery plan to execute, or reference the plan file/location
keywords:
  - tdd
  - test-driven-development
  - red-green-refactor
  - software-delivery
  - implementation
  - slicing
when-to-use:
  - when a software delivery plan exists and needs to be executed slice-by-slice using TDD
  - when you want automated Red-Green-Refactor cycles with human sign-off after each slice
when-not-to-use:
  - when no delivery plan exists yet (use software-plan-iterative first)
  - when the task does not follow TDD methodology
  - when you want to implement without test-first discipline
---

The workflow processes a delivery plan slice-by-slice in a single run. For each slice:

1. **SelectSlice** — the `software-slice-selector` agent reads the plan and prior context to pick the next unfinished slice, or signals Done when all slices are complete

2. **Red phase** — the `software-test-creator` agent writes failing tests for the slice; the `software-test-runner` agent executes the relevant tests to confirm they actually fail; the `software-test-reviewer` agent evaluates test quality and can loop the creator back for revisions

3. **Green phase** — the `software-implementor` agent writes the minimum code to pass; the `software-test-runner` agent executes the relevant tests to verify the implementation; if tests fail the loop sends control back to the implementor

4. **Refactor phase** — the `software-refactorer` agent improves code quality; the `software-test-runner` agent executes the relevant tests to verify no regressions; if tests fail the loop sends control back to the refactorer

5. **Human review** — a structured interview lets the human accept the slice or send it back to the Red phase with revision notes

6. **Loop or finish** — the `software-slice-selector` agent checks for remaining slices and either loops back or ends the workflow

Test execution uses a `software-test-runner` agent instead of a static shell script, allowing it to inspect the project structure, determine the appropriate test framework, and run only the tests relevant to the current slice rather than the full test suite.

Agents use tool-based context passing (`workflow_set_context` / `workflow_get_context`, `workflow_get_output`) rather than prompt interpolation to share state across stages — the slice selector stores the active slice details, the test creator stores the scoped test command and files, and the completion checker records finished slices. This keeps prompts concise across many iterations. Labeled edges provide structured routing via `workflow_set_route` for all agent-driven branch decisions

```dot
digraph software_delivery_tdd {
  Start -> SelectSlice

  SelectSlice [agent="software-slice-selector", prompt-ref="#select-slice-prompt"]
  SelectSlice -> CreateTests  [label="Continue"]
  SelectSlice -> End          [label="Done"]

  subgraph red_phase {
    node [class="red-phase"]

    CreateTests [agent="software-test-creator", prompt-ref="#create-tests-prompt", max_retries=3]
    CreateTests -> RunTestsRed

    RunTestsRed [agent="software-test-runner", prompt-ref="#run-tests-prompt"]
    RunTestsRed -> ReviewTests

    ReviewTests [agent="software-test-reviewer", prompt-ref="#review-tests-prompt"]
    ReviewTests -> Implement     [label="Accept"]
    ReviewTests -> CreateTests   [label="Revise"]
  }

  subgraph green_phase {
    node [class="green-phase"]

    Implement [agent="software-implementor", prompt-ref="#implement-prompt", max_retries=3]
    Implement -> RunTestsGreen

    RunTestsGreen [agent="software-test-runner", prompt-ref="#run-tests-prompt"]
    RunTestsGreen -> Refactor    [label="Pass"]
    RunTestsGreen -> Implement   [label="Fail"]
  }

  subgraph refactor_phase {
    node [class="refactor-phase"]

    Refactor [agent="software-refactorer", prompt-ref="#refactor-prompt", max_retries=3]
    Refactor -> RunTestsRefactor

    RunTestsRefactor [agent="software-test-runner", prompt-ref="#run-tests-prompt"]
    RunTestsRefactor -> HumanReview  [label="Pass"]
    RunTestsRefactor -> Refactor     [label="Fail"]
  }

  HumanReview [interview-ref="#slice-review"]
  HumanReview -> CheckRemaining  [label="Accept"]
  HumanReview -> CreateTests     [label="Revise"]

  CheckRemaining [agent="software-slice-selector", prompt-ref="#check-remaining-prompt"]
  CheckRemaining -> SelectSlice  [label="Continue"]
  CheckRemaining -> End          [label="Done"]
}
```

```text #run-tests-prompt
Run the tests relevant to the current slice.

Use workflow_get_context to read the test command (key "slice.test_command"), test files (key "slice.test_files"), and slice scope (key "slice.scope"). Execute the test command. If no test command is stored, inspect the project structure, build files, and test conventions to determine the correct scoped test command. Run only the tests scoped to this slice — do not run the full test suite.

Report the test results clearly, including which tests passed and which failed along with failure details. If this node has outgoing labeled edges (Pass/Fail), choose the appropriate branch based on the results.
```

```text #select-slice-prompt
Review the delivery plan and select the next unfinished slice of work for implementation.

The delivery plan goal is: $goal

Use workflow_get_context to check for any previously completed slices (key "completed_slices"). Identify the highest-priority slice that has not yet been completed. Present the slice details including its scope, acceptance criteria, and any dependencies.

After selecting a slice, use workflow_set_context to store:
- key "current_slice" — the slice name or identifier
- key "slice.scope" — a concise description of what this slice covers
- key "slice.acceptance_criteria" — the acceptance criteria for this slice
- key "slice.packages" — the packages, crates, modules, or directories involved (so downstream agents can scope their work and tests)

If all slices have been completed, choose the Done branch.
```

```text #create-tests-prompt
Write failing tests for the current slice of work (Red step of TDD).

Use workflow_get_context to read the current slice details (keys "current_slice", "slice.scope", "slice.acceptance_criteria", "slice.packages").

Check for reviewer feedback from a previous iteration using workflow_get_output. If feedback is present, use it to revise the tests instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use workflow_get_context with key "human.feedback" to check for human revision notes and incorporate those as well.

Write tests that:
- Cover the acceptance criteria for this slice
- Are specific and focused on the slice's scope
- Follow existing test conventions in the codebase
- Will fail because the implementation does not exist yet

After writing tests, use workflow_set_context to store:
- key "slice.test_files" — the list of test file paths created or modified
- key "slice.test_command" — the specific command to run only these tests (e.g. "cargo test -p crate-name", "pytest tests/test_specific.py", "npm test -- --testPathPattern=specific")

The tests will be executed automatically in the next step to confirm they fail as expected.
```

```text #review-tests-prompt
Review the tests written for the current slice of work.

Use workflow_get_context to read the slice details (keys "current_slice", "slice.scope", "slice.acceptance_criteria", "slice.test_files"). Use workflow_get_output to see the test execution results from the Red phase run.

Evaluate:
- Do the tests adequately cover the acceptance criteria for this slice?
- Are the tests well-structured, readable, and maintainable?
- Do the tests follow existing conventions in the codebase?
- Are edge cases and error conditions covered appropriately?
- Did the tests fail as expected (confirming they test unimplemented behavior)?
- Will these tests meaningfully validate the implementation?

If the tests are acceptable, choose the Accept branch.
If the tests need changes, choose the Revise branch and provide specific feedback.
```

```text #implement-prompt
Implement the minimum code necessary to make all tests pass (Green step of TDD).

Use workflow_get_context to read the slice details (keys "current_slice", "slice.scope", "slice.acceptance_criteria", "slice.packages", "slice.test_files"). Use workflow_get_output to check for any feedback from a failed test run or previous iteration. If present, fix the issues identified while keeping all tests passing.

Write code that:
- Makes all existing tests pass
- Is minimal — do not add functionality beyond what the tests require
- Follows existing code conventions in the codebase
- Is scoped to the packages and directories listed in "slice.packages"

After writing code, the tests will be executed automatically in the next step to confirm they pass.
```

```text #refactor-prompt
Refactor the implementation while keeping all tests passing (Refactor step of TDD).

Use workflow_get_context to read the slice details (keys "current_slice", "slice.scope", "slice.packages", "slice.test_files"). Use workflow_get_output to check for feedback from a failed refactor verification. If test failures were reported, fix them before continuing with refactoring.

Improve the code quality by:
- Eliminating duplication
- Improving naming and readability
- Simplifying complex logic
- Ensuring the code follows existing patterns and conventions
- Keeping changes scoped to the packages and directories listed in "slice.packages"
- Keeping all tests passing after every change

After refactoring, the tests will be executed automatically to verify no regressions.
```

```text #check-remaining-prompt
Check the delivery plan for any remaining unfinished slices.

Use workflow_get_context to read the current slice (key "current_slice") and previously completed slices (key "completed_slices"). Mark the current slice as completed by using workflow_set_context to update key "completed_slices" — append the current slice name to the list of completed slices.

Compare the completed slices against the full delivery plan. If there are more slices to implement, choose the Continue branch. If all slices have been completed, choose the Done branch.
```

```yaml #slice-review
preamble: |
  The TDD cycle (Red-Green-Refactor) for this slice is complete and all tests are passing.
  Please review the tests, implementation, and refactored code before proceeding.

questions:
  - header: Decision
    question: Is the completed slice acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
    finish-if: Accept

  - header: Revision Notes
    question: What specific changes should be made?
    type: freeform
    show-if: "human.decision == Revise"
    store: human.feedback
```
