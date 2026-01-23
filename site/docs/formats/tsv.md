---
title: TSV
description: Tab-Separated Values
---
# Introduction

TSV (Tab-Separated Values) is a plain-text tabular format similar to CSV but using tabs as delimiters.

# Usage

Use the `.tsv` file extension, or the `--to tsv` or `--from tsv` options, when converting to/from TSV e.g.

```sh
stencila convert table.smd table.tsv
```

# Implementation

TSV support is implemented in the Rust crate [`codec-csv`](https://github.com/stencila/stencila/blob/main/rust/codec-csv) using the [`csv`](https://crates.io/crates/csv) crate.

# Notes

- TSV conversion targets `Datatable` nodes; other node types are not supported.
- Round-tripping is lossy for data types that do not fit TSV's text-only model.
