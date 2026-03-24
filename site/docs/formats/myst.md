---
title: MyST Markdown
description: An extended flavor of Markdown for technical and scientific communication
---

# Introduction

[MyST Markdown](https://mystmd.org/) (Markedly Structured Text) is an extended Markdown flavor that adds directives and roles inspired by [reStructuredText](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html). It is designed for technical documentation and scientific publishing, offering features like admonitions, cross-references, figures with captions, and math blocks within a Markdown-based syntax.

# Usage

To convert to/from MyST Markdown, use the `.myst` file extension, or the `--to myst` or `--from myst` options e.g.

```sh
stencila convert doc.smd doc.myst
```

> [!warning]
> Stencila's MyST Markdown support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila supports bi-directional conversion between Stencila documents and MyST Markdown. MyST directives and roles are mapped to Stencila Schema node types where equivalents exist. The underlying Markdown parser and encoder are shared with [CommonMark](../md) and other Markdown flavors; see the [CommonMark](../md) documentation for implementation details.

# Limitations

- Not all MyST directives and roles have Stencila equivalents; unsupported directives may be imported as raw content or dropped.
- Stencila node types without MyST equivalents are lost when encoding to MyST.
- MyST-specific features like substitutions, glossaries, and some Sphinx extensions are not supported.
- MyST Markdown rendered by Stencila may differ in behavior from MyST rendered by the official MyST parser.
