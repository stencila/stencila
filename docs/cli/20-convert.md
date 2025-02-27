---
title: Conversion
description: Using the Stencila CLI to convert between document file formats
config:
  publish:
    ghost:
      slug: cli-convert
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---

```sh
stencila convert
```

# Overview

The Stencila convert command is a powerful part of how Stencila can represent documents in a number of formats. Convert lets you move from any format (called codecs in Stencila) supported by Stencila to any other. To see the codecs / formats that are supported you can run `stencila codecs` see the documentation seciton on Formats for more information on each format.

# Usage

## Basic usage is as follows: 

```sh
stencila convert source.format target.format
```
Where `source.format` replaced `.format` with one of the file extensions shown in the 'From' column of `stencila codecs` and `target.format` is a format in the 'To' column of `stencila codecs`

### Example:

We start with a file called `test.smd`: 

```markdown
# Header 1

This is a **test** document

* With
* Some Markdown
```

If we run:
```sh
stencila convert test.smd test.yaml
```

We'll see that `test.yaml` contains:

```yaml
$schema: https://stencila.org/v2.0.2/Article.schema.json
'@context': https://stencila.org/v2.0.2/context.jsonld
type: Article
content:
- type: Heading
  level: 1
  content:
  - type: Text
    value:
      string: Header 1
- type: Paragraph
  content:
  - type: Text
    value:
      string: 'This is a '
  - type: Strong
    content:
    - type: Text
      value:
        string: test
  - type: Text
    value:
      string: ' document'
- type: List
  items:
  - type: ListItem
    content:
    - type: Paragraph
      content:
      - type: Text
        value:
          string: With
  - type: ListItem
    content:
    - type: Paragraph
      content:
      - type: Text
        value:
          string: Some Markdown
  order: Unordered
```

# Advanced Usage 

## Dependencies for some Formats

Some codecs in Stencila require third party utilities installed `.docx`, `.odt`, `.tex` all require pandoc as we use the Pandoc-JSON format as an intermediate format. Converting to `.pdf` requires LaTeX to be available as well.

## Passthrough parameters

The `stencila convert` command allows for passthrough parameters to pandoc (for codecs that call out to pandoc as detailed above). This allows you to pass things like `--reference-doc` to Pandoc enabling your own document (`.docx`, or `.odt`) template for your Stencila outputs.
