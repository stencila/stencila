---
title: TSV
description: Tab-Separated Values
---

# Introduction

TSV (Tab-Separated Values) is a plain-text tabular format similar to [CSV](../csv) but using tab characters as delimiters. It is commonly used in bioinformatics and data analysis pipelines.

Stencila supports bi-directional conversion between TSV files and `Datatable` nodes.

# Usage

Use the `.tsv` file extension, or the `--to tsv` or `--from tsv` options, when converting to/from TSV e.g.

```sh
stencila convert table.smd table.tsv
```

# Implementation

TSV support is implemented in the Rust crate [`codec-csv`](https://github.com/stencila/stencila/blob/main/rust/codec-csv) using the [`csv`](https://crates.io/crates/csv) crate with a tab delimiter.

# Limitations

- Only `Datatable` nodes are supported. Other document node types cannot be converted to or from TSV.
- All values are stored as text. Type information is lost on export and inferred on import.
- Tab characters within field values require quoting, which not all tools handle consistently.
