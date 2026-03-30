---
title: "Themes"
description: "Agents for creating and reviewing Stencila themes."
---

Agents for creating and reviewing Stencila themes.

> [!tip] Usage
>
> You can use these agents in the Stencila TUI by selecting one with the `/agent` command, or by starting your prompt with `#agent-name` e.g. `#theme-creator`. Agents can also be referenced by name in workflow node definitions.

- [**Theme Creator Agent**](./theme-creator/): Creates or updates Stencila theme CSS files using semantic tokens, module tokens, and the theme CLI
- [**Theme Reviewer Agent**](./theme-reviewer/): Reviews Stencila theme artifacts for token correctness, cross-target portability, dark-mode handling, and approval readiness. Inspects theme.css files, patches, and plans against the design-token vocabulary and produces a structured review report with prioritized findings.
