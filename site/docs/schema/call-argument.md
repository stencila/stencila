---
title: Call Argument
description: The value of a `Parameter` to call a document with.
---

A `CallArgument` extends `Parameter` by also has some of the same properties as `CodeExecutable` allowing
an expression to be used to set the value of the parameter.


This type is marked as experimental and is likely to change.

# Properties

The `CallArgument` type has these properties:

| Name                    | Description                                                                                            | Type                                                | Inherited from                  | `JSON-LD @id`                                                          | Aliases                                                                                                                   |
| ----------------------- | ------------------------------------------------------------------------------------------------------ | --------------------------------------------------- | ------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                                                          | [`String`](./string.md)                             | [`Entity`](./entity.md)         | [`schema:id`](https://schema.org/id)                                   | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.                                                 | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) | `stencila:executionMode`                                               | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.                                       | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:compilationDigest`                                           | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                                                           | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) | `stencila:compilationMessages`                                         | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                                         | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:executionDigest`                                             | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                                                                | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) | `stencila:executionDependencies`                                       | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                                                                | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) | `stencila:executionDependants`                                         | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                                                           | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) | `stencila:executionTags`                                               | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.                                        | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) | `stencila:executionCount`                                              | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                                         | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) | `stencila:executionRequired`                                           | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.                                           | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) | `stencila:executionStatus`                                             | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution.                                       | [`String`](./string.md)                             | [`Executable`](./executable.md) | `stencila:executionInstance`                                           | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                                                           | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) | `stencila:executionEnded`                                              | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                                                        | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) | `stencila:executionDuration`                                           | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                                                             | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) | `stencila:executionMessages`                                           | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `name`                  | The name of the parameter.                                                                             | [`String`](./string.md)                             | [`Parameter`](./parameter.md)   | [`schema:name`](https://schema.org/name)                               | -                                                                                                                         |
| `label`                 | A short label for the parameter.                                                                       | [`String`](./string.md)                             | [`Parameter`](./parameter.md)   | `stencila:label`                                                       | -                                                                                                                         |
| `value`                 | The current value of the argument.                                                                     | [`Node`](./node.md)                                 | -                               | [`schema:value`](https://schema.org/value)                             | -                                                                                                                         |
| `default`               | The default value of the parameter.                                                                    | [`Node`](./node.md)                                 | [`Parameter`](./parameter.md)   | [`schema:defaultValue`](https://schema.org/defaultValue)               | -                                                                                                                         |
| `validator`             | The validator that the value is validated against.                                                     | [`Validator`](./validator.md)                       | [`Parameter`](./parameter.md)   | `stencila:validator`                                                   | -                                                                                                                         |
| `derivedFrom`           | The dotted path to the object (e.g. a database table column) that the parameter should be derived from | [`String`](./string.md)                             | [`Parameter`](./parameter.md)   | `stencila:derivedFrom`                                                 | `derived-from`, `derived_from`                                                                                            |
| `code`                  | The code to be evaluated for the parameter.                                                            | [`Cord`](./cord.md)                                 | -                               | `stencila:code`                                                        | -                                                                                                                         |
| `programmingLanguage`   | The programming language of the code.                                                                  | [`String`](./string.md)                             | -                               | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language`                                                                            |

# Related

The `CallArgument` type is related to these types:

- Parents: [`Parameter`](./parameter.md)
- Children: none

# Formats

The `CallArgument` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              |                                    |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                    |
| [CSV](../formats/csv.md)                         |              |              |                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                    |
| [Directory](../formats/directory.md)             |              |              |                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                    |

# Bindings

The `CallArgument` type is represented in:

- [JSON-LD](https://stencila.org/CallArgument.jsonld)
- [JSON Schema](https://stencila.org/CallArgument.schema.json)
- Python class [`CallArgument`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/call_argument.py)
- Rust struct [`CallArgument`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/call_argument.rs)
- TypeScript class [`CallArgument`](https://github.com/stencila/stencila/blob/main/ts/src/types/CallArgument.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `CallArgument` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                                                                                               | Strategy                                    |
| -------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `name`   | Min+       | Generate a fixed name.                                                                                                                    | `String::from("name")`                      |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters (and at most one underscore to avoid<br><br>a clash with Markdown emphasis). | Regex `[a-zA-Z_][a-zA-Z0-9]{0,9}`           |
|          | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | Regex `[^\p{C}]{1,100}`                     |
|          | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary()`                       |
| `code`   | Min+       | Generate a simple fixed string of code.                                                                                                   | `Cord::from("code")`                        |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters.                                                                             | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|          | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|          | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary().prop_map(Cord::from)`  |

# Source

This documentation was generated from [`CallArgument.yaml`](https://github.com/stencila/stencila/blob/main/schema/CallArgument.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
