# Link

**A hyperlink to other pages, sections within the same document, resources, or any URL.**

Analogues of `Link` in other schema include:
  - HTML [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)
  - JATS XML [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/ext-link.html)
  - MDAST [`Link`](https://github.com/syntax-tree/mdast#link)
  - OpenDocument [`<text:a>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415212_253892949)
  - Pandoc [`Link`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L270)


**`@id`**: `stencila:Link`

## Properties

The `Link` type has these properties:

| Name    | `@id`                                                            | Type                                                                                              | Description                                            | Inherited from                                                                                   |
| ------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| id      | [`schema:id`](https://schema.org/id)                             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item                           | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| content | `stencila:content`                                               | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The textual content of the link.                       | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |
| target  | `stencila:target`                                                | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The target of the link.                                | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |
| title   | [`schema:headline`](https://schema.org/headline)                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | A title for the link.                                  | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |
| rel     | [`schema:linkRelationship`](https://schema.org/linkRelationship) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The relation between the target and the current thing. | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |

## Related

The `Link` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Link` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                               |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🔷 Low loss       |              | 🚧 Under development    |                                                                                     |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🔷 Low loss       |              | 🚧 Under development    | Encoded using template `[{content}]({target})`                                      |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                     |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                     |

## Bindings

The `Link` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Link.jsonld)
- [JSON Schema](https://stencila.dev/Link.schema.json)
- Python class [`Link`](https://github.com/stencila/stencila/blob/main/python/stencila/types/link.py)
- Rust struct [`Link`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/link.rs)
- TypeScript class [`Link`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Link.ts)

## Source

This documentation was generated from [`Link.yaml`](https://github.com/stencila/stencila/blob/main/schema/Link.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).