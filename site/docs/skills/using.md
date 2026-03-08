---
title: Using Skills
description: How to list, inspect, validate, and use skills with agents.
---

## Listing Skills

See all discovered skills (`stencila skills` is shorthand for `stencila skills list`):

```sh
stencila skills
```

This shows skills from all source directories with their names, source, description, and license.

Filter by source:

```sh
# Only skills from .claude/skills/
stencila skills list --source claude

# Only base Stencila skills
stencila skills list --source stencila
```

Output as JSON or YAML:

```sh
stencila skills list --as json
```

## Inspecting Skills

View the full content of a skill:

```sh
stencila skills show data-analysis
```

This renders the skill as Markdown. Use `--as` for other formats:

```sh
stencila skills show data-analysis --as json
```

## How Agents Use Skills

When an agent session starts, skills are integrated through these steps:

1. **Discovery** — Stencila walks up from the working directory looking for skill directories. The base layer (`.stencila/skills/`) is always included; provider-specific directories (e.g. `.claude/skills/`) are added based on the agent's provider.

2. **Metadata injection** — Compact metadata (name and description only) for all discovered skills is serialized as XML and included in the system prompt:

   ```xml
   <skills>
     <skill name="data-analysis" description="Analyze datasets and generate summary statistics." />
     <skill name="code-review" description="Review code for correctness, style, and security." />
   </skills>
   ```

3. **Tool registration** — A `use_skill` tool is registered in the agent's tool registry with the description: *"Load the full instructions for a workspace skill by name."*

4. **On-demand loading** — When the model determines a skill is relevant, it calls `use_skill` with the skill name. Stencila loads the full `SKILL.md` and returns it as XML:

   ```xml
   <skill name="data-analysis">
     <description>Analyze datasets and generate summary statistics.</description>
     <instructions>
       Step 1: Load the dataset using pandas...
     </instructions>
     <compatibility>Requires Python 3.10+</compatibility>
     <allowed-tools>Bash(python:*) Read</allowed-tools>
   </skill>
   ```

5. **Execution** — The model follows the skill's instructions to complete the task.

This progressive disclosure approach keeps the base context lean — only ~100 tokens per skill at startup — while giving agents access to detailed instructions when they need them.

## Controlling Skill Access

Use the `allowedSkills` field in an agent's `AGENT.md` frontmatter to control which skills the agent can use:

```yaml
# Allow only specific skills
allowedSkills:
  - data-analysis
  - code-review
```

```yaml
# Disable skills entirely
allowedSkills: []
```

When `allowedSkills` is not set, all discovered skills are available. See the [agent configuration reference](../agents/configuration#allowedskills) for details.

## Skills in Workflows

Workflow steps that use agents inherit skill access. If an agent step specifies `allowedSkills`, only those skills are available during that step. This lets you scope skills to specific stages of a pipeline — for example, restricting a data-cleaning step to only the `data-analysis` skill.

See the [workflows documentation](../workflows/) for more on defining workflow steps.
