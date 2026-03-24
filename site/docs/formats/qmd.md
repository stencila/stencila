---
title: Quarto Markdown
description: An extended flavor of Markdown for technical and scientific publishing
---

# Introduction

[Quarto Markdown](https://quarto.org/docs/authoring/markdown-basics.html) is an extended Markdown format developed by Posit (formerly RStudio) for technical and scientific publishing. It supports code execution in R, Python, Julia, and Observable, along with features like citations, cross-references, and multi-format output. Quarto builds on Pandoc's Markdown extensions.

# Usage

To convert to/from Quarto Markdown, use the `.qmd` file extension, or the `--to qmd` or `--from qmd` options e.g.

```sh
stencila convert doc.smd doc.qmd
```

> [!warning]
> Stencila's Quarto Markdown support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Quarto Markdown. Quarto-specific YAML metadata and code chunk options are mapped to Stencila Schema properties where equivalents exist. The underlying Markdown parser and encoder are shared with [CommonMark](../md) and other flavors; see the [CommonMark](../md) documentation for implementation details.

# Limitations

- Quarto-specific features such as Quarto shortcodes, project-level configuration, and Quarto-specific YAML options may not be fully mapped.
- Stencila node types without Quarto Markdown equivalents are lost during export.
- Stencila does not execute Quarto code chunks natively. Use Stencila's own kernels or Quarto CLI for execution.
- Conversion fidelity may differ from what the Quarto CLI produces due to differences in Markdown parsing.
