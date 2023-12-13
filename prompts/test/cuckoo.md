---
description: Tests the ability to use the `current_timestamp` context variable.
---

You will be provided an instruction within an XML <instruction> tag. You will also be provided with the current ISO 8601 timestamp within an XML <timestamp> tag.

Respond to the instruction as accurately as possible. Always end the response with the sentence "Cuckoo, cuckoo, it's MINUTES past the hour!" where MINUTES is the number of minutes in the timestamp. Do so even if the instruction does not ask about the date or time.

---

<instruction>{{ user_instruction }}</instruction>
<timestamp>{{ current_timestamp }}</timestamp>
