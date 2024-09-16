# Math Inline

**A fragment of math, e.g a variable name, to be treated as inline content.**

**`@id`**: `stencila:MathInline`

## Properties

The `MathInline` type has these properties:

| Name                  | Aliases                                                                                                            | `@id`                                        | Type                                                                                                                      | Description                                                         | Inherited from                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                  | -                                                                                                                  | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The identifier for this item.                                       | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `code`                | -                                                                                                                  | `stencila:code`                              | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                               | The code of the equation in the `mathLanguage`.                     | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `mathLanguage`        | `math-language`, `math_language`                                                                                   | `stencila:mathLanguage`                      | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The language used for the equation e.g tex, mathml, asciimath.      | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `authors`             | `author`                                                                                                           | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                         | The authors of the math.                                            | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `provenance`          | -                                                                                                                  | `stencila:provenance`                        | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)*      | A summary of the provenance of the math.                            | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `compilationDigest`   | `compilation-digest`, `compilation_digest`                                                                         | `stencila:compilationDigest`                 | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)    | A digest of the `code` and `mathLanguage`.                          | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `compilationMessages` | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` | `stencila:compilationMessages`               | [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-message.md)* | Messages generated while parsing and compiling the math expression. | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |
| `mathml`              | -                                                                                                                  | `stencila:mathml`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The MathML transpiled from the `code`.                              | [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)      |

## Related

The `MathInline` type is related to these types:

- Parents: [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md)
- Children: none

## Formats

The `MathInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                                                                                                                                         |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🚧 Under development |                                                                                                                                               |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                                                         |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss    | 🔷 Low loss | 🚧 Under development | Encoded as [`<inline-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-formula.html) using special function |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | 🟢 No loss    | 🟢 No loss  | ⚠️ Alpha            | Encoded using implemented function                                                                                                            |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | 🟢 No loss    | 🟢 No loss  | ⚠️ Alpha            |                                                                                                                                               |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | ⚠️ Alpha            |                                                                                                                                               |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                               |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                               |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                               |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                                                               |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                               |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                               |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                               |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                                                                                                                               |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | 🚧 Under development |                                                                                                                                               |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                                                                                                                               |

## Bindings

The `MathInline` type is represented in these bindings:

- [JSON-LD](https://stencila.org/MathInline.jsonld)
- [JSON Schema](https://stencila.org/MathInline.schema.json)
- Python class [`MathInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math_inline.py)
- Rust struct [`MathInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_inline.rs)
- TypeScript class [`MathInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/MathInline.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `MathInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property       | Complexity | Description                                                                                                                                          | Strategy                                    |
| -------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `code`         | Min+       | Generate a simple fixed string of math.                                                                                                              | `Cord::from("math")`                        |
|                | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which <br><br>when leading or trailing causes issues for Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|                | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                     | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary().prop_map(Cord::from)`  |
| `mathLanguage` | Min+       | No language.                                                                                                                                         | `None`                                      |
|                | Low+       | Fixed as TeX (for testing with Markdown which uses dollars to delimit TeX by default)                                                                | `Some(String::from("tex"))`                 |
|                | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                                        | `option::of(r"[a-zA-Z0-9]{1,10}")`          |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `option::of(String::arbitrary())`           |

## Source

This documentation was generated from [`MathInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathInline.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
