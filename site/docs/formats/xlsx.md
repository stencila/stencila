---
title: Microsoft Excel
description: Spreadsheet format (.xlsx)
---
# Introduction

Microsoft Excel's XLSX format is a widely used spreadsheet format for tabular data.

# Usage

Use the `.xlsx` file extension, or the `--from xlsx` option, when converting from XLSX e.g.

```sh
stencila convert workbook.xlsx table.smd
```

# Implementation

XLSX decoding is implemented in the Rust crate [`codec-xlsx`](https://github.com/stencila/stencila/blob/main/rust/codec-xlsx) using the [`calamine`](https://crates.io/crates/calamine) library.

# Notes

- XLSX is currently supported for import only.
- Stencila reads the first worksheet and maps it to a `Datatable`.
