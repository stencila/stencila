---
title: "`stencila compare`"
description: Compare two documents
---

Compare two documents

Compares the two documents semantically, rather than as text, and reports how their nodes correspond and differ. Neither document is presumed correct: they are simply the left and right snapshots that you selected.

Use `--normalize whitespace,typography,initials` to explicitly classify cosmetic changes while retaining their raw values. Classification is disabled by default and does not change equality or exit status.

Supply several output paths to write multiple formats from one comparison. Use `--compact` for JSON without indentation, and `--locations` to add readable labels under `report.locations.left` and `report.locations.right`, indexed by node path. These labels are display metadata; the comparison and its node references remain unchanged. HTML always shows readable labels alongside the precise paths.

Exits with 0 when the documents are equal, 1 when they differ, and 2 on error.

# Usage

```sh
stencila compare [OPTIONS] <LEFT> <RIGHT> [OUTPUTS]...
```

# Examples

```bash
# Compare two documents in the terminal
stencila compare before.smd after.smd

# Compare documents in different formats
stencila compare original.smd roundtripped.docx

# Only report how many differences there are
stencila compare before.smd after.smd --summary

# Classify cosmetic differences while preserving the original values
stencila compare before.jats.xml after.jats.xml --normalize whitespace,typography,initials

# Open a side-by-side view of the comparison in a browser
stencila compare before.smd after.smd --view

# Write the side-by-side view to a HTML file, without opening it
stencila compare before.smd after.smd comparison.html

# Write the comparison artifact as JSON to stdout
stencila compare before.smd after.smd --to json

# Write the comparison artifact to a file
stencila compare before.smd after.smd comparison.yaml

# Write JSON and HTML from one comparison, with readable locations in the JSON
stencila compare before.smd after.smd comparison.json comparison.html --locations --compact

# Ignore differences that are only about labels
stencila compare before.smd after.jats.xml --exclude label

# Ignore labels everywhere except on figures
stencila compare before.smd after.jats.xml --exclude label --include Figure.label

# Ignore everything about links, and JATS reference types anywhere
stencila compare before.smd after.jats.xml --exclude Link --exclude jatsRefType

# Report nothing but heading differences
stencila compare before.smd after.smd --exclude all --include Heading

# Override the format of an input document
stencila compare before.txt after.smd --left-from smd

# Write the changes into the first document as suggestions and comments
stencila compare before.smd after.smd merged.smd

# Same, printed as Stencila Markdown
stencila compare before.smd after.smd --to suggestions
```

# Arguments

| Name        | Description                     |
| ----------- | ------------------------------- |
| `<LEFT>`    | The path of the left document.  |
| `<RIGHT>`   | The path of the right document. |
| `[OUTPUTS]` | The paths of the output files.  |

# Options

| Name                      | Description                                                                                |
| ------------------------- | ------------------------------------------------------------------------------------------ |
| `--left-from`             | The format of the left document.                                                           |
| `--right-from`            | The format of the right document.                                                          |
| `-t, --to`                | The format of the output.                                                                  |
| `--view`                  | Open a side-by-side view of the comparison in a browser. Possible values: `true`, `false`. |
| `--summary`               | Only report counts, not individual differences. Possible values: `true`, `false`.          |
| `--compact`               | Write JSON without indentation. Possible values: `true`, `false`.                          |
| `--locations`             | Include readable locations in JSON and YAML reports. Possible values: `true`, `false`.     |
| `--input-losses`          | Action when there are losses decoding either input document. Default value: `warn`.        |
| `--include`               | Only report differences matching this selector.                                            |
| `--exclude`               | Do not report differences matching this selector.                                          |
| `--normalize`             | Classify cosmetic differences using comma-separated normalization rules.                   |
| `--alignment-cell-budget` | The maximum number of candidate cells that sequence alignment may use.                     |

**Possible values of `--to`**

| Value         | Description                                                        |
| ------------- | ------------------------------------------------------------------ |
| `text`        | A human-readable report                                            |
| `html`        | A side-by-side report as a self-contained HTML page                |
| `json`        | The comparison artifact as JSON                                    |
| `yaml`        | The comparison artifact as YAML                                    |
| `suggestions` | The left document with the right document's changes as suggestions |

**Possible values of `--input-losses`**

| Value    | Description                    |
| -------- | ------------------------------ |
| `warn`   | Warn about losses and continue |
| `ignore` | Say nothing about losses       |
| `abort`  | Abort without comparing        |

**Possible values of `--normalize`**

| Value        | Description                                                                       |
| ------------ | --------------------------------------------------------------------------------- |
| `whitespace` | Collapse runs of Unicode whitespace to a single space                             |
| `typography` | Fold curly quotes, typographic dashes and ellipses to their ASCII forms           |
| `initials`   | Equate dotted or space-separated single-letter given names with separate initials |
