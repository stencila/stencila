---
name: test-kitchen-sink
description: Combined test exercising linear, conditional, parallel, human, shell, and retry patterns
goal: Verify integrated pipeline execution
overrides: |
  * { model: claude-sonnet-4-5; }
  .analysis { reasoning_effort: high; }
---

A "kitchen sink" integration test that combines multiple pipeline concepts in one workflow: shell command nodes, parallel fan-out/fan-in, conditional branching, human-in-the-loop gates, `max_retries`, `class=` with model stylesheet, edge conditions, and edge labels with accelerator keys. Targets the §11.12 parity matrix requirement that a 10+ node pipeline completes without errors.

```dot
digraph Workflow {
  Start -> RunSetup

  RunSetup [cmd="echo setup-complete"]
  RunSetup -> FanOut

  FanOut [label="Fan out"]
  FanOut -> BranchA
  FanOut -> BranchB

  BranchA [prompt="Reply with ONLY the word: alpha", class="analysis"]
  BranchA -> Merge

  BranchB [prompt="Reply with ONLY the word: beta", class="analysis", max_retries=1]
  BranchB -> Merge

  Merge [prompt="Combine the words from parallel branches into a comma-separated list. Reply with ONLY the list."]
  Merge -> CheckQuality

  CheckQuality [label="Check quality"]
  CheckQuality -> ReviewResult  [label="Pass", condition="outcome=success"]
  CheckQuality -> BranchA       [label="Retry", condition="outcome!=success"]

  ReviewResult [ask="Approve the merged result?"]
  ReviewResult -> Summarize     [label="[A] Approve"]
  ReviewResult -> BranchA       [label="[R] Redo"]

  Summarize [prompt="Reply with ONLY the word: done"]
  Summarize -> End
}
```
