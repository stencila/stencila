---
title: Agent
description: An AI agent definition.
---

This is a type used in Stencila Schema for reusable AI agent definitions.

It exists to represent an agent as a document-like artifact that combines
metadata, model and provider preferences, tool and skill permissions, and an
optional Markdown body of system instructions. Agents are discovered from
workspace `.stencila/agents/` and user-level `~/.config/stencila/agents/`,
allowing shared project agents and personal agent configurations to coexist.

Key properties include `name`, `models`, `providers`, `allowedSkills`,
`allowedTools`, and `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Agent`:

- [OpenAI GPT configuration](https://platform.openai.com/): Approximate analogue for a reusable agent configuration combining instructions, model preferences, and tool policies.
- [Anthropic Claude system prompt configuration](https://docs.anthropic.com/): Approximate provider-level analogue for reusable assistant behavior definitions.

# Properties

The `Agent` type has these properties:

| Name                       | Description                                                                                                             | Type                                                                              | Inherited from                       |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------ |
| `name`                     | The name of the agent.                                                                                                  | [`String`](./string.md)                                                           | -                                    |
| `whenToUse`                | Positive selection signals describing when this agent should be used.                                                   | [`String`](./string.md)*                                                          | -                                    |
| `whenNotToUse`             | Negative selection signals describing when this agent should not be used.                                               | [`String`](./string.md)*                                                          | -                                    |
| `frontmatter`              | Frontmatter containing agent metadata.                                                                                  | [`String`](./string.md)                                                           | -                                    |
| `content`                  | The content of the agent (the Markdown body providing system instructions).                                             | [`Block`](./block.md)*                                                            | -                                    |
| `models`                   | Model identifiers for the agent.                                                                                        | [`String`](./string.md)*                                                          | -                                    |
| `providers`                | Provider identifiers for the agent.                                                                                     | [`String`](./string.md)*                                                          | -                                    |
| `modelSize`                | Model size preference for the agent.                                                                                    | [`String`](./string.md)                                                           | -                                    |
| `reasoningEffort`          | Reasoning effort level for the agent.                                                                                   | [`String`](./string.md)                                                           | -                                    |
| `historyThinkingReplay`    | Whether to replay assistant thinking and reasoning in conversation history.                                             | [`String`](./string.md)                                                           | -                                    |
| `truncationPreset`         | Named preset for tool output truncation limits.                                                                         | [`String`](./string.md)                                                           | -                                    |
| `compactionTriggerPercent` | Context usage percentage that triggers proactive history compaction.                                                    | [`UnsignedInteger`](./unsigned-integer.md)                                        | -                                    |
| `trustLevel`               | Trust level controlling how strictly the agent's operations are guarded.                                                | [`String`](./string.md)                                                           | -                                    |
| `allowedSkills`            | Skill names this agent can use.                                                                                         | [`String`](./string.md)*                                                          | -                                    |
| `allowedTools`             | Tool names available to the agent.                                                                                      | [`String`](./string.md)*                                                          | -                                    |
| `allowedDomains`           | Domain allowlist for web_fetch.                                                                                         | [`String`](./string.md)*                                                          | -                                    |
| `disallowedDomains`        | Domain denylist for web_fetch.                                                                                          | [`String`](./string.md)*                                                          | -                                    |
| `enableMcp`                | Whether to enable MCP tools.                                                                                            | [`Boolean`](./boolean.md)                                                         | -                                    |
| `enableMcpCodemode`        | Whether to enable MCP codemode orchestration.                                                                           | [`Boolean`](./boolean.md)                                                         | -                                    |
| `allowedMcpServers`        | MCP server IDs this agent is allowed to use.                                                                            | [`String`](./string.md)*                                                          | -                                    |
| `maxTurns`                 | Maximum conversation turns (0 = unlimited).                                                                             | [`Integer`](./integer.md)                                                         | -                                    |
| `toolTimeout`              | Default timeout for tool execution in seconds.                                                                          | [`Integer`](./integer.md)                                                         | -                                    |
| `maxToolRounds`            | Maximum tool-call rounds per user input.                                                                                | [`Integer`](./integer.md)                                                         | -                                    |
| `maxSubagentDepth`         | Maximum subagent nesting depth.                                                                                         | [`Integer`](./integer.md)                                                         | -                                    |
| `compatibility`            | Environment requirements for the agent.                                                                                 | [`String`](./string.md)                                                           | -                                    |
| `workType`                 | The type of `CreativeWork` (e.g. article, book, software application).                                                  | [`CreativeWorkType`](./creative-work-type.md)                                     | [`CreativeWork`](./creative-work.md) |
| `doi`                      | The work's Digital Object Identifier (https://doi.org/).                                                                | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `about`                    | The subject matter of the content.                                                                                      | [`ThingVariant`](./thing-variant.md)*                                             | [`CreativeWork`](./creative-work.md) |
| `abstract`                 | A short description that summarizes a `CreativeWork`.                                                                   | [`Block`](./block.md)*                                                            | [`CreativeWork`](./creative-work.md) |
| `authors`                  | The authors of the `CreativeWork`.                                                                                      | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `provenance`               | A summary of the provenance of the content within the work.                                                             | [`ProvenanceCount`](./provenance-count.md)*                                       | [`CreativeWork`](./creative-work.md) |
| `contributors`             | A secondary contributor to the `CreativeWork`.                                                                          | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `editors`                  | People who edited the `CreativeWork`.                                                                                   | [`Person`](./person.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `maintainers`              | The maintainers of the `CreativeWork`.                                                                                  | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `comments`                 | Comments about this creative work.                                                                                      | [`Comment`](./comment.md)*                                                        | [`CreativeWork`](./creative-work.md) |
| `dateCreated`              | Date/time of creation.                                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateReceived`             | Date/time that work was received.                                                                                       | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateAccepted`             | Date/time of acceptance.                                                                                                | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateModified`             | Date/time of most recent modification.                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `datePublished`            | Date of first publication.                                                                                              | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `funders`                  | People or organizations that funded the `CreativeWork`.                                                                 | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `fundedBy`                 | Grants that funded the `CreativeWork`; reverse of `fundedItems`.                                                        | ([`Grant`](./grant.md) \| [`MonetaryGrant`](./monetary-grant.md))*                | [`CreativeWork`](./creative-work.md) |
| `genre`                    | Genre of the creative work, broadcast channel or group.                                                                 | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `keywords`                 | Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.  | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `isPartOf`                 | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [`CreativeWorkVariant`](./creative-work-variant.md)                               | [`CreativeWork`](./creative-work.md) |
| `licenses`                 | License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.         | ([`CreativeWorkVariant`](./creative-work-variant.md) \| [`String`](./string.md))* | [`CreativeWork`](./creative-work.md) |
| `parts`                    | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [`CreativeWorkVariant`](./creative-work-variant.md)*                              | [`CreativeWork`](./creative-work.md) |
| `publisher`                | A publisher of the CreativeWork.                                                                                        | [`Person`](./person.md) \| [`Organization`](./organization.md)                    | [`CreativeWork`](./creative-work.md) |
| `bibliography`             | A bibliography of references which may be cited in the work.                                                            | [`Bibliography`](./bibliography.md)                                               | [`CreativeWork`](./creative-work.md) |
| `references`               | References to other creative works, such as another publication, web page, scholarly article, etc.                      | [`Reference`](./reference.md)*                                                    | [`CreativeWork`](./creative-work.md) |
| `text`                     | The textual content of this creative work.                                                                              | [`Text`](./text.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `title`                    | The title of the creative work.                                                                                         | [`Inline`](./inline.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `repository`               | URL of the repository where the un-compiled, human readable source of the work is located.                              | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `path`                     | The file system path of the source of the work.                                                                         | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `commit`                   | The commit hash (or similar) of the source of the work.                                                                 | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `version`                  | The version of the creative work.                                                                                       | [`String`](./string.md) \| [`Number`](./number.md)                                | [`CreativeWork`](./creative-work.md) |
| `alternateNames`           | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)                |
| `description`              | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `identifiers`              | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)                |
| `images`                   | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)                |
| `url`                      | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `id`                       | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md)              |

# Related

The `Agent` type is related to these types:

- Parents: [`CreativeWork`](./creative-work.md)
- Children: none

# Bindings

The `Agent` type is represented in:

- [JSON-LD](https://stencila.org/Agent.jsonld)
- [JSON Schema](https://stencila.org/Agent.schema.json)
- Python class [`Agent`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Agent`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/agent.rs)
- TypeScript class [`Agent`](https://github.com/stencila/stencila/blob/main/ts/src/types/Agent.ts)

***

This documentation was generated from [`Agent.yaml`](https://github.com/stencila/stencila/blob/main/schema/Agent.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
