# Models

**Benchmarking and evaluation of large language models**

## 🤖 Introduction

Stencila has a library of [assistants](../assistants), each of which is specialized for a specific task (e.g. editing code, writing an abstract). These assistants generate a prompt which includes contextual information from your document and execution environment, as well as your instruction, and delegate that task to a large language model (LLM) to generate content.

Different LLMs perform differently depending upon the task. Different LLMs also have different speed and cost characteristics. This module provides benchmarking and evaluation so that Stencila assistants can delegate to the best LLM given the task and the relative weight you place on quality, speed and cost.

> [!NOTE]
> This module in in early development. More to come soon!
