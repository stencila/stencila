---
title: Skills
description: Documentation for Stencila workspace skills — reusable instructions for AI agents.
---

Skills are reusable instruction sets for AI agents. A skill is a directory containing a `SKILL.md` file with YAML frontmatter and a Markdown body. Skills follow the [Agent Skills Specification](https://agentskills.io/specification), an open standard for portable agent instructions.

## How Skills Work

Skills use **progressive disclosure** to minimize context usage:

1. **Metadata** (~100 tokens) — the `name` and `description` of every discovered skill are loaded at session start
2. **Instructions** — the full `SKILL.md` body is loaded only when the agent activates a skill
3. **Resources** — files in `scripts/`, `references/`, and `assets/` are loaded only when referenced

At the start of an agent session, Stencila serializes compact metadata for all discovered skills as XML and injects it into the system prompt:

```xml
<skills>
  <skill name="data-analysis" description="Analyze datasets and generate summary statistics." />
  <skill name="code-review" description="Review code for correctness, style, and security." />
</skills>
```

A `use_skill` tool is registered in the agent's tool registry. When the model determines a skill is relevant to the current task, it calls `use_skill` with the skill's name. Stencila loads the full skill content and returns it as XML:

```xml
<skill name="data-analysis">
  <description>Analyze datasets and generate summary statistics.</description>
  <instructions>
    Step 1: Load the dataset...
  </instructions>
</skill>
```

The model then follows the skill's instructions to complete the task.

## Skill Discovery

Skills are discovered from multiple source directories. Each source is a dot-directory that may contain a `skills/` subdirectory:

| Source | Location | Description |
| ------ | -------- | ----------- |
| **Stencila** | `.stencila/skills/` | Base layer, always loaded first |
| **Claude** | `.claude/skills/` | Anthropic provider override |
| **Codex** | `.codex/skills/` | OpenAI provider override |
| **Gemini** | `.gemini/skills/` | Google Gemini provider override |

Stencila skills are always loaded. Provider-specific directories are added based on the agent's provider — an Anthropic agent sees `.stencila/skills/` plus `.claude/skills/`, an OpenAI agent sees `.stencila/skills/` plus `.codex/skills/`, and so on.

When the same skill name appears in multiple sources, the **provider-specific source wins** (last loaded takes precedence). This lets you define a base skill in `.stencila/skills/` and override it with a provider-tuned version in `.claude/skills/` or `.codex/skills/`.

## Skills vs. Project Docs

Both skills and project docs (`AGENTS.md`, `CLAUDE.md`, `.codex/instructions.md`, `GEMINI.md`) provide instructions to agents, but they serve different purposes:

| | Project Docs | Skills |
| --- | --- | --- |
| **Loading** | Always included in the system prompt | Loaded on demand by the model |
| **Scope** | General project context and conventions | Focused instructions for specific tasks |
| **Size** | Budget-limited (default 32KB total) | Each skill loaded independently |
| **Best for** | Coding conventions, project structure, general rules | Step-by-step workflows, domain-specific procedures |

Use project docs for information every agent session needs. Use skills for detailed instructions that only apply to specific tasks — the model will load them when relevant, keeping the base context lean.

## Next Steps

- [Creating Skills](creating) — write a SKILL.md from scratch
- [Using Skills](using) — list, inspect, validate, and use skills with agents
- [Configuration Reference](configuration) — full SKILL.md frontmatter reference
