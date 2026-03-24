---
title: PDF
description: Portable Document Format
---

# Introduction

[Portable Document Format (PDF)](https://pdfa.org/resource/pdf-specification-archive/) is a widely used format for sharing and publishing fixed-layout documents. Stencila supports both reading from and writing to PDF.

# Usage

Use the `.pdf` file extension, or the `--to pdf` or `--from pdf` options, when converting to/from PDF e.g.

```sh
stencila convert doc.smd doc.pdf
```

When encoding to PDF, the default rendering method uses a headless browser to convert DOM HTML to PDF. Alternatively, use the `--tool latex` or `--tool xelatex` option to render via LaTeX instead:

```sh
stencila convert doc.smd doc.pdf --tool latex
```

> [!info]
> Reading PDFs requires access to the Mistral OCR API. Writing PDFs via LaTeX requires a LaTeX distribution (e.g. TeX Live) to be installed.

> [!warning]
> Stencila's PDF support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

**Reading (decoding):** PDFs are converted to Stencila documents using Mistral OCR (`mistral-ocr-2505`), which extracts text, structure, and images from PDF pages as Markdown. The extracted Markdown is then parsed into Stencila Schema nodes. For small PDFs (8 pages or fewer), metadata extraction and content extraction are done in a single pass. For larger PDFs, metadata is extracted from the first pages separately and combined with content from the full document. Results are cached based on the PDF's content hash to avoid redundant API calls.

**Writing (encoding):** By default, documents are encoded to DOM HTML and converted to PDF using a headless browser. When the `--tool latex` option is used, documents are encoded to LaTeX first and compiled to PDF using a LaTeX engine.

# Limitations

- Reading PDFs depends on the Mistral OCR API, which requires network access and an API key.
- OCR-based extraction is inherently lossy — complex layouts, multi-column text, and embedded fonts may not be accurately reconstructed.
- PDF output does not preserve interactive elements such as executable code chunks.
- Images extracted from PDFs during import are embedded as base64 data URIs.
