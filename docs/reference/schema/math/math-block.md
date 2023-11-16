# Math Block

**A block of math, e.g an equation, to be treated as block content.**

**`@id`**: `stencila:MathBlock`

## Properties

The `MathBlock` type has these properties:

| Name                | Aliases                                                                                                  | `@id`                                | Type                                                                                                                   | Description                                                      | Inherited from                                                                                   |
| ------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                | -                                                                                                        | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                        | The identifier for this item.                                    | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `code`              | -                                                                                                        | `stencila:code`                      | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                            | The code of the equation in the `mathLanguage`.                  | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `mathLanguage`      | `math-language`, `math_language`                                                                         | `stencila:mathLanguage`              | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                        | The language used for the equation e.g tex, mathml, asciimath.   | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `compilationDigest` | `compilation-digest`, `compilation_digest`                                                               | `stencila:compilationDigest`         | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md) | A digest of the `code` and `mathLanguage`.                       | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `compilationErrors` | `compilation-errors`, `compilation_errors`, `compilationError`, `compilation-error`, `compilation_error` | `stencila:compilationErrors`         | [`CompilationError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-error.md)*  | Errors generated when parsing and compiling the math expression. | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `mathml`            | -                                                                                                        | `stencila:mathml`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                        | The MathML transpiled from the `code`.                           | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `label`             | -                                                                                                        | `stencila:label`                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                        | A short label for the math block.                                | -                                                                                                |

## Related

The `MathBlock` type is related to these types:

- Parents: [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)
- Children: none

## Formats

The `MathBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding      | Status                 | Notes                                                                                                                                     |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------- | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |               | 🚧 Under development    | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                                                     |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss        | 🔷 Low loss    | 🚧 Under development    | Encoded as [`<disp-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/disp-formula.html) using special function |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟢 No loss        | 🟢 No loss     | ⚠️ Alpha               | Encoded using special function                                                                                                            |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |               | ⚠️ Alpha               |                                                                                                                                           |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                                                           |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                                                           |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                                                           |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |               | 🟢 Stable               |                                                                                                                                           |

## Bindings

The `MathBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/MathBlock.jsonld)
- [JSON Schema](https://stencila.dev/MathBlock.schema.json)
- Python class [`MathBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math_block.py)
- Rust struct [`MathBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_block.rs)
- TypeScript class [`MathBlock`](https://github.com/stencila/stencila/blob/main/typescript/src/types/MathBlock.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `MathBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property       | Complexity | Description                                                                                                                                          | Strategy                                   |
| -------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `code`         | Min+       | Generate a simple fixed string of math.                                                                                                              | `Cord::new("math")`                        |
|                | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which <br><br>when leading or trailing causes issues for Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::new)` |
|                | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                     | `r"[^\p{C}]{1,100}".prop_map(Cord::new)`   |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary().prop_map(Cord::new)`  |
| `mathLanguage` | Min+       | No language.                                                                                                                                         | `None`                                     |
|                | Low+       | Fixed as TeX (for testing with Markdown which uses dollars to delimit TeX by default)                                                                | `Some(String::from("tex"))`                |
|                | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                                        | `option::of(r"[a-zA-Z0-9]{1,10}")`         |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `option::of(String::arbitrary())`          |

## Source

This documentation was generated from [`MathBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathBlock.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.