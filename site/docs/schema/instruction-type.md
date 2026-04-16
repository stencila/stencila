---
title: Instruction Type
description: An operation requested by an instruction.
---

This is an enumeration used in Stencila Schema describing the operation requested by
an instruction.

It exists so editing workflows can distinguish actions such as creating,
revising, or describing content using a controlled vocabulary. This allows
prompts, tools, and interfaces to select behavior consistently.

See [`Instruction.instructionType`](./instruction.md#instructiontype) and
related properties for where this enumeration is used.


# Members

The `InstructionType` type has these members:

| Member     | Description                                                                                                                                                                                                                                                               |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Discuss`  | Discuss document, kernel, workspace or other contexts. Normally only used for `Chat`s.                                                                                                                                                                                    |
| `Create`   | Create new document content, usually a single document node (e.g. `Paragraph` or `Table`), ignoring any existing content nested within the instruction. The instruction message will normally include the type of content to produce (e.g. "paragraph", "table", "list"). |
| `Describe` | Describe other document content. The instruction message should indicate the target for the description e.g. "describe figure 1", "describe next", "describe prev output"                                                                                                 |
| `Edit`     | Edit existing document nodes. Expected to return the same node types as existing nodes.                                                                                                                                                                                   |
| `Fix`      | Fix an existing document node, usually a `CodeChunk`, `CodeInline`, or `MathBlock`. Expected to return the same node type without any `compilationErrors` or `executionErrors`.                                                                                           |

# Bindings

The `InstructionType` type is represented in:

- [JSON-LD](https://stencila.org/InstructionType.jsonld)
- [JSON Schema](https://stencila.org/InstructionType.schema.json)
- Python type [`InstructionType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`InstructionType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction_type.rs)
- TypeScript type [`InstructionType`](https://github.com/stencila/stencila/blob/main/ts/src/types/InstructionType.ts)

***

This documentation was generated from [`InstructionType.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructionType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
