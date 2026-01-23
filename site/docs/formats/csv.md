---
title: CSV
description: Comma-Separated Values
---
# Introduction

CSV (Comma-Separated Values) is a simple tabular data format widely used for spreadsheets and data exchange.

# Usage

Use the `.csv` file extension, or the `--to csv` or `--from csv` options, when converting to/from CSV e.g.

```sh
stencila convert table.smd table.csv
```

# Implementation

CSV support is implemented in the Rust crate [`codec-csv`](https://github.com/stencila/stencila/blob/main/rust/codec-csv) using the [`csv`](https://crates.io/crates/csv) crate.

# Notes

- CSV conversion targets `Datatable` nodes; other node types are not supported.
- Round-tripping is lossy for data types that do not fit CSV's text-only model.
