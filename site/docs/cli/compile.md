---
title: "`stencila compile`"
description: Compile a document
---

Compile a document

# Usage

```sh
stencila compile [OPTIONS] <INPUT>
```

# Examples

```bash
# Compile a document to check for errors
stencila compile document.md

# Compile and cache document
stencila compile temp.md --cache
```

# Arguments

| Name      | Description                          |
| --------- | ------------------------------------ |
| `<INPUT>` | The path of the document to compile. |

# Options

| Name                    | Description                                                                                                     |
| ----------------------- | --------------------------------------------------------------------------------------------------------------- |
| `--no-save`             | Do not save the document after compiling it. Possible values: `true`, `false`.                                  |
| `--cache`               | Cache the document after compiling it. Possible values: `true`, `false`.                                        |
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

**Possible values of `--citation-style`**

| Value                   | Description                              |
| ----------------------- | ---------------------------------------- |
| `author-year`           | Author-year citations like (Smith, 2023) |
| `bracketed-numeric`     | Bracketed numeric citations like [1]     |
| `parenthetic-numeric`   | Parenthetic numeric citations like (1)   |
| `superscripted-numeric` | Superscripted numeric citations like ¹   |

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

# Note

Compiling a document checks for source path errors in
include and call blocks and prepares the document for
execution without actually running any code.
