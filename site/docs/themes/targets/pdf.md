---
title: PDF Theme Target
description: |
  How paged-media tokens and print styles map into PDF output.
---

# Overview

PDF output is produced by rendering the document as HTML with print media styles enabled. Theme tokens in `pages.css` and module-level `@media print` rules define page size, margins, headers/footers, and print-specific styling.

This translation happens whenever you create a PDF with Stencila, for example using the CLI's [`render`](../../cli/render.md) command:

```sh
stencila render report.smd report.pdf
```

# Paged Media Tokens

Paged media tokens come from [`pages.css`](https://github.com/stencila/stencila/blob/main/web/src/themes/base/pages.css) and are documented [here](../tokens/print.smd). These tokens map directly into CSS `@page` rules that are respected by the PDF renderer.

## Page Size and Margins

| Tokens | PDF output |
| --- | --- |
| `--page-width`, `--page-height` | `@page` size |
| `--page-margin-*` | Page margins |
| `--page-padding-*` | Padding between content and margin boxes |

## Header/Footer (Margin Boxes)

| Tokens | PDF output |
| --- | --- |
| `--page-top-*-content`, `--page-bottom-*-content` | Margin box content |
| `--page-margin-font-family`, `--page-margin-font-size` | Margin box typography |
| `--page-margin-color`, `--page-margin-line-height` | Margin box text styles |
| `--page-border-*` | Header/footer rules |

## First-Page Overrides

| Tokens | PDF output |
| --- | --- |
| `--page1-*` | First-page margin box overrides |

# Print Variants

Component modules include `@media print` variants (e.g. tables, figures, code) to adjust styling specifically for PDF/print output. Where available, `*-print` tokens are used to override screen defaults for print.

# Implementation Notes

- The PDF renderer uses print media and honors `@page` rules (see [`html_to_png.rs`](https://github.com/stencila/stencila/blob/main/rust/convert/src/html_to_png.rs)).
- Paged media tokens live in [`pages.css`](https://github.com/stencila/stencila/blob/main/web/src/themes/base/pages.css) and are documented in `site/docs/themes/tokens/print.smd`.
