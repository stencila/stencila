---
title: DOCX
description: Microsoft Word DOCX Format
---
# Introduction

[DOCX](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/d683fa62-8042-4360-a824-b79045a6aabd) is a popular file format for representing word processing documents. Although originally developed for Microsoft Word, many other word processors, tools, and workflows support DOCX.

Stencila provides support for DOCX as a way to collaborate with colleagues using word processors and to integrate with workflows and tools using DOCX.

# Usage

> [!info]
> Converting to/from DOCX requires [Pandoc to be installed](https://pandoc.org/installing.html).

Use the `.docx` file extension, or the `--to docx` or `--from docx` options, when converting to/from DOCX e.g.

```sh
stencila convert doc.smd doc.docx
```

> [!warning]
> Stencila's DOCX support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila provides lossy bidirectional conversion to DOCX powered by [Pandoc](https://pandoc.org/). To convert documents to/from DOCX, you will need to have Pandoc installed. See the [`pandoc`](../pandoc) format for more details.

# Notes

- DOCX conversion is lossy for complex layout and some Word-specific features.
- Pandoc governs most of the DOCX mapping behavior.
