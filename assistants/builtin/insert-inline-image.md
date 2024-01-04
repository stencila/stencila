---
version: "0.1.0"

# Specify the generation task should output an image
# This will limit delegates to those that support
# text-to-image generation tasks.
task-output: image

# Match instructions to insert an inline image.
preference-rank: 100
instruction-type: insert-inlines
instruction-regexes:
  - (?i)\bimage\b

# Attempt to transform generated nodes to images and assert
# that there is only one.
transform-nodes: ImageObject
assert-nodes: ^ImageObject$

# Preliminary testing indicated poor results when using XML tags or other
# formatting in prompts for text-to-image LLMs. Given that the document is provided
# in plain text as context following the user instruction.
# For `openai/dall-e-2` at least, the prompt is restricted to 1000 characters, not tokens,
# so only the first part of the document will be included. 
document-format: text
---

An assistant specialized for inserting an inline `ImageObject`.

---

---

{% filter trim_end_chars(context_length) %}
{{ instruction_text }} inspired by {{ document_formatted }}
{% endfilter %}
