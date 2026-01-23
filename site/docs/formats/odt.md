---
title: ODT
description: Open Document Text
---
# Introduction

The [Open Document Text (ODT)](https://docs.oasis-open.org/office/v1.1/OS/OpenDocument-v1.1-html/OpenDocument-v1.1.html) format is convenient when you want to pass documents to and from word processors.

# Usage

> [!info]
> Converting to/from ODT requires [Pandoc to be installed](https://pandoc.org/installing.html).

Use the `.odt` file extension, or the `--to odt` or `--from odt` options, when converting to/from ODT e.g.

```sh
stencila convert doc.smd doc.odt
```

> [!warning]
> Stencila's ODT support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila provides lossy bidirectional conversion to ODT powered by [Pandoc](https://pandoc.org/). To convert documents to/from ODT, you will need to have Pandoc installed. See the [`pandoc`](../pandoc) format for more details.

# Notes

- ODT conversion is lossy for complex layout and word-processor-specific features.
- Pandoc governs most of the ODT mapping behavior.
