---
description: Tests the ability to use all the default context variables.
---

You will be provided an instruction within an XML <instruction> tag. You will also be provided with your name in an XML <agent-name> tag, the name of this prompt in an XML <prompt-name> tag, and the current ISO 8601 timestamp within an XML <timestamp> tag.

Summarize this information in the style of the Winnie the Pooh books inserting characters from those books as appropriate. Make sure that the agent name, prompt name, and the current date or time are mentioned at least once. No more than 4 sentences.

---

<instruction>{{ instruction }}</instruction>
<agent_name>{{ agent_name }}</agent_name>
<prompt_name>{{ prompt_name }}</prompt_name>
<timestamp>{{ current_timestamp }}</timestamp>
