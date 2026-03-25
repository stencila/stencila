---
title: AT Protocol JSON
description: Encode Stencila documents as AT Protocol richtext records
---

# Introduction

[AT Protocol](https://atproto.com) (Authenticated Transfer Protocol) is the protocol underlying Bluesky and other decentralized social applications. AT Protocol represents rich text using a flat text string annotated with _facets_ — byte-range spans that carry typed features such as links, bold, and emphasis.

The Stencila AT Protocol codec encodes Stencila documents into AT Protocol-compatible JSON structures. It flattens the Stencila inline tree into richtext with facets, and encodes block-level content using [OXA](https://oxa.dev) block type identifiers.

This is an **encode-only** codec. Decoding from AT Protocol JSON back to Stencila documents is not currently supported.

# Usage

> [!warning] Under development
>
> This codec is still under active development. The JSON it produces should be
> considered provisional, and the exact encoding may change in future releases
> as support evolves — particularly while the OXA lexicon and related AT
> Protocol conventions are still being developed.

Use the `.atproto.json` file extension, or the `--to atprotojson` option, when converting to AT Protocol JSON:

```sh
stencila convert article.smd article.atproto.json
stencila convert doc.md doc.atproto.json
```

# Block Type Mappings

The following Stencila block types are supported:

| Stencila Type      | AT Protocol `$type`                 | Notes                                    |
| ------------------ | ----------------------------------- | ---------------------------------------- |
| `Paragraph`        | `pub.oxa.blocks.defs#paragraph`     | Text with facets                         |
| `Heading`          | `pub.oxa.blocks.defs#heading`       | Includes `level` field                   |
| `CodeBlock`        | `pub.oxa.blocks.defs#code`          | Includes `value` and optional `language` |
| `MathBlock`        | `pub.oxa.blocks.defs#math`          | Includes `tex` field                     |
| `ThematicBreak`    | `pub.oxa.blocks.defs#thematicBreak` | Type-only, no content                    |
| `QuoteBlock`       | `pub.oxa.blocks.defs#blockquote`    | Paragraphs joined with `\n`              |
| `List` (ordered)   | `pub.oxa.blocks.defs#orderedList`   | Recursive `children` with `startIndex`   |
| `List` (unordered) | `pub.oxa.blocks.defs#unorderedList` | Recursive `children`                     |

Unsupported block types (e.g. `Figure`, `Table`) are omitted from the output and a loss is recorded.

# Inline Type Mappings

Inline formatting is encoded as facets on the flat text string. Each facet has a byte-range index and one or more typed features from multiple "families" — different AT Protocol lexicons that understand the same formatting concept. This allows a single document to be rendered correctly by different applications:

- **OXA** (`pub.oxa.richtext.facet#*`): Full-fidelity features for OXA-aware applications
- **Leaflet** (`pub.leaflet.richtext.facet#*`): Compatibility features for Leaflet-aware applications
- **Bluesky** (`app.bsky.richtext.facet#*`): Link features for Bluesky-compatible applications (HTTP/HTTPS URLs only)

The current mapping of Stencila `Inline` node types to facet features is:

| Stencila Type | Facet Features                                               |
| ------------- | ------------------------------------------------------------ |
| `Emphasis`    | OXA emphasis + Leaflet italic                                |
| `Strong`      | OXA strong + Leaflet bold                                    |
| `CodeInline`  | OXA inlineCode + Leaflet code                                |
| `Strikeout`   | OXA strikethrough + Leaflet strikethrough                    |
| `Underline`   | OXA underline + Leaflet underline                            |
| `Subscript`   | OXA subscript (no Leaflet equivalent)                        |
| `Superscript` | OXA superscript (no Leaflet equivalent)                      |
| `Link`        | OXA link + Leaflet link (+ Bluesky link for HTTP/HTTPS URLs) |

# Provisional Lexicon Status

The OXA block and richtext NSIDs (Namespaced Identifiers) used by this codec are **provisional**. The OXA Lexicon is still under active development, and the specific type identifiers may change as the specification matures. The Leaflet richtext facet NSIDs are also subject to change.

When the OXA and Leaflet lexicons stabilize, the identifiers used by this codec will be updated accordingly.

# Known Limitations

- **Encode-only**: Decoding from AT Protocol JSON is not supported.
- **Article-only**: Only `Article` nodes can be encoded; other root node types return an error.
- **Dropped properties**: Block-level `id` and `classes` properties are not preserved.
- **Dropped article metadata**: `authors` and `abstract` fields are recorded as losses.
- **QuoteBlock flattening**: Multi-paragraph quote blocks are flattened to newline-joined text, losing block structure.
- **List constraints**: Mixed nesting types (e.g. ordered list containing unordered sub-list) and multi-block list items record losses.
- **Unsupported types**: Block and inline types not listed above are omitted with loss tracking.
