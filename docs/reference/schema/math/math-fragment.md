# Math Fragment

**A fragment of math, e.g a variable name, to be treated as inline content.**

**`@id`**: `stencila:MathFragment`

## Properties

The `MathFragment` type has these properties:

| Name          | `@id`                                | Type                                                                                                               | Description                                                    | Inherited from                                                                                   |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item.                                  | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| mathLanguage  | `stencila:mathLanguage`              | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The language used for the equation e.g tex, mathml, asciimath. | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| code          | `stencila:code`                      | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                        | The code of the equation in the `mathLanguage`.                | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md) | A digest of the `code` and `mathLanguage`.                     | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| errors        | `stencila:errors`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | Errors that occurred when parsing the math equation.           | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| mathml        | `stencila:mathml`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The MathML transpiled from the `code`.                         | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |

## Related

The `MathFragment` type is related to these types:

- Parents: [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)
- Children: none

## Formats

The `MathFragment` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                                                                             |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                                                         |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss        |              | 🚧 Under development    | Encoded to tag [`<inline-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-formula.html) using special function |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟢 No loss        |              | 🚧 Under development    | Encoded using special function                                                                                                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                                                                   |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                                   |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                                   |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                                   |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                                                                                   |

## Bindings

The `MathFragment` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/MathFragment.jsonld)
- [JSON Schema](https://stencila.dev/MathFragment.schema.json)
- Python class [`MathFragment`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math_fragment.py)
- Rust struct [`MathFragment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_fragment.rs)
- TypeScript class [`MathFragment`](https://github.com/stencila/stencila/blob/main/typescript/src/types/MathFragment.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `MathFragment` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any optional properties that are not in this table are set to `None`.

| Property       | Complexity | Description                                                                       | Strategy                                      |
| -------------- | ---------- | --------------------------------------------------------------------------------- | --------------------------------------------- |
| `mathLanguage` | Min+       | Generate a fixed string.                                                          | `String::from("lang")`                        |
|                | Low+       | Generate one of the math language names.                                          | Regex `(asciimath)\|(mathml)\|(tex)`          |
|                | High+      | Generate a random string of up to 10 alphanumeric characters.                     | Regex `[a-zA-Z0-9]{1,10}`                     |
|                | Max        | Generate an arbitrary string.                                                     | `String::arbitrary()`                         |
| `code`         | Min+       | Generate a simple fixed string of math.                                           | `Cord::new("math")`                           |
|                | Low+       | Generate a random string of up to 10 alphanumeric & space characters.             | `r"[a-zA-Z0-9 \t]{1,10}".prop_map(Cord::new)` |
|                | High+      | Generate a random string of up to 100 characters (excluding control characters).  | `r"[^\p{C}]{1,100}".prop_map(Cord::new)`      |
|                | Max        | Generate an arbitrary string.                                                     | `String::arbitrary().prop_map(Cord::new)`     |

## Source

This documentation was generated from [`MathFragment.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathFragment.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).