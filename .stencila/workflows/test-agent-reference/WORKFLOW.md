---
name: test-agent-reference
description: Test agent= attribute for named agent resolution
goal: Say hello
---

Tests the `agent=` node attribute, which references a named agent from the workspace or user agent registry. The workflow runner resolves the agent by name and delegates the LLM call to that agent's session. If no matching agent is found, the runner logs a warning and falls back to the default agent.

```dot
digraph Workflow {
  Start -> Greet

  Greet [agent="general", prompt="Reply with ONLY the word: hello"]
  Greet -> Verify

  Verify [agent="general", prompt="Does '$last_output' equal 'hello'? Verify the result and choose Pass or Fail."]
  Verify -> End  [label="Pass"]
  Verify -> Fail [label="Fail"]
}
```
