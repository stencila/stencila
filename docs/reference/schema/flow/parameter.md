---
title: Parameter
description: A parameter of a document.
config:
  publish:
    ghost:
      type: post
      slug: parameter
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Flow
---

This type is marked as unstable and is subject to change.

# Properties

The `Parameter` type has these properties:

| Name                    | Description                                                                                            | Type                                                                                           | Inherited from                                                             | `JSON-LD @id`                                            | Aliases                                                                                                                   |
| ----------------------- | ------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)         | [`schema:id`](https://schema.org/id)                     | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.                                                 | [`ExecutionMode`](https://stencila.ghost.io/docs/reference/schema/execution-mode)              | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionMode`                                 | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.                                       | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:compilationDigest`                             | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                                                           | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:compilationMessages`                           | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                                         | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDigest`                               | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                                                                | [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency)* | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDependencies`                         | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                                                                | [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDependants`                           | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                                                           | [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag)*               | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionTags`                                 | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.                                        | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                           | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionCount`                                | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                                         | [`ExecutionRequired`](https://stencila.ghost.io/docs/reference/schema/execution-required)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionRequired`                             | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.                                           | [`ExecutionStatus`](https://stencila.ghost.io/docs/reference/schema/execution-status)          | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionStatus`                               | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution.                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionInstance`                             | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                                                           | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionEnded`                                | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                                                        | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                         | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDuration`                             | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                                                             | [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message)*       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionMessages`                             | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `name`                  | The name of the parameter.                                                                             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                          | [`schema:name`](https://schema.org/name)                 | -                                                                                                                         |
| `label`                 | A short label for the parameter.                                                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                          | `stencila:label`                                         | -                                                                                                                         |
| `value`                 | The current value of the parameter.                                                                    | [`Node`](https://stencila.ghost.io/docs/reference/schema/node)                                 | -                                                                          | [`schema:value`](https://schema.org/value)               | -                                                                                                                         |
| `default`               | The default value of the parameter.                                                                    | [`Node`](https://stencila.ghost.io/docs/reference/schema/node)                                 | -                                                                          | [`schema:defaultValue`](https://schema.org/defaultValue) | -                                                                                                                         |
| `validator`             | The validator that the value is validated against.                                                     | [`Validator`](https://stencila.ghost.io/docs/reference/schema/validator)                       | -                                                                          | `stencila:validator`                                     | -                                                                                                                         |
| `derivedFrom`           | The dotted path to the object (e.g. a database table column) that the parameter should be derived from | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                          | `stencila:derivedFrom`                                   | `derived-from`, `derived_from`                                                                                            |

# Related

The `Parameter` type is related to these types:

- Parents: [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)
- Children: [`CallArgument`](https://stencila.ghost.io/docs/reference/schema/call-argument)

# Formats

The `Parameter` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                                      | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ------------------------------------------------------------------------------------------------------------ | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                              |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              | Encoded using special function                                                                               |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              | Encoded as [`<parameter>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/parameter.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                                           |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                              |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                              |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                                              |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                              |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                              |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                                                                                              |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                                                                                              |
| [Microsoft Excel XLSX](https://stencila.ghost.io/docs/reference/formats/xlsx)       |              |              |                                                                                                              |
| [Microsoft Excel XLS](https://stencila.ghost.io/docs/reference/formats/xls)         |              |              |                                                                                                              |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                                                                                              |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                              |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                              |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                              |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                              |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                                              |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                              |

# Bindings

The `Parameter` type is represented in:

- [JSON-LD](https://stencila.org/Parameter.jsonld)
- [JSON Schema](https://stencila.org/Parameter.schema.json)
- Python class [`Parameter`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/parameter.py)
- Rust struct [`Parameter`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/parameter.rs)
- TypeScript class [`Parameter`](https://github.com/stencila/stencila/blob/main/ts/src/types/Parameter.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Parameter` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                                                                                               | Strategy                          |
| -------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| `name`   | Min+       | Generate a fixed name.                                                                                                                    | `String::from("name")`            |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters (and at most one underscore to avoid<br><br>a clash with Markdown emphasis). | Regex `[a-zA-Z_][a-zA-Z0-9]{0,9}` |
|          | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | Regex `[^\p{C}]{1,100}`           |
|          | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary()`             |

# Source

This documentation was generated from [`Parameter.yaml`](https://github.com/stencila/stencila/blob/main/schema/Parameter.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
