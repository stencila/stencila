---
title: Microsoft Excel (XLS)
description: Legacy Microsoft Excel binary spreadsheet format (.xls)
---

# Introduction

XLS is the legacy binary spreadsheet format used by Microsoft Excel 97–2003. While superseded by [XLSX](../xlsx), XLS files remain common in older datasets and archives. Stencila can import XLS files, converting spreadsheet data into `Datatable` nodes.

# Usage

Use the `.xls` file extension, or the `--from xls` option, when importing from XLS e.g.

```sh
stencila convert workbook.xls table.smd
```

> [!info]
> XLS is currently supported for import (decoding) only.

# Implementation

XLS decoding is implemented in the Rust crate [`codec-xlsx`](https://github.com/stencila/stencila/blob/main/rust/codec-xlsx) using the [`calamine`](https://crates.io/crates/calamine) library.

# Limitations

- Export (encoding) to XLS is not supported. Consider exporting to [CSV](../csv) for tabular data output.
- Only the first worksheet in the workbook is read; additional sheets are ignored.
- Cell formatting, formulas, charts, and macros are not preserved — only cell values are extracted.
- The XLS binary format has known limitations with very large files and complex formatting.
