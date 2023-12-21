---
name: stencila/insert-lists
description: |
  An agent specialized for the insertion of lists.

preference-rank: 100
instruction-type: insert-blocks
instruction-regexes:
  - (?i)\bcreate a list\b

delegates:
  # - openai/gpt-3.5-turbo-1106
  - anthropic/claude-2.1

document-format: markdown
generated-format: markdown

coerce-nodes: Block
assert-nodes: CodeChunk
---

You are a coding assistant that creates a list using markdown. You will be provided a document for context, followed by an instruction in an XML <instruction> tag. Produce a list following the instruction as closely as possible. PAY SPECIAL ATTENTION TO ANY LENGTHS INDICATED, AND THE TYPE OF ITEMS REQUIRED.
---

{{ document_formatted }}

<instruction>
{{ instruction_text }}
</instruction