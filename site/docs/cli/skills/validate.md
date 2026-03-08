---
title: "`stencila skills validate`"
description: Validate a skill
---

Validate a skill

Checks that a skill conforms to the Agent Skills Specification naming and constraint rules. Accepts a skill name, a directory path, or a path to a SKILL.md file.

# Usage

```sh
stencila skills validate <TARGET>
```

# Examples

```bash
# Validate a skill by name
stencila skills validate data-analysis

# Validate a skill directory
stencila skills validate .stencila/skills/data-analysis

# Validate a SKILL.md file directly
stencila skills validate .stencila/skills/data-analysis/SKILL.md
```

# Arguments

| Name       | Description                                   |
| ---------- | --------------------------------------------- |
| `<TARGET>` | Skill name, directory path, or SKILL.md path. |
