---
title:
- type: Text
  value: Math
---

# Math

**Abstract base type for a mathematical variable or equation.**

This is a base type for `MathFragment` and `MathBlock` and should not
normally be instantiated.
This type has a similar structure and purpose to `CodeStatic` which is a base type
for `CodeFragment`, `CodeBlock` etc.


**`@id`**: `stencila:Math`

## Properties

The `Math` type has these properties:

| Name          | `@id`                                | Type                                                                                  | Description                                                    | Inherited from                                                      |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------- |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The identifier for this item                                   | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| mathLanguage  | `stencila:mathLanguage`              | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The language used for the equation e.g tex, mathml, asciimath. | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| code          | `stencila:code`                      | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The code of the equation in the `mathLanguage`.                | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) | A digest of the `code` and `mathLanguage`.                     | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| errors        | `stencila:errors`                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | Errors that occurred when parsing the math equation.           | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |
| mathml        | `stencila:mathml`                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The MathML transpiled from the `code`.                         | [`Math`](https://stencila.dev/docs/reference/schema/math/math)      |

## Related

The `Math` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: [`MathBlock`](https://stencila.dev/docs/reference/schema/math/math-block), [`MathFragment`](https://stencila.dev/docs/reference/schema/math/math-fragment)

## Bindings

The `Math` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Math.jsonld)
- [JSON Schema](https://stencila.dev/Math.schema.json)
- Python class [`Math`](https://github.com/stencila/stencila/blob/main/python/stencila/types/math.py)
- Rust struct [`Math`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math.rs)
- TypeScript class [`Math`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Math.ts)

## Source

This documentation was generated from [`Math.yaml`](https://github.com/stencila/stencila/blob/main/schema/Math.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).