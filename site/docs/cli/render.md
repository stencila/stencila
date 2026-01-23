---
title: "`stencila render`"
description: Render a document
---

Render a document

# Usage

```sh
stencila render [OPTIONS] [INPUT] [OUTPUTS]... [-- <ARGUMENTS>...]
```

# Examples

```bash
# Render a document and preview in browser
stencila render document.smd

# Render to a specific output format
stencila render report.md report.docx

# Render to multiple formats
stencila render analysis.md output.html output.pdf

# Render from stdin to stdout
echo "# Hello" | stencila render --to html

# Render with document parameters
stencila render template.md output.html -- --name="John" --year=2024

# Render ignoring execution errors
stencila render notebook.md report.pdf --ignore-errors

# Render and cache a document
stencila render temp.md output.html --cache

# Spread render with multiple parameter combinations (grid)
stencila render report.md 'report-{region}-{species}.pdf' -- region=north,south species=ABC,DEF

# Spread render with positional pairing (zip) and output to nested folders
stencila render report.md '{region}/{species}/report.pdf' --spread=zip -- region=north,south species=ABC,DEF

# Spread render with explicit cases
stencila render report.md 'report-{i}.pdf' --spread=cases --case="region=north species=ABC" --case="region=south species=DEF"
```

# Arguments

| Name          | Description                         |
| ------------- | ----------------------------------- |
| `[INPUT]`     | The path of the document to render. |
| `[OUTPUTS]`   | The paths of desired output files.  |
| `[ARGUMENTS]` | Arguments to pass to the document.  |

# Options

| Name                    | Description                                                                                                     |
| ----------------------- | --------------------------------------------------------------------------------------------------------------- |
| `--spread`              | Enable multi-variant (spread) execution mode.                                                                   |
| `--spread-max`          | Maximum number of runs allowed in spread mode (default: 100). Default value: `100`.                             |
| `--case`                | Explicit parameter sets for cases mode.                                                                         |
| `--cache`               | Cache the document after rendering it. Possible values: `true`, `false`.                                        |
| `--ignore-errors`       | Ignore any errors while executing document. Possible values: `true`, `false`.                                   |
| `--force-all`           | Re-execute all node types regardless of current state. Possible values: `true`, `false`.                        |
| `--skip-code`           | Skip executing code. Possible values: `true`, `false`.                                                          |
| `--skip-instructions`   | Skip executing instructions. Possible values: `true`, `false`.                                                  |
| `--retain-suggestions`  | Retain existing suggestions for instructions. Possible values: `true`, `false`.                                 |
| `--force-unreviewed`    | Re-execute instructions with suggestions that have not yet been reviewed. Possible values: `true`, `false`.     |
| `--force-accepted`      | Re-execute instructions with suggestions that have been accepted. Possible values: `true`, `false`.             |
| `--skip-rejected`       | Skip re-executing instructions with suggestions that have been rejected. Possible values: `true`, `false`.      |
| `--dry-run`             | Prepare, but do not actually perform, execution tasks. Possible values: `true`, `false`.                        |
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
| `--strip-scopes`        | Scopes defining which properties of nodes should be stripped.                                                   |
| `--strip-types`         | A list of node types to strip.                                                                                  |
| `--strip-props`         | A list of node properties to strip.                                                                             |

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

**Possible values of `--spread`**

| Value   | Description                                            |
| ------- | ------------------------------------------------------ |
| `grid`  | Cartesian product of multi-valued parameters (default) |
| `zip`   | Positional pairing of multi-valued parameters          |
| `cases` | Explicitly enumerated parameter sets via `--case`      |

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
