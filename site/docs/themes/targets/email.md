---
title: Email Theme Target
description: |
  How theme tokens map into MJML and email-friendly HTML.
---

# Overview

When Stencila renders email output, theme tokens are converted into [MJML](https://mjml.io/) attributes and CSS rules, then compiled to HTML suitable for email clients. Theme values are computed with pixel conversion so layout values remain consistent across email clients.

This translation happens whenever you create email-ready HTML with Stencila, for example using the CLI's [`render`](../../cli/render.md) command:

```sh
stencila render report.smd report.email.html
```

# Token Mapping

The email encoder focuses on typography and component-level styling that can be safely represented in email HTML.

## Core Typography

| Tokens                   | Email output                                       |
| ------------------------ | -------------------------------------------------- |
| `--text-font-family`     | `<mj-all font-family>`                             |
| `--text-color-primary`   | `<mj-all color>` and base text color               |
| `--text-font-size`       | `<mj-text font-size>` (px)                         |
| `--text-line-height`     | Base line-height for body and paragraphs           |
| `--text-color-secondary` | Secondary text color (authors, abstract, metadata) |
| `--text-color-muted`     | Muted text color for supporting text               |

## Headings

| Tokens                                              | Email output           |
| --------------------------------------------------- | ---------------------- |
| `--heading-font-family`                             | Heading font family    |
| `--heading-color`                                   | Heading color          |
| `--heading-line-height`                             | Heading line-height    |
| `--heading-letter-spacing`                          | Heading letter spacing |
| `--heading-font-weight`                             | Heading font weight    |
| `--heading-font-size` + `--heading-font-size-ratio` | h1-h6 font sizes       |

## Links

| Tokens               | Email output                  |
| -------------------- | ----------------------------- |
| `--link-color`       | Link color                    |
| `--link-decoration`  | Link underline style          |
| `--link-color-hover` | Hover color (where supported) |

## Code, Quotes, and Tables

| Tokens                                                                                                                         | Email output                             |
| ------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------- |
| `--code-font-family`, `--code-color`, `--code-background`, `--code-border-color`, `--code-border-radius`, `--code-line-height` | Inline and block code styles             |
| `--quote-background`, `--quote-border-width`, `--quote-border-color`, `--quote-font-style`, `--quote-padding`                  | Block quote styling                      |
| `--table-border-color`, `--table-header-background`, `--table-header-font-weight`, `--table-cell-padding`                      | Table borders, headers, and cell padding |

## Article Metadata

| Tokens                 | Email output                                      |
| ---------------------- | ------------------------------------------------- |
| `--article-title-*`    | Title font family, size, weight, color, alignment |
| `--article-authors-*`  | Author font size, color, alignment, spacing       |
| `--article-abstract-*` | Abstract font size, color, alignment, background  |

## Lists and Surfaces

| Tokens                                                        | Email output                            |
| ------------------------------------------------------------- | --------------------------------------- |
| `--list-indent`, `--list-item-spacing`, `--list-marker-color` | List indentation, spacing, marker color |
| `--surface-background`                                        | Body background color                   |
| `--border-color-default`                                      | Divider and border defaults             |
| `--border-radius-default`                                     | Rounded corners for abstract callouts   |

# Implementation Notes

- Theme variables are computed with pixel conversion in [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-email/src/encode_theme.rs).
- The MJML/HTML encoder only uses tokens that are reliable in email clients.
- For full token usage, see [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-email/src/encode_theme.rs).
