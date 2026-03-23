---
title: Using Agents
description: How to list, inspect, run, and manage agents from the CLI and TUI.
---

## Listing Agents

See all discovered agents (`stencila agents` is shorthand for `stencila agents list`):

```sh
stencila agents
```

This shows agents from all three sources (workspace, user, CLI-detected) with their names, descriptions, source, and configuration summary.

Filter by source:

```sh
# Only workspace agents
stencila agents list --source workspace

# Only CLI-detected agents
stencila agents list --source cli
```

Output as JSON or YAML:

```sh
stencila agents list --as json
```

## Inspecting Agents

View the full definition of an agent:

```sh
stencila agents show code-engineer
stencila agents show code-engineer --as json
```

## Running Agents

Run an agent with a prompt from the CLI:

```sh
stencila agents run code-engineer "What files are in this directory?"
```

Text arguments and file paths can be mixed — file paths are automatically detected, read, and included as content:

```sh
stencila agents run code-reviewer "Review this file:" src/main.rs
```

Write the agent's output to a file:

```sh
stencila agents run code-engineer "Generate a README" --output README.md
```

### Dry Runs

Preview how an agent would be configured without actually running it:

```sh
stencila agents run code-engineer "Hello" --dry-run
```

This shows the agent metadata, prompt, instructions (from the AGENT.md body), and the full session configuration — useful for debugging agent behavior.

## Resolving Routing

See how an agent session would be routed (API vs CLI, which provider and model):

```sh
stencila agents resolve code-engineer
```

Example output:

```
Agent:       code-engineer (.stencila/agents/code-engineer)
Provider:    anthropic (agent definition)
Model:       claude → claude-sonnet-4-5
Session:     API
Credentials: ANTHROPIC_API_KEY (environment variable)
```

Add `--why` for extended details including provider priority, credential sources, and alias resolution:

```sh
stencila agents resolve code-engineer --why
```

If an agent uses `any` in `models` or `providers`, the explanation also helps you see when routing fell through to the next stage. For example, an agent with `models: [mistral-large-latest, any]` may show that the preferred model was skipped and selection continued via `model-size`, `providers`, or the default provider.

## The Default Agent

When no agent name is specified (e.g. in the TUI), Stencila uses the "default" agent. The default is resolved in this order:

1. The `[agents].default` setting in `stencila.toml`
2. The first discovered agent (workspace agents first, then user, then CLI-detected)

Configure the default in your project's `stencila.toml`:

```toml
[agents]
default = "code-engineer"
```

Or in the user-level config at `~/.config/stencila/stencila.toml`.

## Using Agents in the TUI

The Stencila TUI uses the default agent for its chat interface. Start the TUI with:

```sh
stencila
```

The agent session provides the same agentic loop as the CLI — tool calls are executed, results are displayed, and the model continues reasoning until the task is complete.

## Project Documentation

Agents automatically discover and include project documentation in their system prompt. The following files are recognized:

| File | Providers |
| ---- | --------- |
| `AGENTS.md` | All |
| `CLAUDE.md` | Anthropic |
| `.codex/instructions.md` | OpenAI |
| `GEMINI.md` | Gemini |

These files are loaded from the git root through every directory on the path to the working directory. Root-level files load first; subdirectory files are appended with higher precedence. The total budget is configurable (default 32KB) and content is truncated with a marker if exceeded.

This means existing `CLAUDE.md` or `.codex/instructions.md` files work automatically — agents read the project instructions that were written for the model's native CLI tool.

## Workspace Skills

Agents can use [workspace skills](../skills/) — reusable instruction sets stored in `.stencila/skills/`. Skills are discovered at session start, and their metadata (name and description) is included in the system prompt. The model can then load full skill content on demand using the `use_skill` tool.

Skills are also discovered from provider-specific directories (e.g. `.claude/skills/` for Anthropic agents). On name conflicts, the provider-specific source wins. See [Creating Skills](../skills/creating) for how to write skills and [Using Skills](../skills/using) for how agents load them at runtime.

When an agent's `allowed-skills` contains exactly one skill, Stencila also preloads that skill's full instructions into the initial system prompt in addition to making it available via `use_skill`. This avoids spending an extra model turn just to load the only permitted skill.

Disable skills for an agent by setting `allowed-skills` to an empty array in the agent definition:

```yaml
allowed-skills: []
```

Or restrict to specific skills:

```yaml
allowed-skills:
  - testing
  - documentation
```

## Commit Attribution

When agents make git commits, Stencila can be attributed in the commit. Configure this globally:

```toml
[agents]
commit_attribution = "co-author"   # default
```

| Mode | Effect |
| ---- | ------ |
| `author` | Sets Stencila as the commit author |
| `co-author` (default) | Adds a `Co-authored-by` trailer to the commit message |
| `committer` | Sets Stencila as the commit committer |
| `none` | No Stencila attribution in commits |

Agents are always instructed not to make commits unless explicitly asked to by the user.
