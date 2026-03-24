---
title: DOCX
description: Microsoft Word DOCX format
---

# Introduction

[DOCX](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/d683fa62-8042-4360-a824-b79045a6aabd) is a widely used file format for word processing documents. Although originally developed for Microsoft Word, DOCX is supported by many word processors, collaborative editing tools, and publishing workflows.

Stencila provides bi-directional conversion to and from DOCX, making it possible to collaborate with colleagues using word processors and to integrate with DOCX-based workflows.

# Usage

> [!info]
> Converting to/from DOCX requires [Pandoc to be installed](https://pandoc.org/installing.html).

Use the `.docx` file extension, or the `--to docx` or `--from docx` options, when converting to/from DOCX e.g.

```sh
stencila convert doc.smd doc.docx
```

When encoding to DOCX, you can use the `--reproducible` flag to embed a JSON cache inside the DOCX file, enabling higher-fidelity round-trips when the document is later imported back into Stencila:

```sh
stencila convert doc.smd doc.docx --reproducible
```

> [!warning]
> Stencila's DOCX support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila provides bi-directional conversion to DOCX. The core document structure conversion is powered by [Pandoc](https://pandoc.org/) via the [Pandoc JSON](../pandoc) intermediate format. On top of this, Stencila applies its own post-processing to the generated DOCX file, including custom theming, font embedding, page layout, and headers/footers.

When the `--reproducible` flag is used, Stencila embeds a JSON representation of the document inside the DOCX file. On re-import, this cache is used to reconstitute Stencila-specific node types that Pandoc cannot represent, improving round-trip fidelity.

# Limitations

- Pandoc must be installed separately; it is not bundled with Stencila.
- Conversion is lossy for Stencila node types that have no DOCX/Pandoc equivalent (e.g. executable code chunks, parameters, styled blocks).
- Complex Word-specific features such as tracked changes, comments, and form fields are not preserved during import.
- Custom DOCX templates from other tools may not be fully compatible with Stencila's post-processing.
