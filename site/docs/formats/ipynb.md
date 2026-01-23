---
title: IPYNB
description: Jupyter Notebook Format
---
# Introduction

The [Jupyter Notebook Format](https://nbformat.readthedocs.io/en/latest/) (previously known the IPython Notebook (IPYNB) format), is a JSON-based format that integrates code, visualizations, equations, and narrative text in a single document. It is designed to support reproducible computational workflows by allowing users to execute code in real time while interweaving multimedia content and explanatory text.

# Usage

Use the `.ipynb` file extension, or the `--to ipynb` or `--from ipynb` options, when converting to/from Jupyter Notebooks e.g.

```sh
stencila convert doc.smd doc.ipynb
```

> [!warning]
> Stencila's IPYNB support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila supports bi-directional conversion between Stencila documents and `ipynb` files powered by the [`nbformat`](https://crates.io/crates/nbformat) Rust crate.

# Notes

- Jupyter notebooks focus on code cells and outputs; some Stencila-specific features may not round-trip.
- Output rendering is dependent on notebook tooling.
