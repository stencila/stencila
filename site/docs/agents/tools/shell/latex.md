---
title: "LaTeX"
description: "Guards against destructive LaTeX build tool operations"
---

This page lists the safe and destructive patterns in the **LaTeX Build Tools** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## LaTeX Build Tools

**Pack ID:** `latex.build_tools`

Guards against destructive LaTeX build tool operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `latex.build_tools.latexmk_version` | `^latexmk\s+--version\b[^\|><]*$` |
| `latex.build_tools.biber_version` | `^biber\s+--version\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `latex.build_tools.latexmk_clean` | Removes all generated files including PDFs and auxiliary files | Use `latexmk -c` (lowercase) to clean only auxiliary files, preserving PDFs | Medium |
| `latex.build_tools.latexmk_clean_all` | Removes all generated files including PDFs, auxiliary files, and extra generated files | Use `latexmk -c` (lowercase) to clean only auxiliary files, preserving PDFs | High |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/latex.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/latex.rs).
