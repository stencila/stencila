# Stencila Prompts

**Prompts for AI agents specialized for scientific research and writing**

## 🤖 Introduction

Custom prompts are an effective way to improve the performance of large language models and other generative AI on specific tasks in specific contexts. This module contains custom prompts which are, or can be, used by Stencila when creating tasks for AI agents.

There are three types of prompts in this module:

- [`builtin`](builtin): prompts that are embedded into the `stencila` CLI binary to be used by builtin agents

- [`contrib`](contrib): contributed prompts that are not builtin but which can be fetched from this repo

- [`test`](test): prompts used during development for testing
 

## ✏️ Format

Prompts are written in Markdown files with a YAML header, the system prompt, a thematic break (three dashes i.e. `---`), and the user prompt:

```markdown
---
description: A description of the prompt.
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

<style>{{ instruction }}</style>

{% for fragment in fragments %}
<fragment>{{ fragment }}</fragment>
{% endfor %}
```

Currently, the following are added to each prompt rendering context:

- `agent_name`: The name of the agent
- `prompt_name`: The name of the prompt
- `current_timestamp`: The current time formatted as a ISO RFC3339 (8601) timestamp
- `instruction`: The instruction provided by the user

## ⚡ Usage

### CLI

The `stencila ai chat` and `stencila ai gen` commands (🦄 both in development and don't actually exist at the moment!) have a `--prompt` option which you can pass the name of the prompt to.

This is mainly useful for development testing. When the CLI is compiled in debug mode, prompts will be reloaded from disk each time they are used. This means that you can alter the prompt during a chat session and check how it alters responses. To run a chat in debug mode with a specific prompt:

```console
cargo run --bin stencila -- ai chat --prompt <NAME>
```

### Rust

Within Stencila's Rust code several types of agents are implemented. Each type of agent has a default prompt (which is used unless the user specifies one). To declare which prompt an agent should use override the `Agent.default_prompt` method with the name of a builtin prompt e.g.

```rust
impl Agent for SpecialAgent {
    fn default_prompt(&self) -> &str {
        "special"
    }

    ...
}
```
