---
title: Code Expression
description: An executable code expression.
config:
  publish:
    ghost:
      type: post
      slug: code-expression
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Code
---

Note that `CodeExpression` nodes lack the `executionPure` property that `CodeChunk` nodes have because they should be side-effect free.

# Properties

The `CodeExpression` type has these properties:

| Name                    | Description                                                      | Type                                                                                           | Inherited from                                                                      | `JSON-LD @id`                                                          | Aliases                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)                  | [`schema:id`](https://schema.org/id)                                   | -                                                                                                                         |
| `executionMode`         | Under which circumstances the code should be executed.           | [`ExecutionMode`](https://stencila.ghost.io/docs/reference/schema/execution-mode)              | -                                                                                   | `stencila:executionMode`                                               | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:compilationDigest`                                           | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:compilationMessages`                                         | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDigest`                                             | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency)* | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDependencies`                                       | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDependants`                                         | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag)*               | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionTags`                                               | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                           | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionCount`                                              | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](https://stencila.ghost.io/docs/reference/schema/execution-required)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionRequired`                                           | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](https://stencila.ghost.io/docs/reference/schema/execution-status)          | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionStatus`                                             | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionInstance`                                           | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionEnded`                                              | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                         | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionDuration`                                           | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message)*       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)          | `stencila:executionMessages`                                           | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `code`                  | The code.                                                        | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                                 | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:code`                                                        | -                                                                                                                         |
| `programmingLanguage`   | The programming language of the code.                            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                                   | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language`                                                                            |
| `executionBounds`       | The environment in which code should be executed.                | [`ExecutionBounds`](https://stencila.ghost.io/docs/reference/schema/execution-bounds)          | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:executionBounds`                                             | `execution-bounds`, `execution_bounds`                                                                                    |
| `executionBounded`      | The execution bounds, if any, on the last execution.             | [`ExecutionBounds`](https://stencila.ghost.io/docs/reference/schema/execution-bounds)          | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:executionBounded`                                            | `execution-bounded`, `execution_bounded`                                                                                  |
| `authors`               | The authors of the executable code.                              | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                            | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | [`schema:author`](https://schema.org/author)                           | `author`                                                                                                                  |
| `provenance`            | A summary of the provenance of the code.                         | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*         | [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable) | `stencila:provenance`                                                  | -                                                                                                                         |
| `output`                | The value of the expression when it was last evaluated.          | [`Node`](https://stencila.ghost.io/docs/reference/schema/node)                                 | -                                                                                   | `stencila:output`                                                      | -                                                                                                                         |

# Related

The `CodeExpression` type is related to these types:

- Parents: [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable)
- Children: none

# Formats

The `CodeExpression` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | -------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                                                                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🔷 Low loss   | 🔷 Low loss   | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                                 |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Microsoft Word](https://stencila.ghost.io/docs/reference/formats/docx)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/czst)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                    |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                    |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                                                                                    |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                                                                                    |
| [Microsoft Excel](https://stencila.ghost.io/docs/reference/formats/xlsx)            |              |              |                                                                                                    |
| [Microsoft Excel (XLS)](https://stencila.ghost.io/docs/reference/formats/xls)       |              |              |                                                                                                    |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                                                                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              |              |                                                                                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                    |

# Bindings

The `CodeExpression` type is represented in:

- [JSON-LD](https://stencila.org/CodeExpression.jsonld)
- [JSON Schema](https://stencila.org/CodeExpression.schema.json)
- Python class [`CodeExpression`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_expression.py)
- Rust struct [`CodeExpression`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_expression.rs)
- TypeScript class [`CodeExpression`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeExpression.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `CodeExpression` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                                                                                     | Strategy                                    |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `code`   | Min+       | Generate a simple fixed string of code.                                                                                         | `Cord::from("code")`                        |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters (excludes whitespace which<br><br>can be problematic in Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|          | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|          | Max        | Generate an arbitrary string.                                                                                                   | `String::arbitrary().prop_map(Cord::from)`  |

# Source

This documentation was generated from [`CodeExpression.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeExpression.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
