---
title: IPYNB
description: Jupyter Notebook format
---

# Introduction

The [Jupyter Notebook format](https://nbformat.readthedocs.io/en/latest/) (`.ipynb`, originally the IPython Notebook format) is a JSON-based format that integrates code, visualizations, equations, and narrative text in a single document. It is widely used for reproducible computational workflows, data science, and interactive computing.

# Usage

Use the `.ipynb` file extension, or the `--to ipynb` or `--from ipynb` options, when converting to/from Jupyter Notebooks e.g.

```sh
stencila convert doc.smd doc.ipynb
```

> [!warning]
> Stencila's IPYNB support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila supports bi-directional conversion between Stencila documents and `.ipynb` files powered by the [`nbformat`](https://crates.io/crates/nbformat) Rust crate.

# Limitations

- Jupyter notebooks organize content into cells (code, markdown, raw). Stencila node types that don't fit this cell model may not round-trip cleanly.
- Notebook outputs (e.g. rich display objects, widget state) are preserved during import but may not be fully reproduced on export without re-execution.
- Notebook-level metadata (e.g. kernel specification) is mapped where possible, but custom metadata fields may be dropped.
