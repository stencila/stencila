---
name: test-count-to-goal
description: Test workflow that counts up to a user-specified goal number using a loop
---

A test of looping with conditional branching and `$goal` expansion. The workflow starts at 1 and repeatedly increments until it reaches `$goal`.

```dot
digraph Workflow {
  Start -> Init

  Init [prompt="Reply with just the number: 1"]
  Init -> Step

  Step [prompt="If $last_output equals $goal, the counting is complete — choose Done. Otherwise add one to $last_output and reply with ONLY the new number."]
  Step -> Verify [label="Done"]
  Step -> Step   [label="Continue"]

  Verify [prompt="Does '$last_output' confirm the count reached '$goal'? Verify the result and choose Pass or Fail."]
  Verify -> End  [label="Pass"]
  Verify -> Fail [label="Fail"]
}
```
