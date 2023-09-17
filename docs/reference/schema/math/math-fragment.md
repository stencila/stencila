---
title:
- type: Text
  value: MathFragment
---

# Math Fragment

**A fragment of math, e.g a variable name, to be treated as inline content.**

**`@id`**: `stencila:MathFragment`

## Properties

The `MathFragment` type has these properties:

| Name          | `@id`                                | Type                                                                                  | Description                                                    | Inherited from                                                      |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------- |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The identifier for this item                                   | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| mathLanguage  | `stencila:mathLanguage`              | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The language used for the equation e.g tex, mathml, asciimath. | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| code          | `stencila:code`                      | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The code of the equation in the `mathLanguage`.                | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) | A digest of the `code` and `mathLanguage`.                     | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| errors        | `stencila:errors`                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | Errors that occurred when parsing the math equation.           | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| mathml        | `stencila:mathml`                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The MathML transpiled from the `code`.                         | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |

## Related

The `MathFragment` type is related to these types:

- Parents: [`Math`](https://stencila.dev/docs/reference/schema/math/math)
- Children: none

## Formats

The `MathFragment` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                     |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math) |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                           |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss      |              | 🚧 Under development    | Encoded using special function                                                            |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                           |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                           |

## Bindings

The `MathFragment` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/MathFragment.jsonld)
- [JSON Schema](https://stencila.dev/MathFragment.schema.json)
- Python class [`MathFragment`](https://github.com/stencila/stencila/blob/main/python/stencila/types/math_fragment.py)
- Rust struct [`MathFragment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_fragment.rs)
- TypeScript class [`MathFragment`](https://github.com/stencila/stencila/blob/main/typescript/src/types/MathFragment.ts)

## Source

This documentation was generated from [`MathFragment.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathFragment.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).