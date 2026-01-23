---
title: "`stencila convert`"
description: Convert a document to another format
---

Convert a document to another format

# Usage

```sh
stencila convert [OPTIONS] [INPUT] [OUTPUTS]... [-- <TOOL_ARGS>...]
```

# Examples

```bash
# Convert Stencila Markdown to MyST Markdown
stencila convert document.smd document.myst

# Convert to multiple output formats
stencila convert input.smd output.html output.pdf output.docx

# Specify input and output formats explicitly
stencila convert input.txt output.json --from plain --to json

# Convert with specific codec options
stencila convert doc.md doc.html --standalone

# Convert only specific pages from a PDF
stencila convert document.pdf extract.md --pages 1,3,5-10

# Convert all pages except specific ones
stencila convert report.pdf content.md --exclude-pages 5,15

# Convert only odd pages from a document
stencila convert book.pdf odd-pages.md --pages odd

# Use an external tool like Pandoc
stencila convert doc.md doc.tex --tool pandoc

# Pass arguments to external tool
stencila convert doc.md doc.pdf --tool pandoc -- --pdf-engine=xelatex

# Convert from stdin to stdout (defaults to JSON)
echo "# Hello" | stencila convert
```

# Arguments

| Name          | Description                                               |
| ------------- | --------------------------------------------------------- |
| `[INPUT]`     | The path, URL or other identifier for the input file.     |
| `[OUTPUTS]`   | The paths of desired output files.                        |
| `[TOOL_ARGS]` | Arguments to pass through to the tool using for encoding. |

# Options

| Name                    | Description                                                                                                     |
| ----------------------- | --------------------------------------------------------------------------------------------------------------- |
| `-f, --from`            | The format of the input/s.                                                                                      |
| `--fine`                | Use fine decoding if available for input format. Possible values: `true`, `false`.                              |
| `--coarse`              | Use coarse decoding if available for input format. Possible values: `true`, `false`.                            |
| `--pages`               | Pages to include when decoding multi-page documents.                                                            |
| `--exclude-pages`       | Pages to exclude when decoding multi-page documents.                                                            |
| `--ignore-artifacts`    | Ignore cached artifacts and force re-processing. Possible values: `true`, `false`.                              |
| `--no-artifacts`        | Prevent creating artifacts during decoding. Possible values: `true`, `false`.                                   |
| `--island-wrap`         | Wrap specified environments in Island nodes during decoding. Default value: `figure,table,longtable,landscape`. |
| `--no-island-wrap`      | Disable automatic Island wrapping of environments. Possible values: `true`, `false`.                            |
| `--island-style`        | Style to apply to auto-created Island nodes.                                                                    |
| `--input-losses`        | Action when there are losses decoding from input files. Default value: `debug`.                                 |
| `--include-structuring` | Structuring operations to include (comma-separated).                                                            |
| `--exclude-structuring` | Structuring operations to exclude (comma-separated).                                                            |
| `--citation-style`      | The citation style to assume for text-to-citation structuring.                                                  |
| `--strip-scopes`        | Scopes defining which properties of nodes should be stripped.                                                   |
| `--strip-types`         | A list of node types to strip.                                                                                  |
| `--strip-props`         | A list of node properties to strip.                                                                             |
| `-t, --to`              | The format of the output/s.                                                                                     |
| `--template`            | The template document to use.                                                                                   |
| `--reproducible`        | Encode executable nodes so that they are reproducible. Possible values: `true`, `false`.                        |
| `--highlight`           | Highlight the rendered outputs of executable nodes. Possible values: `true`, `false`.                           |
| `--no-highlight`        | Do not highlight the rendered outputs of executable nodes. Possible values: `true`, `false`.                    |
| `--standalone`          | Encode as a standalone document. Possible values: `true`, `false`.                                              |
| `--not-standalone`      | Do not encode as a standalone document when writing to file. Possible values: `true`, `false`.                  |
| `--theme`               | The CSS theme to use when encoding to HTML and HTML-derived formats.                                            |
| `--view`                | The document view to use when encoding to HTML and HTML-derived formats.                                        |
| `--embed-media`         | Embed media files as data URIs. Possible values: `true`, `false`.                                               |
| `--extract-media`       | Extract embedded media to a folder.                                                                             |
| `--embed-supplements`   | Embed supplemental files directly into the document. Possible values: `true`, `false`.                          |
| `--extract-supplements` | Extract embedded supplemental content to separate files.                                                        |
| `--recursive`           | Recursively encode the content of `IncludeBlock`s to their source file. Possible values: `true`, `false`.       |
| `--compact`             | Use a compact form of encoding if available. Possible values: `true`, `false`.                                  |
| `--pretty`              | Use a "pretty" form of encoding if available. Possible values: `true`, `false`.                                 |
| `--output-losses`       | Action when there are losses encoding to output files. Default value: `debug`.                                  |
| `--from-tool`           | The tool to use for decoding inputs.                                                                            |
| `--tool`                | The tool to use for encoding outputs (e.g. pandoc).                                                             |

**Possible values of `--citation-style`**

| Value                   | Description                              |
| ----------------------- | ---------------------------------------- |
| `author-year`           | Author-year citations like (Smith, 2023) |
| `bracketed-numeric`     | Bracketed numeric citations like [1]     |
| `parenthetic-numeric`   | Parenthetic numeric citations like (1)   |
| `superscripted-numeric` | Superscripted numeric citations like ¹   |

**Possible values of `--strip-scopes`**

| Value         | Description                                              |
| ------------- | -------------------------------------------------------- |
| `authors`     | Strip authorship properties of nodes                     |
| `provenance`  | Strip provenance properties of nodes                     |
| `metadata`    | Strip metadata properties of nodes                       |
| `content`     | Strip content properties of nodes                        |
| `archive`     | Strip archive properties of nodes                        |
| `temporary`   | Strip temporary properties of nodes                      |
| `code`        | Strip code properties of executable nodes                |
| `compilation` | Strip compilation related properties of executable nodes |
| `execution`   | Strip execution related properties of executable nodes   |
| `output`      | Strip output properties of executable nodes              |
| `timestamps`  | Strip timestamp properties                               |

**Possible values of `--include-structuring`, `--exclude-structuring`**

| Value                           | Description                                                         |
| ------------------------------- | ------------------------------------------------------------------- |
| `none`                          | No structuring operations                                           |
| `all`                           | All structuring operations                                          |
| `sections-to-keywords`          | Extract keywords from the "Keywords" section                        |
| `sections-to-abstract`          | Extract abstract from the "Abstract" section                        |
| `sections-to-references`        | Extract references from "References" section                        |
| `headings-to-title`             | Extract document title from the first heading                       |
| `heading1-to-title`             | Extract document title from the very first level 1 heading          |
| `heading1-to-title-single`      | Extract document title from a single level 1 heading (conservative) |
| `headings-decrement`            | Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)              |
| `headings-primary-level1`       | Ensure that all "primary" headings have level 1                     |
| `headings-to-sections`          | Create a section for each heading                                   |
| `headings-to-paragraphs`        | Transform headings to paragraphs if appropriate                     |
| `paragraphs-to-keywords`        | Extract keywords from paragraphs starting with "Keywords"           |
| `paragraphs-to-headings`        | Transform paragraphs to headings if appropriate                     |
| `paragraphs-to-sentences`       | Split paragraphs into individual sentences                          |
| `figures-with-captions`         | Combine an image with a figure caption before or after it           |
| `tables-with-captions`          | Combine a table caption with the following table or datatable       |
| `table-images-to-rows`          | Convert table images to table rows using OCR                        |
| `tables-to-datatables`          | Transform tables into datatables if possible                        |
| `unwrap-media-objects`          | Unwrap media objects from paragraphs to block level                 |
| `unwrap-quote-blocks`           | Unwrap quote blocks containing more than two child blocks           |
| `text-to-citations`             | Convert text to structured citations                                |
| `text-to-links`                 | Convert URL text to structured links                                |
| `math-to-citations`             | Convert math to structured citations                                |
| `math-images-to-tex`            | Convert math images to TeX code using OCR                           |
| `links-to-citations`            | Convert links to citations                                          |
| `normalize-citations`           | Normalize citation formatting and grouping                          |
| `remove-pre-primary`            | Remove content before the first primary heading                     |
| `remove-frontmatter-duplicates` | Remove front matter that duplicates article metadata                |
| `remove-empty-headings`         | Remove empty headings                                               |
| `remove-empty-tables`           | Remove empty tables and datatables                                  |
| `remove-empty-lists`            | Remove empty lists                                                  |
| `remove-empty-paragraphs`       | Remove empty paragraphs                                             |
| `remove-empty-text`             | Remove empty text                                                   |
