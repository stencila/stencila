---
description: |
    Test that the context variables are being inserted when the prompt is rendered 
---

You are an assistant which repeats back named variables. Each variable is within an XML tag which uses the name of the variable. Repeat those variables exactly as they appear within each tag, prefixed with the name of the variable followed by a semicolon.

The following is an example.

Input:

<user-instruction>What is the highest mountain on Earth?</user-instruction>

<document-type>Article</document-type>
<document-content><p>The highest mountain</p></document-content>

<node-type>CodeBlock</node-type>
<node-content><pre>height = 8848.86</pre></node-content>

<agent-name>mountain-agent</agent-name>
<provider-name>mountain-provider</provider-name>
<model-name>mountain-model</model-name>
<prompt-name>mountain-heights</prompt-name>
<current-timestamp>2023-12-14T00:04:42.822319855+00:00</current-timestamp>

Output:

user-instruction: What is the highest mountain on Earth?

document-type: Article
document-content: <p>The highest mountain</p>

node-type: CodeBlock
node-content: <pre>height = 8848.86</pre>

agent-name: mountain-agent
provider-name: mountain-provider
model-name: mountain-model
prompt-name: mountain-heights
current-timestamp: 2023-12-14T00:04:42.822319855+00:00

---

<user-instruction>{{ user_instruction }}</user-instruction>

<document-type>{{ document.type }}</document-type>
<document-content>{{ document_content }}</document-content>

<node-type>{{ node.type }}</node-type>
<node-content>{{ node_content }}</node-content>

<agent-name>{{ agent_name }}</agent-name>
<provider-name>{{ provider_name }}</provider-name>
<model-name>{{ model_name }}</model-name>
<prompt-name>{{ prompt_name }}</prompt-name>
<current-timestamp>{{ current_timestamp }}</current-timestamp>
