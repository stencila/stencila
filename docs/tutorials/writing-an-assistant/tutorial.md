# Building a custom assistant

## Background

This tutorial walks you through creating a custom assistant.
"Assistants" are Stencila’s term for agents that collaborate with you when writing a document.
The assistants in Stencila are powered by Large Language Models (LLMs), such as OpenAI’s GPT.
Stencila can use many of the most popular models including ones you can run locally if privacy or cost is a concern.
You will need to have established access to them (add a link here to how to do this).

In its simplest form, an assistant just embeds access to these various models in the document, recording any conversation you might have.
This is already an improvement over cutting and pasting text from another app, as stencila records the origin of any text that is changed or include, and provides an attribution.

Stencila’s special assistants take this further, however.
A custom assistant allows you to tailor the request, giving it specific instructions, and also including any appropriate context from the current document or elsewhere.
Stencila can also post-process the results, ensuring the reply is in the expected format, and even retry when it is not.

Stencila already includes some special assistants that can insert figures and tables or code. But you can write your own assistants very easily, as they are simply Markdown files with some constraints on format.

This tutorial walks you through a few simple examples.

## Getting Started

Stencila looks for any custom written assistants in a default folder. You can see what this folder is by running:

```bash
stencila cli config --dir assistants
```

The location will vary, depending on what platform you are on (Mac, Unix, or Windows).

You might prefer to place your assistants locally, or in a git repo, however.
To do that, you override the default folder by defining the environment variable `STENCILA_ASSISTANTS_DIR`.
You can check this has worked by running the command above again.

```bash
export STENCILA_ASSISTANTS_DIR=`pwd`/assistants
stencila cli config --dir assistants
# Should show the path to the assistants folder in the current directory
```

## Writing the Assistant

An assistant is written as a Markdown file with a YAML header, and two sections, divided by a thematic break (`---`).
The YAML header defines the name of the assistant, a version number, and some options (we'll come back to these).
The first section is a description of the assistant, and the second section is the instructions for the assistant.
The bulk of the work is done in the instructions section.
This part is included as a "prompt" that is sent to the LLM.
