# Compilation Digest

**A digest of the content, semantics and dependencies of an executable node.**

**`@id`**: `stencila:CompilationDigest`

## Properties

The `CompilationDigest` type has these properties:

| Name                 | Aliases                                      | `@id`                                | Type                                                                                                               | Description                                                                 | Inherited from                                                                                   |
| -------------------- | -------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                 | -                                            | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item.                                               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `stateDigest`        | `state-digest`, `state_digest`               | `stencila:stateDigest`               | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | A digest of the state of a node.                                            | -                                                                                                |
| `semanticDigest`     | `semantic-digest`, `semantic_digest`         | `stencila:semanticDigest`            | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | A digest of the semantics of the node with respect to the dependency graph. | -                                                                                                |
| `dependenciesDigest` | `dependencies-digest`, `dependencies_digest` | `stencila:dependenciesDigest`        | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | A digest of the semantic digests of the dependencies of a node.             | -                                                                                                |
| `dependenciesStale`  | `dependencies-stale`, `dependencies_stale`   | `stencila:dependenciesStale`         | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | A count of the number of dependencies that are stale.                       | -                                                                                                |
| `dependenciesFailed` | `dependencies-failed`, `dependencies_failed` | `stencila:dependenciesFailed`        | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | A count of the number of dependencies that failed.                          | -                                                                                                |

## Related

The `CompilationDigest` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `CompilationDigest` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |       |

## Bindings

The `CompilationDigest` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CompilationDigest.jsonld)
- [JSON Schema](https://stencila.org/CompilationDigest.schema.json)
- Python class [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/compilation_digest.py)
- Rust struct [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/compilation_digest.rs)
- TypeScript class [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/ts/src/types/CompilationDigest.ts)

## Source

This documentation was generated from [`CompilationDigest.yaml`](https://github.com/stencila/stencila/blob/main/schema/CompilationDigest.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
