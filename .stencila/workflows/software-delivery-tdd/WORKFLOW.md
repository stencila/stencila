---
name: software-delivery-tdd
description: Execute a software delivery plan slice-by-slice using test-driven development with Red-Green-Refactor cycles, agent-driven scoped test execution, iterative review, and human approval after each completed slice
goal-hint: Which delivery plan should be executed? Provide the plan content or reference the plan file path
keywords:
  - tdd
  - test-driven-development
  - red-green-refactor
  - software-delivery
  - implementation
  - slicing
  - human-in-the-loop
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

2. **Red phase** — the `software-test-creator` agent writes failing tests for the slice; the `software-test-executor` agent executes the relevant tests to confirm they actually fail; the `software-test-reviewer` agent evaluates test quality and can loop the creator back for revisions

3. **Green phase** — the `software-implementor` agent writes the minimum code to pass; the `software-test-executor` agent executes the relevant tests to verify the implementation; if tests fail the loop sends control back to the implementor

4. **Refactor phase** — the `software-refactorer` agent improves code quality; the `software-test-executor` agent executes the relevant tests to verify no regressions; if tests fail the loop sends control back to the refactorer

5. **Human review** — a structured interview lets the human accept the slice or send it back to the Red phase with revision notes

6. **Loop or finish** — the `software-slice-checker` agent marks the slice done and checks for remaining slices, then either loops back or ends the workflow

Test execution uses a `software-test-executor` agent instead of a static shell script, allowing it to inspect the project structure, determine the appropriate test framework, and run only the tests relevant to the current slice rather than the full test suite.

Stages share state via `workflow_set_context` / `workflow_get_context` and `workflow_get_output` rather than prompt interpolation — context keys hold the active slice details, scoped test metadata, and completed slice tracking. This keeps prompts concise across many iterations. Revision loops rely on `workflow_get_output` as the feedback channel: the test-reviewer's output text is the feedback that the test-creator reads on the next iteration, and failed test-execution output is the feedback that the implementor and refactorer read. Labeled edges provide structured routing via `workflow_set_route` for all agent-driven branch decisions.

```dot
digraph software_delivery_tdd {
  Start -> SelectSlice

  SelectSlice [agent="software-slice-selector", prompt-ref="#select-slice-prompt", context-writable=true]
  SelectSlice -> CreateTests  [label="Continue"]
  SelectSlice -> End          [label="Done"]

  subgraph red_phase {
    node [class="red-phase"]

    CreateTests [agent="software-test-creator", prompt-ref="#create-tests-prompt", max_retries=3, context-writable=true]
    CreateTests -> RunTestsRed

    RunTestsRed [agent="software-test-executor", prompt-ref="#run-tests-prompt"]
    RunTestsRed -> ReviewTests

    ReviewTests [agent="software-test-reviewer", prompt-ref="#review-tests-prompt"]
    ReviewTests -> Implement     [label="Accept"]
    ReviewTests -> CreateTests   [label="Revise"]
  }

  subgraph green_phase {
    node [class="green-phase"]

    Implement [agent="software-implementor", prompt-ref="#implement-prompt", max_retries=3]
    Implement -> RunTestsGreen

    RunTestsGreen [agent="software-test-executor", prompt-ref="#run-tests-prompt", max_retries=3]
    RunTestsGreen -> Refactor    [label="Pass"]
    RunTestsGreen -> Implement   [label="Fail"]
  }

  subgraph refactor_phase {
    node [class="refactor-phase"]

    Refactor [agent="software-refactorer", prompt-ref="#refactor-prompt", max_retries=3]
    Refactor -> RunTestsRefactor

    RunTestsRefactor [agent="software-test-executor", prompt-ref="#run-tests-prompt", max_retries=3]
    RunTestsRefactor -> HumanReview  [label="Pass"]
    RunTestsRefactor -> Refactor     [label="Fail"]
  }

  HumanReview [interview-ref="#slice-review"]
  HumanReview -> CheckRemaining  [label="Accept"]
  HumanReview -> CreateTests     [label="Revise"]

  CheckRemaining [agent="software-slice-checker", prompt-ref="#check-slices-prompt", context-writable=true]
  CheckRemaining -> SelectSlice  [label="Continue"]
  CheckRemaining -> End          [label="Done"]
}
```

```text #select-slice-prompt
Select the next unfinished slice of work from the delivery plan.

The delivery plan goal is: $goal

Step 1 — read workflow state:
  Use workflow_get_context to read key "completed_slices" (the list of previously completed slice names).

Step 2 — delegate to the slice-selection skill:
  Pass the completed slices list and the plan goal to the slice-selection skill. The skill will read the delivery plan, identify the next unfinished slice, and report its details (name, scope, acceptance criteria, packages) or signal that all slices are complete.

Step 3 — store the skill's outputs into workflow context:
  If a slice was selected, use workflow_set_context to store:
  - key "current_slice" — the slice name or identifier
  - key "slice.scope" — the scope description
  - key "slice.acceptance_criteria" — the acceptance criteria
  - key "slice.packages" — the packages, crates, modules, or directories involved

Step 4 — route:
  If a slice was selected, call workflow_set_route with label "Continue".
  If all slices are complete, call workflow_set_route with label "Done".
```

```text #create-tests-prompt
Write failing tests for the current slice of work (Red step of TDD).

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "current_slice" — the slice name
  - key "slice.scope" — what the slice covers
  - key "slice.acceptance_criteria" — the criteria the tests must verify
  - key "slice.packages" — the packages or directories involved

  Check for reviewer feedback from a previous iteration using workflow_get_output.
  Also use workflow_get_context with key "human.feedback" to check for human revision notes.

Step 2 — delegate to the test-creation skill:
  Pass the slice name, scope, acceptance criteria, and target packages as inputs.
  If reviewer feedback or human revision notes exist, pass those as the revision feedback input.
  The skill will discover codebase test conventions, write the tests, verify they fail as expected,
  and report the test file paths and test command.

Step 3 — store the skill's outputs into workflow context:
  Use workflow_set_context to store:
  - key "slice.test_files" — the list of test file paths created or modified
  - key "slice.test_command" — the specific command to run only these tests

The tests will be executed automatically in the next step to confirm they fail as expected.
```

```text #review-tests-prompt
Review the tests written for the current slice of work.

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "current_slice" — the slice name
  - key "slice.scope" — what the slice covers
  - key "slice.acceptance_criteria" — the criteria the tests must verify
  - key "slice.packages" — the packages or directories involved
  - key "slice.test_files" — the test file paths to review
  - key "slice.test_command" — the command used to run the tests

  Use workflow_get_output to read the test execution results from the Red phase run.

Step 2 — delegate to the test-review skill:
  Pass the slice name, scope, acceptance criteria, target packages, test files, test command,
  and test execution results as inputs. The skill will independently discover codebase conventions,
  evaluate the tests across all quality dimensions, and produce a structured review report
  with an Accept or Revise recommendation.

Step 3 — route based on the skill's recommendation:
  If the skill recommends Accept, call workflow_set_route with label "Accept".
  If the skill recommends Revise, call workflow_set_route with label "Revise" — the review
  report serves as feedback for the test-creation agent in the next iteration.
```

```text #run-tests-prompt
Run the tests relevant to the current slice.

Delivery plan goal: $goal

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "slice.test_command" — the test command to run
  - key "slice.test_files" — the test file paths
  - key "slice.scope" — what the slice covers
  - key "slice.packages" — the packages or directories involved
  - key "current_slice" — the slice name (for the report header)

Step 2 — delegate to the test-execution skill:
  Pass the test command, test files, slice scope, target packages, and slice name as inputs.
  If no test command is stored, the skill will discover the correct scoped test command.
  The skill will execute the tests, parse the output, and report a structured pass/fail result.

Step 3 — route based on the skill's result:
  If this node has outgoing labeled edges: call workflow_set_route with label "Pass" if all tests passed, or "Fail" if any test failed.
```

```text #implement-prompt
Implement the minimum code necessary to make all tests pass (Green step of TDD).

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "current_slice" — the slice name
  - key "slice.scope" — what the slice covers
  - key "slice.acceptance_criteria" — the criteria being implemented
  - key "slice.packages" — the packages or directories to work in
  - key "slice.test_files" — the test files that must pass

  Use workflow_get_output to check for feedback from a failed test run or previous iteration.

Step 2 — implement:
  Write the minimum code to make all tests pass. If feedback from a prior failure is present,
  fix the issues identified while keeping all tests passing.

  Requirements:
  - Make all existing tests pass
  - Be minimal — do not add functionality beyond what the tests require
  - Follow existing code conventions in the codebase
  - Scope changes to the packages and directories listed in "slice.packages"

The tests will be executed automatically in the next step to confirm they pass.
```

```text #refactor-prompt
Refactor the implementation while keeping all tests passing (Refactor step of TDD).

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "current_slice" — the slice name
  - key "slice.scope" — what the slice covers
  - key "slice.packages" — the packages or directories to work in
  - key "slice.test_files" — the test files that must keep passing

  Use workflow_get_output to check for feedback from a failed refactor verification.

Step 2 — refactor:
  If test failures were reported in prior feedback, fix them first. Then improve the code:
  - Eliminate duplication
  - Improve naming and readability
  - Simplify complex logic
  - Ensure the code follows existing patterns and conventions
  - Keep changes scoped to the packages and directories in "slice.packages"
  - Keep all tests passing after every change

The tests will be executed automatically to verify no regressions.
```

```text #check-slices-prompt
Check the delivery plan for any remaining unfinished slices.

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "current_slice" — the slice that was just completed
  - key "completed_slices" — the list of previously completed slices

Step 2 — delegate to the slice-completion skill:
  Pass the current slice name and the completed slices list as inputs. The skill will append
  the current slice to the completed list, compare against the full delivery plan, and report
  the updated completed slices list and whether more work remains.

Step 3 — store the skill's outputs into workflow context:
  Use workflow_set_context to store:
  - key "completed_slices" — the updated completed slices list returned by the skill

Step 4 — route based on the skill's result:
  If more slices remain, call workflow_set_route with label "Continue".
  If all slices are complete, call workflow_set_route with label "Done".
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
