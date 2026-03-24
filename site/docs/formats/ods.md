---
title: OpenDocument Spreadsheet
description: OpenDocument spreadsheet format (.ods)
---

# Introduction

OpenDocument Spreadsheet (ODS) is an open standard spreadsheet format used by LibreOffice Calc, Apache OpenOffice Calc, and other applications. Stencila can import ODS files, converting spreadsheet data into `Datatable` nodes.

# Usage

Use the `.ods` file extension, or the `--from ods` option, when importing from ODS e.g.

```sh
stencila convert workbook.ods table.smd
```

> [!info]
> ODS is currently supported for import (decoding) only.

# Implementation

ODS decoding is implemented in the Rust crate [`codec-xlsx`](https://github.com/stencila/stencila/blob/main/rust/codec-xlsx) using the [`calamine`](https://crates.io/crates/calamine) library.

# Limitations

- Export (encoding) to ODS is not supported. Consider exporting to [CSV](../csv) for tabular data output.
- Only the first worksheet is read; additional sheets are ignored.
- Cell formatting, formulas, charts, and macros are not preserved — only cell values are extracted.
