---
name: test-refs
description: Test prompt-ref, shell-ref, and ask-ref content block references in workflows
goal: Verify that workflow content block references resolve correctly
---

This workflow tests reusable multiline prompt, shell, and ask content stored in fenced code blocks with ids and referenced from the DOT pipeline.

```dot
digraph test_refs {
    Start -> Create -> Verify
    Verify -> HumanReview [label="Pass", condition="context.last_output=ok"]
    Verify -> Fail        [label="Fail", condition="context.last_output!=ok"]
    HumanReview -> End    [label="Accept"]
    HumanReview -> Fail   [label="Reject"]

    Create      [agent="coder-a", prompt-ref="#creator-prompt"]
    Verify      [shell-ref="#verify-shell"]
    HumanReview [ask-ref="#human-question", question-type="confirmation"]

    Fail
}
```

```text #creator-prompt
Reply with exactly the word yes.
```

```sh #verify-shell
echo "$last_output" | grep -qx yes && echo ok || echo fail
```

```text #human-question
Is the referenced-content workflow feature working as expected?
```
