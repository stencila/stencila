---
title: LaTeX
description: A typesetting system for technical and scientific documents
---

# Introduction

[LaTeX](https://www.latex-project.org/) is a document preparation system widely used for scientific, technical, and mathematical writing. It provides fine-grained control over typesetting, cross-references, bibliographies, and document structure.

Stencila supports bi-directional conversion between Stencila documents and LaTeX. The `.latex` extension is used for LaTeX documents (as distinct from [plain TeX](../tex) files using `.tex`).

# Usage

Use the `.latex` file extension, or the `--to latex` or `--from latex` options, when converting to/from LaTeX e.g.

```sh
stencila convert doc.smd doc.latex
```

# Implementation

LaTeX support is implemented in the Rust crate [`codec-latex`](https://github.com/stencila/stencila/blob/main/rust/codec-latex), with per-node-type encoding derived via [`codec-latex-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-latex-trait) and [`codec-latex-derive`](https://github.com/stencila/stencila/blob/main/rust/codec-latex-derive).

# Limitations

- Conversion is lossy for Stencila node types that have no LaTeX equivalent (e.g. interactive widgets, styled blocks with CSS).
- LaTeX import is limited; complex custom macros and packages may not be fully parsed.
- For plain TeX input/output (without LaTeX preamble conventions), use the [TeX](../tex) format.
