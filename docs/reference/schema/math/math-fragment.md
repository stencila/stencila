# Math Fragment

**A fragment of math, e.g a variable name, to be treated as inline content.**

**`@id`**: `stencila:MathFragment`

## Properties

The `MathFragment` type has these properties:

| Name          | `@id`                                | Type                                                                                                               | Description                                                    | Inherited from                                                                                   |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item                                   | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| mathLanguage  | `stencila:mathLanguage`              | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The language used for the equation e.g tex, mathml, asciimath. | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| code          | `stencila:code`                      | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The code of the equation in the `mathLanguage`.                | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md) | A digest of the `code` and `mathLanguage`.                     | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| errors        | `stencila:errors`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | Errors that occurred when parsing the math equation.           | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| mathml        | `stencila:mathml`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The MathML transpiled from the `code`.                         | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |

## Related

The `MathFragment` type is related to these types:

- Parents: [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)
- Children: none

## Formats

The `MathFragment` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                                                                        |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss        |              | 🚧 Under development    | Encoded to tag [`<inline-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-formula) using special function |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟢 No loss        |              | 🚧 Under development    | Encoded using special function                                                                                                               |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                                                              |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                              |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                              |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                              |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                                                                              |

## Bindings

The `MathFragment` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/MathFragment.jsonld)
- [JSON Schema](https://stencila.dev/MathFragment.schema.json)
- Python class [`MathFragment`](https://github.com/stencila/stencila/blob/main/python/stencila/types/math_fragment.py)
- Rust struct [`MathFragment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_fragment.rs)
- TypeScript class [`MathFragment`](https://github.com/stencila/stencila/blob/main/typescript/src/types/MathFragment.ts)

## Source

This documentation was generated from [`MathFragment.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathFragment.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).