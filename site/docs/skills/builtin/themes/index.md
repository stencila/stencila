---
title: "Themes"
description: "Skills for creating and reviewing Stencila themes."
---

Skills for creating and reviewing Stencila themes.

To give an agent access to a skill, add it to the `allowed-skills` list in the agent's AGENT.md frontmatter e.g. `allowed-skills: theme-creation`. When creating a new agent, you can prompt the `#agent-creator` agent or the `~agent-creation-iterative` workflow to use specific skills.

- [**Theme Creation Skill**](./theme-creation/): Create, update, or plan a Stencila theme for documents or published sites. Use when asked to choose a theme direction, write or patch theme.css, recommend semantic or module token families, customize site navigation or branding, tune PDF and print page tokens, align web, Python, and R plots with a Stencila design system, list available builtin tokens with `stencila themes tokens`, or validate a theme file with `stencila themes validate`.
- [**Theme Review Skill**](./theme-review/): Critically review an existing or proposed Stencila theme artifact for correctness, token usage, target coverage, cross-target portability, dark-mode handling, maintainability, and approval readiness. Use when asked to review, critique, assess, audit, or validate a theme.css file, theme patch, theme plan, site theme, document theme, plot theme, print or PDF theme, check design tokens, assess DOCX or email behavior, review dark mode support, or validate with stencila themes validate.
