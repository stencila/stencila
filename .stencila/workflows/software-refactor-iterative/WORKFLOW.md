---
name: software-refactor-iterative
description: Iteratively refactor part or all of a software project using the software-code-refactorer and software-code-reviewer agents, with test verification after each pass and human approval before completion
goal-hint: What code should be refactored? Describe the scope (files, modules, packages) and any specific quality improvements you want
keywords:
  - refactoring
  - code quality
  - code cleanup
  - iterative
  - review
  - human-in-the-loop
  - safe transformation
  - conventions
when-to-use:
  - when existing production code needs quality improvements such as duplication removal, naming cleanup, complexity reduction, or convention alignment
  - when you want agent-driven refactoring with automated code review and human approval cycles
  - when refactoring scope spans part or all of a project and needs iterative refinement with test verification
when-not-to-use:
  - when writing new features or production code (use software-delivery-tdd)
  - when the codebase has no tests — refactoring without test coverage is unsafe
  - when the task is a one-pass code review without making changes (use the software-code-reviewer agent directly)
  - when the task involves design, planning, or test creation rather than refactoring existing code
---

This workflow first uses the `software-code-refactorer` agent to apply safe transformations, then the `software-test-executor` agent runs scoped tests to verify no regressions. If tests fail, control loops back to the refactorer with the failure output as feedback. If tests pass, the `software-code-reviewer` agent evaluates the refactored code and chooses Accept or Revise — on Revise its response provides specific feedback for the next refactoring pass. After the reviewer accepts, a structured human review interview lets the user accept or send the changes back for further revision with specific notes. The `Refactor` node uses `workflow_get_output` to retrieve reviewer or test-failure feedback and `workflow_get_context` with key `human.feedback` to retrieve human revision notes. All iterating agent nodes use `fidelity="full"` with explicit `thread-id` values so each agent's LLM session is reused across iterations, avoiding the cost of re-reading files and re-discovering conventions on every loop. A graph-wide `max-session-turns` default of 10 caps context growth, with the heavy-context `Refactor` node overridden to 5.

```dot
digraph software_refactor_iterative {
  node [max-session-turns="10"]

  Start -> Refactor

  Refactor [
    agent="software-code-refactorer",
    prompt-ref="#refactor-prompt",
    fidelity="full",
    thread-id="refactorer",
    max-session-turns="5"
  ]
  Refactor -> RunTests

  RunTests [
    agent="software-test-executor",
    prompt-ref="#run-tests-prompt",
    fidelity="full",
    thread-id="test-executor"
  ]
  RunTests -> Review     [label="Pass"]
  RunTests -> Refactor   [label="Fail"]

  Review [
    agent="software-code-reviewer",
    prompt-ref="#review-prompt",
    fidelity="full",
    thread-id="reviewer"
  ]
  Review -> HumanReview  [label="Accept"]
  Review -> Refactor     [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> End      [label="Accept"]
  HumanReview -> Refactor [label="Revise"]
}
```

```text #refactor-prompt
Refactor the code for the goal:

$goal

Before starting, use workflow_get_output to check for feedback from a previous iteration. This may be:
- Code review feedback from the software-code-reviewer (if the reviewer requested revisions)
- Test failure output from the software-test-executor (if tests broke after a prior refactor)

If feedback is present, use it to address the specific issues identified rather than starting over. If you disagree with a specific review finding, you may skip it but note your reasoning.

Also use workflow_get_context with key "human.feedback" to check for human revision notes and incorporate those as well.

Requirements:
- Discover and follow existing codebase conventions
- Apply safe transformations: duplication removal, naming improvements, complexity reduction, convention alignment
- Keep all existing tests passing after every change
- Scope changes to the files, modules, or packages described in the goal
- Do not add new features or change external behavior
- Verify the code compiles and tests pass after your changes
```

```text #run-tests-prompt
Run the tests relevant to the refactored code.

Refactoring goal: $goal

Step 1 — determine test scope:
  Examine the refactoring goal to identify which packages, modules, or directories were affected.
  Discover the appropriate test command for the project and scope it to the relevant areas.

Step 2 — execute tests:
  Run the scoped tests. If no scoped test command is obvious, run the project's full test suite.

Step 3 — route based on results:
  If this node has outgoing labeled edges: call workflow_set_route with label "Pass" if all
  tests passed, or "Fail" if any test failed. The failure output serves as feedback for the
  refactorer in the next iteration.
```

```text #review-prompt
Review the refactored code for the goal:

$goal

Step 1 — examine the changes:
  Read the files that were modified by the refactoring. Use the goal description to identify
  the scope of files, modules, or packages to review.

Step 2 — evaluate the refactoring:
  Assess the changes across these dimensions:
  - Correctness: Do the changes preserve existing behavior? Are there any subtle bugs introduced?
  - Quality: Is duplication reduced? Is complexity improved? Are names clear and consistent?
  - Conventions: Do the changes follow the codebase's existing patterns and style?
  - Security: Are there any security concerns introduced by the refactoring?
  - Maintainability: Is the refactored code easier to understand and modify?

Step 3 — route:
  If the refactoring is acceptable, choose the Accept branch.
  If the refactoring needs changes, choose the Revise branch and provide specific, actionable
  feedback in your response describing what should be improved.
```

```yaml #human-review-interview
preamble: |
  The software-code-reviewer agent has approved the refactoring and all tests are passing.
  Please review the changes and decide whether to accept or send them back for revision.

questions:
  - header: Decision
    question: Is the refactoring acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
    finish-if: Accept

  - header: Revision Notes
    question: What specific changes or improvements should be made?
    store: human.feedback
```
