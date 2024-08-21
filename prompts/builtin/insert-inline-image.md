---
version: "0.1.0"

preference-rank: 100
instruction-type: insert-inlines
instruction-examples:
  - image

task-output: image
expected-nodes: ImageObject
---

An assistant specialized for inserting an inline `ImageObject`.

Preliminary testing indicated poor results when adding the document or other context (either as plain text or formatted) to the prompt. Given that, this assistant has no system prompt.
