---
title: Compilation Digest
description: A digest of the content, semantics and dependencies of an executable node.
config:
  publish:
    ghost:
      type: post
      slug: compilation-digest
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Flow
---

# Properties

The `CompilationDigest` type has these properties:

| Name                 | Description                                                                 | Type                                                                                  | Inherited from                                                     | `JSON-LD @id`                        | Aliases                                      |
| -------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | -------------------------------------------- |
| `id`                 | The identifier for this item.                                               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                                            |
| `stateDigest`        | A digest of the state of a node.                                            | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:stateDigest`               | `state-digest`, `state_digest`               |
| `semanticDigest`     | A digest of the semantics of the node with respect to the dependency graph. | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:semanticDigest`            | `semantic-digest`, `semantic_digest`         |
| `dependenciesDigest` | A digest of the semantic digests of the dependencies of a node.             | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:dependenciesDigest`        | `dependencies-digest`, `dependencies_digest` |
| `dependenciesStale`  | A count of the number of dependencies that are stale.                       | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:dependenciesStale`         | `dependencies-stale`, `dependencies_stale`   |
| `dependenciesFailed` | A count of the number of dependencies that failed.                          | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:dependenciesFailed`        | `dependencies-failed`, `dependencies_failed` |

# Related

The `CompilationDigest` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `CompilationDigest` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)      | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |         |

# Bindings

The `CompilationDigest` type is represented in:

- [JSON-LD](https://stencila.org/CompilationDigest.jsonld)
- [JSON Schema](https://stencila.org/CompilationDigest.schema.json)
- Python class [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/compilation_digest.py)
- Rust struct [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/compilation_digest.rs)
- TypeScript class [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/ts/src/types/CompilationDigest.ts)

# Source

This documentation was generated from [`CompilationDigest.yaml`](https://github.com/stencila/stencila/blob/main/schema/CompilationDigest.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
