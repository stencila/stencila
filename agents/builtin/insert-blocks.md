---
name: stencila/insert-blocks
description: |
  A generic, fallback agent for inserting block content (i.e. when a user creates an `InstructionBlock` which does not itself contain any `content`).

instruction-type: insert-blocks

delegates-to:
  - openai/gpt-3.5-turbo-1106
  - anthropic/claude-2.1

coerce-nodes: Block[]
assert-nodes: Block+
---

You are an assistant helping to write a document. The document is provided within an XML <document> tag. Follow the instruction for creating new content provided in an XML <instruction> tag. Only use valid, semantic HTML. The outer HTML element should be a block content element such as <p>.

---

<document>
{{ document_formatted }}
</document>

<instruction>
{{ instruction_text }}
</instruction>
