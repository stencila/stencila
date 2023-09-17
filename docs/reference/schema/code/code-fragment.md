---
title:
- type: Text
  value: CodeFragment
---

# Code Fragment

**Inline code.**

**`@id`**: `stencila:CodeFragment`

## Properties

The `CodeFragment` type has these properties:

| Name                | `@id`                                                                  | Type                                                               | Description                           | Inherited from                                                              |
| ------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------- | --------------------------------------------------------------------------- |
| id                  | [`schema:id`](https://schema.org/id)                                   | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item          | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)         |
| code                | `stencila:code`                                                        | [`Cord`](https://stencila.dev/docs/reference/schema/data/cord)     | The code.                             | [`CodeStatic`](https://stencila.dev/docs/reference/schema/code/code-static) |
| programmingLanguage | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The programming language of the code. | [`CodeStatic`](https://stencila.dev/docs/reference/schema/code/code-static) |

## Related

The `CodeFragment` type is related to these types:

- Parents: [`CodeStatic`](https://stencila.dev/docs/reference/schema/code/code-static)
- Children: none

## Formats

The `CodeFragment` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                                       |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<code>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code)                   |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<monospace>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/monospace) |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss      |              | 🚧 Under development    | Encoded using special function                                                                              |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                                             |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                             |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                             |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                             |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                                             |

## Bindings

The `CodeFragment` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/CodeFragment.jsonld)
- [JSON Schema](https://stencila.dev/CodeFragment.schema.json)
- Python class [`CodeFragment`](https://github.com/stencila/stencila/blob/main/python/stencila/types/code_fragment.py)
- Rust struct [`CodeFragment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_fragment.rs)
- TypeScript class [`CodeFragment`](https://github.com/stencila/stencila/blob/main/typescript/src/types/CodeFragment.ts)

## Source

This documentation was generated from [`CodeFragment.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeFragment.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).