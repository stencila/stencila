---
title: Agents
description: Documentation for Stencila's coding agents.
---

Stencila agents pair large language models with general, and research-focused, tools in a programmable agentic loop. An agent is a directory containing an `AGENT.md` file that defines which model to use, what tools are available, and how the agent should behave.

## How Agents Work

Agents operate in an **agentic loop**: the model reasons about a task, requests one or more tool calls, Stencila executes them and returns the results, and the model reasons again. This cycle repeats until the task is complete or a limit is reached. It is this loop — not the model alone — that makes agents capable of multi-step tasks like debugging across files, running experiments, or iterating on code until tests pass.

```
User prompt → Model reasons → Tool calls → Results → Model reasons → ... → Done
```

Each agent session automatically receives:

- **Environment context** — working directory, git branch, recent commits, platform info
- **Project docs** — `AGENTS.md`, `CLAUDE.md`, `.codex/instructions.md`, or `GEMINI.md` files discovered in the project tree
- **[Workspace skills](../skills/)** — reusable instructions from `.stencila/skills/` loaded on demand
- **Provider-aligned tools** — file, shell, web, and editing tools matched to the model's provider

## Agent Discovery

Agents are discovered from three sources, listed from lowest to highest precedence:

| Source | Location | Description |
| ------ | -------- | ----------- |
| **CLI-detected** | System PATH | Auto-detected CLI tools (`claude`, `codex`, `gemini`) exposed as agents |
| **User** | `~/.config/stencila/agents/` | Personal agents shared across all workspaces |
| **Workspace** | `.stencila/agents/` | Project-specific agents versioned with the repo |

When the same agent name appears in multiple locations, the higher-precedence version wins. A workspace agent named `claude` overrides the CLI-detected `claude` agent, for example.

## Session Routing

When an agent session starts, Stencila routes it to either an **API backend** or a **CLI backend**:

1. If the agent's provider is an explicit CLI provider (`claude-cli`, `codex-cli`, `gemini-cli`), a CLI-backed session is always used.
2. Otherwise an API provider is resolved — from the agent definition, inferred from the model name, or the default configured provider.
3. If API credentials exist for that provider, an API session is created.
4. If no API credentials exist but a corresponding CLI tool is available (e.g. `anthropic` → `claude-cli`), the session falls back to CLI.
5. If no CLI fallback exists (e.g. `mistral`, `deepseek`), an error is returned asking the user to set the appropriate API key.

Use `stencila agents resolve <name>` to see how a specific agent would be routed.

## Next Steps

- [Creating Agents](creating) — create and configure your own agents
- [Using Agents](using) — run agents from the CLI and TUI
- [Configuration Reference](configuration) — full reference for `AGENT.md` properties
- [Tools](tools/) — the tools available to agents and the guard system that evaluates every tool call
