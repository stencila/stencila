---
title: For Block
description: Repeat a block content for each item in an array.
config:
  publish:
    ghost:
      type: post
      slug: for-block
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Flow
---

This type is marked as unstable and is subject to change.

# Properties

The `ForBlock` type has these properties:

| Name                    | Description                                                                   | Type                                                                                           | Inherited from                                                                      | `JSON-LD @id`                                                          | Aliases                                                                                                                   |
| ----------------------- | ----------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)                  | [`schema:id`](https://schema.org/id)                                   | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.                        | [`ExecutionMode`](https://stencila.ghost.io/docs/reference/schema/execution-mode)              | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionMode`                                               | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.              | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:compilationDigest`                                           | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                                  | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:compilationMessages`                                         | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDigest`                                             | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                                       | [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency)* | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDependencies`                                       | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                                       | [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDependants`                                         | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                                  | [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag)*               | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionTags`                                               | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.               | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                           | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionCount`                                              | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                | [`ExecutionRequired`](https://stencila.ghost.io/docs/reference/schema/execution-required)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionRequired`                                           | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.                  | [`ExecutionStatus`](https://stencila.ghost.io/docs/reference/schema/execution-status)          | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionStatus`                                             | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution.              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionInstance`                                           | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                                  | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionEnded`                                              | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                               | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                         | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDuration`                                           | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                                    | [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message)*       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionMessages`                                           | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `code`                  | The code.                                                                     | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                                 | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:code`                                                        | -                                                                                                                         |
| `programmingLanguage`   | The programming language of the code.                                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language`                                                                            |
| `executionBounds`       | The environment in which code should be executed.                             | [`ExecutionBounds`](https://stencila.ghost.io/docs/reference/schema/execution-bounds)          | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:executionBounds`                                             | `execution-bounds`, `execution_bounds`                                                                                    |
| `executionBounded`      | The execution bounds, if any, on the last execution.                          | [`ExecutionBounds`](https://stencila.ghost.io/docs/reference/schema/execution-bounds)          | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:executionBounded`                                            | `execution-bounded`, `execution_bounded`                                                                                  |
| `authors`               | The authors of the executable code.                                           | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                            | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | [`schema:author`](https://schema.org/author)                           | `author`                                                                                                                  |
| `provenance`            | A summary of the provenance of the code.                                      | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*         | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:provenance`                                                  | -                                                                                                                         |
| `variable`              | The name to give to the variable representing each item in the iterated array | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                                   | `stencila:variable`                                                    | -                                                                                                                         |
| `content`               | The content to repeat for each item                                           | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                              | -                                                                                   | `stencila:content`                                                     | -                                                                                                                         |
| `otherwise`             | The content to render if there are no items                                   | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                              | -                                                                                   | `stencila:otherwise`                                                   | -                                                                                                                         |
| `iterations`            | The content repeated for each iteration                                       | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                              | -                                                                                   | `stencila:iterations`                                                  | `iteration`                                                                                                               |

# Related

The `ForBlock` type is related to these types:

- Parents: [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable)
- Children: none

# Formats

The `ForBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                    |

# Bindings

The `ForBlock` type is represented in:

- [JSON-LD](https://stencila.org/ForBlock.jsonld)
- [JSON Schema](https://stencila.org/ForBlock.schema.json)
- Python class [`ForBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/for_block.py)
- Rust struct [`ForBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/for_block.rs)
- TypeScript class [`ForBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/ForBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `ForBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property              | Complexity | Description                                                                                                                               | Strategy                                      |
| --------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                                   | `Cord::from("code")`                          |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (excludes whitespace which<br><br>can be problematic in Markdown).           | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`   |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`     |
|                       | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary().prop_map(Cord::from)`    |
| `programmingLanguage` | Min+       | Generate a simple fixed string.                                                                                                           | `Some(String::from("lang"))`                  |
|                       | Low+       | Generate one of the well known programming language short names.                                                                          | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                             | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                       | Max        | Generate an arbitrary string.                                                                                                             | `option::of(String::arbitrary())`             |
| `variable`            | Min+       | Generate a fixed variable name.                                                                                                           | `String::from("item")`                        |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (and at most one underscore to avoid<br><br>a clash with Markdown emphasis). | Regex `[a-zA-Z_][a-zA-Z0-9]{0,9}`             |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | Regex `[^\p{C}]{1,100}`                       |
|                       | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary()`                         |
| `content`             | Min+       | A single simple paragraph.                                                                                                                | `vec![p([t("For content")])]`                 |
|                       | Low+       | Generate up to four arbitrary, non-recursive, block nodes.                                                                                | `vec_blocks_non_recursive(4)`                 |
|                       | Max        | Generate up to eight arbitrary, non-recursive, block nodes.                                                                               | `vec_blocks_non_recursive(8)`                 |
| `otherwise`           | Min+       | No otherwise clause.                                                                                                                      | `None`                                        |
|                       | Low+       | Generate up to two arbitrary, non-recursive, block nodes.                                                                                 | `option::of(vec_blocks_non_recursive(2))`     |
|                       | Max        | Generate up to four arbitrary, non-recursive, block nodes.                                                                                | `option::of(vec_blocks_non_recursive(4))`     |

# Source

This documentation was generated from [`ForBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/ForBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
