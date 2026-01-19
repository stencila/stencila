---
title: Code Expression
description: An executable code expression.
---

Note that `CodeExpression` nodes lack the `executionPure` property that `CodeChunk` nodes have because they should be side-effect free.

# Properties

The `CodeExpression` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from                           | `JSON-LD @id`                                                          | Aliases                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md)                  | [`schema:id`](https://schema.org/id)                                   | -                                                                                                                         |
| `executionMode`         | Under which circumstances the code should be executed.           | [`ExecutionMode`](./execution-mode.md)              | -                                        | `stencila:executionMode`                                               | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          | `stencila:compilationDigest`                                           | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md)          | `stencila:compilationMessages`                                         | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          | `stencila:executionDigest`                                             | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md)          | `stencila:executionDependencies`                                       | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md)          | `stencila:executionDependants`                                         | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md)          | `stencila:executionTags`                                               | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](./integer.md)                           | [`Executable`](./executable.md)          | `stencila:executionCount`                                              | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md)          | `stencila:executionRequired`                                           | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md)          | `stencila:executionStatus`                                             | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](./string.md)                             | [`Executable`](./executable.md)          | `stencila:executionInstance`                                           | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md)          | `stencila:executionEnded`                                              | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](./duration.md)                         | [`Executable`](./executable.md)          | `stencila:executionDuration`                                           | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md)          | `stencila:executionMessages`                                           | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `code`                  | The code.                                                        | [`Cord`](./cord.md)                                 | [`CodeExecutable`](./code-executable.md) | `stencila:code`                                                        | -                                                                                                                         |
| `programmingLanguage`   | The programming language of the code.                            | [`String`](./string.md)                             | -                                        | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language`                                                                            |
| `executionBounds`       | The environment in which code should be executed.                | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) | `stencila:executionBounds`                                             | `execution-bounds`, `execution_bounds`                                                                                    |
| `executionBounded`      | The execution bounds, if any, on the last execution.             | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) | `stencila:executionBounded`                                            | `execution-bounded`, `execution_bounded`                                                                                  |
| `authors`               | The authors of the executable code.                              | [`Author`](./author.md)*                            | [`CodeExecutable`](./code-executable.md) | [`schema:author`](https://schema.org/author)                           | `author`                                                                                                                  |
| `provenance`            | A summary of the provenance of the code.                         | [`ProvenanceCount`](./provenance-count.md)*         | [`CodeExecutable`](./code-executable.md) | `stencila:provenance`                                                  | -                                                                                                                         |
| `output`                | The value of the expression when it was last evaluated.          | [`Node`](./node.md)                                 | -                                        | `stencila:output`                                                      | -                                                                                                                         |

# Related

The `CodeExpression` type is related to these types:

- Parents: [`CodeExecutable`](./code-executable.md)
- Children: none

# Formats

The `CodeExpression` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                                                                                    |
| [JATS](../formats/jats.md)                       | 🔷 Low loss   | 🔷 Low loss   | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html) |
| [Markdown](../formats/md.md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                                 |
| [Stencila Markdown](../formats/smd.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Quarto Markdown](../formats/qmd.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [MyST Markdown](../formats/myst.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LLM Markdown](../formats/llmd.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                    |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                    |
| [Directory](../formats/directory.md)             |              |              |                                                                                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                    |

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
