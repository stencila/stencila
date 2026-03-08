---
name: test-kitchen-sink
description: Combined test exercising linear, conditional, parallel, human, shell, and retry patterns
goal: Verify integrated pipeline execution
modelStylesheet: |
  * { llm_model: claude-sonnet-4-5; }
  .analysis { reasoning_effort: high; }
---

A "kitchen sink" integration test that combines multiple pipeline concepts in one workflow: shell command nodes, parallel fan-out/fan-in, conditional branching, human-in-the-loop gates, `max_retries`, `class=` with model stylesheet, edge conditions, and edge labels with accelerator keys. Targets the §11.12 parity matrix requirement that a 10+ node pipeline completes without errors.

```dot
digraph Workflow {
    Start -> RunSetup -> FanOut
    FanOut -> BranchA
    FanOut -> BranchB
    BranchA -> Merge
    BranchB -> Merge
    Merge -> CheckQuality
    CheckQuality -> ReviewResult  [label="Pass", condition="outcome=success"]
    CheckQuality -> BranchA       [label="Retry", condition="outcome!=success"]
    ReviewResult -> Summarize     [label="[A] Approve"]
    ReviewResult -> BranchA       [label="[R] Redo"]
    Summarize -> End

    RunSetup      [cmd="echo setup-complete"]
    BranchA       [prompt="Reply with ONLY the word: alpha", class="analysis"]
    BranchB       [prompt="Reply with ONLY the word: beta", class="analysis", max-retries=1]
    Merge         [prompt="Combine the words from parallel branches into a comma-separated list. Reply with ONLY the list."]
    ReviewResult  [ask="Approve the merged result?"]
    Summarize     [prompt="Reply with ONLY the word: done"]
}
```
