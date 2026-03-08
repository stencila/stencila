---
name: test-agent-reference
description: Test agent= attribute for named agent resolution
goal: Say hello
---

Tests the `agent=` node attribute, which references a named agent from the workspace or user agent registry. The workflow runner resolves the agent by name and delegates the LLM call to that agent's session. If no matching agent is found, the runner logs a warning and falls back to the default agent.

```dot
digraph Workflow {
    Start -> Greet -> Verify

    Greet [agent="default", prompt="Reply with ONLY the word: hello"]

    Verify [prompt="Does '$last_output' equal 'hello'? Reply with ONLY yes or no in lowercase, nothing else."]
    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail

    Fail
}
```
