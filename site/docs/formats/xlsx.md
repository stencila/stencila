---
title: Microsoft Excel
description: Microsoft Excel spreadsheet format (.xlsx)
---

# Introduction

Microsoft Excel's XLSX format is the current standard spreadsheet format for Microsoft Excel and is widely supported by other spreadsheet applications. Stencila can import XLSX files, converting spreadsheet data into `Datatable` nodes for further processing.

# Usage

Use the `.xlsx` file extension, or the `--from xlsx` option, when importing from XLSX e.g.

```sh
stencila convert workbook.xlsx table.smd
```

> [!info]
> XLSX is currently supported for import (decoding) only.

# Implementation

XLSX decoding is implemented in the Rust crate [`codec-xlsx`](https://github.com/stencila/stencila/blob/main/rust/codec-xlsx) using the [`calamine`](https://crates.io/crates/calamine) library.

# Limitations

- Export (encoding) to XLSX is not supported. Consider exporting to [CSV](../csv) for tabular data output.
- Only the first worksheet in the workbook is read; additional sheets are ignored.
- Cell formatting, formulas, charts, and macros are not preserved — only cell values are extracted.
- Merged cells are read as individual cells, which may produce unexpected results for complex layouts.
