---
title: Microsoft Excel (XLS)
description: Legacy spreadsheet format (.xls)
---
# Introduction

XLS is the legacy Microsoft Excel binary format for spreadsheets.

# Usage

Use the `.xls` file extension, or the `--from xls` option, when converting from XLS e.g.

```sh
stencila convert workbook.xls table.smd
```

# Implementation

XLS decoding is implemented in the Rust crate [`codec-xlsx`](https://github.com/stencila/stencila/blob/main/rust/codec-xlsx) using the [`calamine`](https://crates.io/crates/calamine) library.

# Notes

- XLS is currently supported for import only.
- Stencila reads the first worksheet and maps it to a `Datatable`.
