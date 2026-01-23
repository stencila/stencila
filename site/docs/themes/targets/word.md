---
title: Word (DOCX) Theme Target
description: |
  How Stencila theme tokens map into DOCX styles and page layout.
---

# Overview

When Stencila renders DOCX, theme tokens are translated into Word styles (paragraph, character, and table styles) plus section layout. The encoder uses computed theme variables so CSS values (including `calc()` and `color-mix()`) resolve before mapping into DOCX.

This translation happens whenever you create a Word document with Stencila, for example using the CLI's [`render`](../../cli/render.md) command:

```sh
stencila render report.myst report.docx
```

# Theme Module Coverage

The DOCX encoder maps specific theme modules into Word styles. This keeps the mapping predictable and aligns with the CSS architecture.

| Theme module | DOCX output | Implementation |
| --- | --- | --- |
| `tokens-semantic.css` | Document defaults + Normal | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `headings.css` | Heading1-9 + Heading1Char-9Char | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `paragraphs.css` | Normal + BodyText | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `code.css` | VerbatimChar (inline code) | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `links.css` | Hyperlink style | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `quotes.css` | BlockText style | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `lists.css` | List paragraph spacing (markers via numbering.xml) | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `tables.css` | Table style + TableCaption | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `figures.css` | ImageCaption + CaptionedFigure | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `articles.css` | Title, Author, Abstract, AbstractTitle | [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) |
| `pages.css` | Page size, margins, headers/footers | [`encode_page_layout.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_page_layout.rs) |

# Page Layout (Paged Media)

Paged media tokens (e.g. `--page-width`, `--page-margin-*`, `--page-top-*-content`) are translated into DOCX section properties and header/footer content. First-page overrides (`--page1-*`) map to the DOCX first-page header/footer definitions.

# Gaps and Not-Yet-Mapped Areas

Some web-only or interactive modules are not mapped to DOCX styles. As of the current implementation:

- Not yet mapped: `admonitions.css`, `breaks.css`, `citations.css`, `labels.css`, `references.css`
- Not applicable to DOCX: `datatables.css`, `diagrams.css`, `images.css`, `plots.css`, `math.css`

For the authoritative mapping details, see [`encode_theme.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_theme.rs) and [`encode_page_layout.rs`](https://github.com/stencila/stencila/blob/main/rust/codec-docx/src/encode_page_layout.rs).
