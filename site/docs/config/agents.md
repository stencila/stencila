---
title: Agents Config
description: Agent configuration.
---

Agent configuration.

Controls which agent is used by default.

**Type:** `AgentsConfig`

# `default`

**Type:** `string` (optional)

The name of the default agent to use when no agent is specified.

When set, the TUI and other callers that request the "default" agent
will use this agent instead.

```toml
[agents]
default = "code-engineer"
```

# `commit_attribution`

**Type:** `CommitAttribution` (optional)

How Stencila should be attributed in git commits made by agents.

Supported values:
- `"author"`: Stencila is set as commit author.
- `"co-author"`: Stencila is added as commit co-author (default).
- `"committer"`: Stencila is set as commit committer.
- `"none"`: Stencila is not mentioned in attribution.

```toml
[agents]
commit_attribution = "co-author"
```

| Value | Description |
|-------|-------------|
| `author` | Set Stencila as the commit author (`GIT_AUTHOR_*`). |
| `co-author` | Keep normal author/committer identity and add a Stencila co-author trailer. |
| `committer` | Set Stencila as the commit committer (`GIT_COMMITTER_*`). |
| `none` | Do not mention Stencila in commit attribution. |


***

This documentation was generated from [`lib.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/lib.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
