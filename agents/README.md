# Stencila Agents

**AI agents specialized for scientific research, coding and writing**

## 🤖 Introduction

Custom prompts are an effective way to improve the performance of large language models and other generative AI on specific tasks in specific contexts. This module contains custom prompts which are, or can be, used by Stencila when creating tasks for AI agents.

There are two types of agents in this module:

- [`builtin`](builtin): agents that are embedded into the `stencila` CLI binary to be used by builtin agents

- [`contrib`](contrib): contributed agents that are not builtin but which can be fetched from this repo (🦄 this functionality does not yet exist!)

## ✏️ Format

Agents are specified in Markdown files with a YAML header, the system prompt, a thematic break (three dashes i.e. `---`), and the user prompt. i.e.

```markdown
---
name: example/agent
description: An example agent

delegates:
  - openai/gpt-3.5-turbo-1106
  - anthropic/claude-2.1
---

The system prompt

---

The user prompt
```

### Header

The following fields are required in the header:

- `name`: a unique name for the agent
- `description`: a description of what the agent does and how it does it
- `delegates`: a list of general agents that will be delegated to; delegation will be attempted in the order specified

### Prompts

The user prompts are Jinja templates so you can use [this syntax](https://docs.rs/minijinja/latest/minijinja/syntax/index.html) to alter the prompt based on the context of the instruction. e.g.

```markdown
You will be provided with several fragments of text, each within an XML <fragment> tag. Summarize the fragments as accurately as possible in the style provided in the XML <style> tag. Use no more than 4 sentences.

---

<style>{{ instruction_text }}</style>

{% for fragment in fragments %}
<fragment>{{ fragment }}</fragment>
{% endfor %}
```

## 🛠️ Development

There are some tools in Stencila for helping with [prompt engineering](https://en.wikipedia.org/wiki/Prompt_engineering): improving the performance of prompts used by agents for a specific task.

### Listing

The `stencila agents` command (⚠️ this is planned to be renamed to `stencila ai agents` soon) provides a list of the available agents. If the relevant API key is not set in an environment variable, the corresponding agents will not appear in this list.

### Testing

The `stencila test` command (⚠️ will be renamed to `stencila ai test`) can be used to run agents on the test instructions in the [`tests`](tests) folder.

Each test folder has a `document.md`, an example document written in Markdown, and one or more instructions written in YAML files. You can run each test instruction individually, e.g.

```console
cd agents/tests
cargo run -p cli test doctor-who create-summary --reps 3
```

The `test` command will create a Markdown file with the same name as the instruction (in this case `create-summary.md`) with a header detailing the task created from the instruction and the document and the content generated for each repetition.

### REPL

The `stencila repl` command (⚠️ will be renamed to `stencila ai repl`) provides a read-evaluate-print-loop for engineering agents. When the CLI is compiled in debug mode, prompts will be reloaded from disk each time they are used.

This means that you can alter the prompt during a REPL session and check how it affects performance. The REPL has up and down arrow history support so you can easily repeat the same instructions after modifying the prompt.

#### Options

Options such as the agent can be set at the start of a session e.g.

```console
cargo run -p cli repl --agent stencila/insert-blocks
```

or during the session:

```
>> --agent stencila/modify-inlines
Options were updated
>> ?options
{"agent":"stencila/modify-inlines"}
```

For a full list of options use `--help`. You can set any of the options this way. For example, setting the temperature of the model:

```
>> --temperature 0.2
Options were updated
>> ?options
{"agent":"stencila/modify-inlines","temperature":0.2}
```

#### Recording

At session start up, you can specify the `--record` flag to make the REPL ask you whether you want to store the trial (the agent, prompt, instruction, response, options used etc) in a local SQLite database:

```sh
$ touch testing.sqlite3 # In the future this should not be necessary
$ cargo run -p cli repl --record
```

```
>> create a 3x5 table with animal names as column headers
custom/insert-block

| Animal 1 | Animal 2 | Animal 3 | Animal 4 | Animal 5 |
|----------|----------|----------|----------|----------|
|          |          |          |          |          |
|          |          |          |          |          |
|          |          |          |          |          |
>> Would you like to record this trial? (y/n)
>> y
>> create a 3x5 table with specific animal names as column headers
custom/insert-block

| Lion | Tiger | Elephant | Giraffe | Zebra |
|------|-------|----------|---------|-------|
|      |       |          |         |       |
|      |       |          |         |       |
|      |       |          |         |       |
>> Would you like to record this trial? (y/n)
>>
```
