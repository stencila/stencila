---
name: test-human-gates
description: Test workflow with multiple human-in-the-loop gates for exercising the CliInterviewer
---

# Test Human Gates

This workflow exercises various `wait.human` gate patterns via the `CliInterviewer`. It has no LLM calls — every non-structural node is a human gate — so it can run instantly and offline.

```dot
digraph Workflow {
    Start -> Approve

    // --- Binary choice (yes / no style) ---------------------------------
    // Two outgoing edges simulate a yes/no gate.
    Approve [ask="Deploy to production?"]
    Approve -> Picked     [label="Yes, deploy"]
    Approve -> Rejected   [label="No, abort"]

    Rejected [shape=invtriangle]

    // --- Three-way choice ------------------------------------------------
    // Three edges test the dialoguer Select prompt with more options.
    Picked [ask="Pick an environment"]
    Picked -> ReviewChanges  [label="Staging"]
    Picked -> ReviewChanges  [label="Production"]
    Picked -> ReviewChanges  [label="Development"]

    // --- Edge labels without explicit accelerator keys -------------------
    // The engine derives keys from the first letter of each label.
    ReviewChanges [ask="How do the changes look?"]
    ReviewChanges -> Confirm  [label="Good"]
    ReviewChanges -> Confirm  [label="Needs work"]
    ReviewChanges -> Confirm  [label="Terrible"]

    // --- Single-choice gate (auto-continue) ------------------------------
    // Only one outgoing edge — the user must accept it.
    Confirm [ask="Press Enter to finish"]
    Confirm -> End [label="Continue"]
}
```
