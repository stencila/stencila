# Stencila Agents

**AI agents specialized for scientific research and writing**

## 🤖 Introduction

Custom prompts are an effective way to improve the performance of large language models and other generative AI on specific tasks in specific contexts. This module contains custom prompts which are, or can be, used by Stencila when creating tasks for AI agents.

There are three types of agents in this module:

- [`builtin`](builtin): agents that are embedded into the `stencila` CLI binary to be used by builtin agents

- [`contrib`](contrib): contributed agents that are not builtin but which can be fetched from this repo (this does not yet exist)

- [`example`](example): example agents which illustrate alternative approaches to prompt engineering and are useful for testing when developing Stencila
 

## ✏️ Format

Agents are specified in Markdown files with a YAML header, the system prompt, a thematic break (three dashes i.e. `---`), and the user prompt:

```markdown
---
name: example/agent
extends: openai/gpt-3.5-turbo-0613
description: A description of the agent.
---

The system prompt

---

The user prompt
```

Currently we just use the YAML header for a description but in the future more meta-data may be added there.

The system and user prompts are Jinja templates so you can use [this syntax](https://docs.rs/minijinja/latest/minijinja/syntax/index.html) to alter the prompts based on the context passed to it. e.g.

```markdown
---
description: Summarizes fragments in a specific style.
---

You will be provided with several fragments of text, each within an XML <fragment> tag. Summarize the fragments as accurately as possible in the style provided in the XML <style> tag. Use no more than 4 sentences.

---

<style>{{ user_instruction }}</style>

{% for fragment in fragments %}
<fragment>{{ fragment }}</fragment>
{% endfor %}
```

Currently, the following are added to each prompt rendering context:

- `agent_name`: The name of the agent
- `provider_name`: The name of the model provider e.g. `openai`
- `model_name`: The name of the model e.g. `gpt3.5-turbo`
- `prompt_name`: The name of the prompt
- `current_timestamp`: The current time formatted as a ISO RFC3339 (8601) timestamp
- `user_instruction`: The instruction provided by the user

## 🛠️ Development

There are some tools in Stencila for helping with [prompt engineering](https://en.wikipedia.org/wiki/Prompt_engineering): improving the performance of prompts used by agents for a specific task.

### CLI

The `stencila repl` command (⚠️ this is planned to be renamed to `stencila ai repl` soon) provides a workbench for engineering the prompts used in Stencila. When the CLI is compiled in debug mode, prompts will be reloaded from disk each time they are used. This means that you can alter the prompt during a REPL session and check how it affects performance. The REPL has up and down arrow history support so you can easily repeat the same instructions after modifying the prompt.

#### Options

Options such as the prompt and agent can be set at the start of a session e.g.

```console
cargo run --bin stencila repl --prompt <NAME>
```

or during the session:

```
>> --prompt winnie
Options were updated
>> ?options
{"prompt_name":"winnie"}
```

For a full list of options use `--help`. You can set any of the options this way. For example, setting the temperature of the model:

```
>> --temperature 0.2
Options were updated
>> ?options
{"prompt_name":"winnie","temperature":0.2}
```

#### Agent

By default the best agent for the given instruction is used (currently we have a simple ordering, but that will be improved). If you want to use a specific agent use the `--agent` option.

```
>> ?agent
No specific agent chosen; use `--agent` to specify one
>> --agent openai/gpt-3.5-turbo-1106
>> ?agent
openai/gpt-3.5-turbo-1106
```

#### Recording trials

At session start up, you can specify the `--record` flag to make the REPL ask you whether you want to store the trial (the agent, prompt, instruction, response, options used etc) in a local SQLite database:

```sh
$ touch testing.sqlite3 # In the future this should not be necessary
$ cargo run --bin stencila repl --record
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

### Rust

Within Stencila's Rust code several types of agents are implemented. Each type of agent has a default prompt (which is used unless the user specifies one). To declare which prompt an agent should use override the `Agent.prompt` method with the name of a builtin prompt e.g.

```rust
impl Agent for SpecialAgent {
    fn prompt(&self) -> &str {
        "special"
    }

    ...
}
```
