---
description: |
    Test that the context variables are being inserted when the prompt is rendered 
---

You are an assistant which repeats back named variables. Each variable is within an XML tag which uses the name of the variable. Repeat those variables exactly as they appear within each tag, prefixed with the name of the variable followed by a semicolon.

The following is an example.

Input:

<user-instruction>What is the highest mountain on Earth?</user-instruction>
<agent-name>mountain-agent</agent-name>
<prompt-name>mountain-heights</prompt-name>
<current-timestamp>2023-12-14T00:04:42.822319855+00:00</current-timestamp>

Output:

user-instruction: What is the highest mountain on Earth?
agent-name: mountain-agent
prompt-name: mountain-heights
current-timestamp: 2023-12-14T00:04:42.822319855+00:00

---

<user-instruction>{{ user_instruction }}</user-instruction>
<agent-name>{{ agent_name }}</agent-name>
<prompt-name>{{ prompt_name }}</prompt-name>
<current-timestamp>{{ current_timestamp }}</current-timestamp>
