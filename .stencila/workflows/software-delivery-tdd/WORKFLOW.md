---
name: software-delivery-tdd
description: Execute a software delivery plan using test-driven development with Red-Green-Refactor cycles, agent-driven scoped test execution, iterative review, and human approval after each completed slice or combined slice batch
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

The workflow processes a delivery plan in successive execution units during a single run. Each execution unit may correspond to one plan-authored slice or to several adjacent compatible slices combined by the selector to normalize overly fine-grained plans. For each selected unit:

1. **SelectSlice** — the `software-slice-selector` agent marks the just-completed slice or slice batch (if any), updates the completed slices list, reads the plan and prior context to pick the next unfinished execution unit, and reports whether more slices remain — or signals Done when all slices are complete

2. **Red phase** — the `software-test-creator` agent writes failing tests for the slice; the `software-test-executor` agent executes the relevant tests to confirm they actually fail; the `software-test-reviewer` agent evaluates test quality and can loop the creator back for revisions

3. **Green phase** — the `software-implementor` agent writes the minimum code to pass; the `software-test-executor` agent executes the relevant tests to verify the implementation; if tests fail the loop sends control back to the implementor

4. **Refactor phase** — the `software-code-refactorer` agent improves code quality; the `software-test-executor` agent executes the relevant tests to verify no regressions; if tests fail the loop sends control back to the refactorer

5. **Human review** — a structured interview lets the human accept the execution unit, accept and commit it, or send it back to the Red phase with revision notes; choosing "Accept and Commit" routes through a CommitSlice agent that stages and commits the changes before continuing; on acceptance (with or without commit), control returns to SelectSlice which handles both completion tracking and next-unit selection

Test execution uses a `software-test-executor` agent instead of a static shell script, allowing it to inspect the project structure, determine the appropriate test framework, and run only the tests relevant to the current slice rather than the full test suite.

All agent nodes use `fidelity="full"` with explicit `thread-id` values so that each agent's LLM session is reused across iterations. This avoids the startup cost of re-reading files, re-discovering codebase conventions, and re-parsing the delivery plan on every loop. Each agent role gets its own thread (`slice-selector`, `test-creator`, `test-executor`, `test-reviewer`, `implementor`, `refactorer`), with the three test-executor nodes sharing a single `test-executor` thread since they perform the same job in different phases. The one-shot `CommitSlice` node uses the default fidelity since it has no iteration overhead. A graph-wide `max-session-turns` default of 10 caps context growth on lightweight nodes, while heavy-context nodes (`CreateTests`, `Implement`, `Refactor`) override this to 5 since they read and write many files per turn and accumulate context faster.

Stages share state via `workflow_set_context` / `workflow_get_context` and `workflow_get_output` rather than prompt interpolation — context keys hold the active slice details, scoped test metadata, and completed slice tracking. The existing context keys are retained even when the selected work corresponds to several combined plan slices: in that case `current_slice`, `slice.scope`, `slice.acceptance_criteria`, and `slice.packages` describe the selected execution unit, while `completed_slices` continues to track the underlying plan slice identifiers. This keeps prompts concise across many iterations. Revision loops rely on `workflow_get_output` as the feedback channel: the test-reviewer's output text is the feedback that the test-creator reads on the next iteration, and failed test-execution output is the feedback that the implementor and refactorer read. Labeled edges provide structured routing via `workflow_set_route` for all agent-driven branch decisions.

```dot
digraph software_delivery_tdd {
  node [max-session-turns="10"]

  Start -> SelectSlice

  SelectSlice [
    agent="software-slice-selector",
    prompt-ref="#select-slice-prompt",
    context-writable=true,
    fidelity="full",
    thread-id="slice-selector"
  ]
  SelectSlice -> CreateTests  [label="Continue"]
  SelectSlice -> End          [label="Done"]

  subgraph red_phase {
    node [class="red-phase"]

    CreateTests [
      agent="software-test-creator",
      prompt-ref="#create-tests-prompt",
      context-writable=true,
      fidelity="full",
      thread-id="test-creator",
      max-session-turns="5"
    ]
    CreateTests -> RunTestsRed

    RunTestsRed [
      agent="software-test-executor",
      prompt-ref="#run-tests-prompt",
      fidelity="full",
      thread-id="test-executor"
    ]
    RunTestsRed -> ReviewTests

    ReviewTests [
      agent="software-test-reviewer",
      prompt-ref="#review-tests-prompt",
      fidelity="full",
      thread-id="test-reviewer"
    ]
    ReviewTests -> Implement     [label="Accept"]
    ReviewTests -> CreateTests   [label="Revise"]
  }

  subgraph green_phase {
    node [class="green-phase"]

    Implement [
      agent="software-implementor",
      prompt-ref="#implement-prompt",
      fidelity="full",
      thread-id="implementor",
      max-session-turns="5"
    ]
    Implement -> RunTestsGreen

    RunTestsGreen [
      agent="software-test-executor",
      prompt-ref="#run-tests-prompt",
      fidelity="full",
      thread-id="test-executor"
    ]
    RunTestsGreen -> Refactor    [label="Pass"]
    RunTestsGreen -> Implement   [label="Fail"]
  }

  subgraph refactor_phase {
    node [class="refactor-phase"]

    Refactor [
      agent="software-code-refactorer",
      prompt-ref="#refactor-prompt",
      fidelity="full",
      thread-id="refactorer",
      max-session-turns="5"
    ]
    Refactor -> RunTestsRefactor

    RunTestsRefactor [
      agent="software-test-executor",
      prompt-ref="#run-tests-prompt",
      fidelity="full",
      thread-id="test-executor"
    ]
    RunTestsRefactor -> HumanReview  [label="Pass"]
    RunTestsRefactor -> Refactor     [label="Fail"]
  }

  HumanReview [interview-ref="#slice-review"]
  HumanReview -> SelectSlice  [label="Accept"]
  HumanReview -> CommitSlice  [label="Accept and Commit"]
  HumanReview -> CreateTests  [label="Revise"]

  CommitSlice [
    agent="general",
    prompt-ref="#commit-slice-prompt"
  ]
  CommitSlice -> SelectSlice
}
```

```text #select-slice-prompt
Mark the just-completed slice or slice batch (if any) and select the next unfinished execution unit from the delivery plan.

The delivery plan goal is: $goal

Note on "current_slice": this key serves a dual role. When entering SelectSlice after
HumanReview acceptance it holds the most recently completed execution unit name. After
SelectSlice stores a newly selected execution unit it holds the next unit to work on. On the
first invocation it is empty. The selected unit may represent one plan slice or a combined
batch of adjacent compatible plan slices.

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "current_slice" — the most recently selected execution unit (treat as just-completed when
    re-entering after acceptance; empty on first invocation)
  - key "completed_slices" — the list of previously completed slice names

Step 2 — delegate to the slice-selection skill:
  Pass the just-completed slice or execution unit name (from "current_slice", may be empty on first invocation),
  the completed slices list, and the plan goal to the slice-selection skill. The skill will:
  - Append the just-completed slice or the underlying slices represented by a just-completed combined unit to the completed list (if provided)
  - Read the delivery plan and identify the next unfinished execution unit, combining adjacent compatible slices when appropriate to normalize overly narrow planning granularity
  - Report the updated completed list, selected execution unit details (name, included slices, scope, acceptance criteria,
    packages), and whether more slices remain — or signal that all slices are complete

Step 3 — clear stale slice-scoped context:
  Use workflow_set_context to clear transient state from the previous slice so it cannot
  leak into the next one:
  - key "human.feedback" — set to ""
  - key "slice.test_files" — set to ""
  - key "slice.test_command" — set to ""

Step 4 — store the skill's outputs into workflow context:
  Use workflow_set_context to store:
  - key "completed_slices" — the updated completed slices list returned by the skill
  If a slice or slice batch was selected, also store:
  - key "current_slice" — the selected execution unit name or identifier
  - key "slice.scope" — the scope description
  - key "slice.acceptance_criteria" — the acceptance criteria
  - key "slice.packages" — the packages, crates, modules, or directories involved

Step 5 — route:
  If a slice or slice batch was selected, call workflow_set_route with label "Continue".
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

```yaml #slice-review
preamble: |
  The TDD cycle (Red-Green-Refactor) for this slice is complete and all tests are passing.
  Please review the tests, implementation, and refactored code before proceeding.

questions:
  - header: Decision
    question: Is the completed slice acceptable?
    type: single-select
    options:
      - label: Accept and Commit
      - label: Accept
      - label: Revise
    store: human.decision

  - header: Revision Notes
    question: What specific changes should be made?
    type: freeform
    show-if: "human.decision == Revise"
    store: human.feedback
```

```text #commit-slice-prompt
Commit the changes from the completed TDD slice.

Step 1 — read workflow state:
  Use workflow_get_context to read:
  - key "current_slice" — the slice name
  - key "slice.scope" — what the slice covers
  - key "slice.packages" — the packages or directories involved

Step 2 — stage changes:
  Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
  Stage the files related to this slice. Use "slice.packages" as a guide for which paths
  are most relevant, but include other changed files (e.g., test fixtures, configuration,
  shared modules) when they are clearly part of this slice's work. Use your judgement —
  avoid staging unrelated changes that happened to be in the working tree.

Step 3 — commit:
  Compose a commit message based on the slice name, scope, and the actual changes staged.
  Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
  project's commit message conventions and follow them. Also check for any commit message
  instructions in the system prompt or prior context and apply those.
  Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue to the next slice regardless.
```
