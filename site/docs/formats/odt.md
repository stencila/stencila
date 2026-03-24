---
title: ODT
description: OpenDocument Text
---

# Introduction

[OpenDocument Text (ODT)](https://docs.oasis-open.org/office/v1.1/OS/OpenDocument-v1.1-html/OpenDocument-v1.1.html) is an open standard document format used by LibreOffice, Apache OpenOffice, and other word processors. Stencila supports ODT as a way to exchange documents with open-source word processing tools.

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

Stencila provides bi-directional conversion to ODT powered by [Pandoc](https://pandoc.org/) via the [Pandoc JSON](../pandoc) intermediate format. See the [Pandoc format](../pandoc) documentation for more details on the intermediate representation.

# Limitations

- Pandoc must be installed separately; it is not bundled with Stencila.
- Conversion is lossy for Stencila node types that have no ODT/Pandoc equivalent (e.g. executable code chunks, parameters).
- ODT-specific formatting such as master pages, complex table styles, and embedded OLE objects are not preserved during import.
