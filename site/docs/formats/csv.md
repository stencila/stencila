---
title: CSV
description: Comma-Separated Values
---

# Introduction

CSV (Comma-Separated Values) is a simple, widely used tabular data format. Each line represents a row, with values separated by commas. CSV is supported by virtually all spreadsheet applications and data analysis tools.

Stencila supports bi-directional conversion between CSV files and `Datatable` nodes.

# Usage

Use the `.csv` file extension, or the `--to csv` or `--from csv` options, when converting to/from CSV e.g.

```sh
stencila convert table.smd table.csv
```

# Implementation

CSV support is implemented in the Rust crate [`codec-csv`](https://github.com/stencila/stencila/blob/main/rust/codec-csv) using the [`csv`](https://crates.io/crates/csv) crate.

# Limitations

- Only `Datatable` nodes are supported. Other document node types (articles, paragraphs, etc.) cannot be converted to or from CSV.
- All values are stored as text. Numeric and boolean types are inferred on import but type information is lost on export.
- CSV has no standard for encoding nested or structured data within cells.
