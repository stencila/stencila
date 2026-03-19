---
name: test-thread-reuse
description: Test fidelity=full and thread_id for session reuse across agent nodes
goal: Test the reuse of threads
---

Tests that two agent nodes sharing the same `thread_id` with `fidelity=full` reuse a session, allowing the second node to recall context from the first. A third node without session reuse should not have that context.

```dot
digraph Workflow {
  Start -> Remember

  Remember [
    fidelity="full",
    thread_id="shared",
    prompt="Remember that the secret word is 'tangerine'. Do not repeat it in response to this message but do respond with that word the next time I ask."
  ]
  Remember -> Recall

  Recall [
    fidelity="full",
    thread_id="shared",
    prompt="What was the secret word I told you earlier? Do NOT check last output. Respond with ONLY the word."
  ]
  Recall -> VerifyRecall

  VerifyRecall [shell="echo '$last_output' | grep -qi tangerine && echo pass || echo fail"]
  VerifyRecall -> Fresh       [label="Pass", condition="context.last_output=pass"]
  VerifyRecall -> Fail        [label="Fail"]

  Fresh [prompt="What was the secret word I told you earlier? Do NOT check last output. Respond with ONLY the word, or 'unknown' if you do not know."]
  Fresh -> VerifyFresh

  VerifyFresh [shell="echo '$last_output' | grep -qi tangerine && echo fail || echo pass"]
  VerifyFresh -> ListMessages    [label="Pass", condition="context.last_output=pass"]
  VerifyFresh -> Fail         [label="Fail"]

  ListMessages [
    fidelity="full",
    thread_id="shared",
    prompt="How many messages in this conversation thread, list them."
  ]
  ListMessages -> End
}
```
