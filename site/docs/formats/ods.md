---
title: OpenDocument Spreadsheet
description: OpenDocument Spreadsheet format (.ods)
---
# Introduction

OpenDocument Spreadsheet (ODS) is an open standard spreadsheet format used by LibreOffice and other tools.

# Usage

Use the `.ods` file extension, or the `--from ods` option, when converting from ODS e.g.

```sh
stencila convert workbook.ods table.smd
```

# Implementation

ODS decoding is implemented in the Rust crate [`codec-xlsx`](https://github.com/stencila/stencila/blob/main/rust/codec-xlsx) using the [`calamine`](https://crates.io/crates/calamine) library.

# Notes

- ODS is currently supported for import only.
- Stencila reads the first worksheet and maps it to a `Datatable`.
