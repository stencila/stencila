---
title: PDF
description: Portable Document Format
---
# Introduction

[Portable Document Format (PDF)](https://pdfa.org/resource/pdf-specification-archive/) is an output format that can be used to render documents suitable for publication or sharing.

# Usage

> [!info]
> Converting to PDF requires [Pandoc](https://pandoc.org/installing.html) and a PDF engine (e.g. `pdflatex`) to be installed.

Use the `.pdf` file extension, or the `--to pdf` option, when converting to PDF e.g.

```sh
stencila convert doc.smd doc.pdf
```

> [!warning]
> Stencila's PDF support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila provides lossy conversion to PDF powered by [Pandoc](https://pandoc.org/). To convert documents to PDF, you will need to have Pandoc and a PDF engine installed. See the [`pandoc`](../pandoc) format for more details.

# Notes

- PDF output is intended for publishing and sharing, not for further editing.
- Some formatting and interactive elements do not round-trip through PDF.
