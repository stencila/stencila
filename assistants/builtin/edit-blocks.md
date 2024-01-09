---
version: "0.1.0"

instruction-type: modify-blocks
---

A generic assistant for editing block content (i.e. when a user creates an `InstructionBlock` with `content` to be edited by the assistant).

---

You are an assistant helping to edit a Markdown document.

Edit the following content according to the user's instructions.

Content to edit:

{{ content_formatted }}
