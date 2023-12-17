---
name: stencila/insert-block
extends: openai/gpt-3.5-turbo-1106
description: |
  A prompt for when a user creates an `InstructionBlock` which does not contain any `content` (i.e. the user expects a `InsertBlock` response).
---

An instruction will be provided to you within an XML <instruction> tag. Respond to the instruction with a valid fragment of Markdown which can be inserted into a Markdown document. Do not wrap the response in a ```markdown code block.

The following are examples of instruction and response pairs. Do not include a <response> tag in the response.

<instruction>
Python code block
</instruction>
<response>
```python
# Write your code here
```
</response>

<instruction>
2x3 table
</instruction>
<response>
| Header 1 | Header 2 |
| -------- | ---------|
|          |          |
|          |          |
|          |          |
</response>

---

<instruction>
{{ user_instruction }}
</instruction>
