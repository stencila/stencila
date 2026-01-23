---
title: MyST Markdown
description: An extended flavor of Markdown for technical, scientific communication and publication
---
# Introduction

[MyST Markdown](https://mystmd.org/) is an extended flavor of Markdown that supports additional syntax features like directives and roles, familiar to users of [reStructuredText](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html). It is designed to enhance the expressiveness of standard Markdown for technical documentation while retaining Markdown's simplicity and readability.

# Usage

To convert to/from MyST Markdown, use the `.myst` file extension, or the `--to myst` or `--from myst` options e.g.

```sh
stencila convert doc.smd doc.myst
```

> [!warning]
> Stencila's MyST Markdown support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila supports bi-directional conversion between Stencila documents and MyST Markdown. See our [CommonMark](../md) documentation for implementation details.

# Notes

- MyST is a Markdown flavor; some Stencila nodes may be lossy when round-tripping.
- MyST directives and roles map to Stencila extensions where possible.
