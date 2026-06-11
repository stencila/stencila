---
title: Action Agent
description: A human, organization, software application, or Stencila AI agent that performs, provides, or participates in an action.
---

This is a union type used by [`Action`](./action.md) properties that identify
agents. It follows schema.org's `Person` and `Organization` range for
`Action.agent`, while adding `SoftwareApplication` for software tools and
Stencila `Agent` for reusable AI agent definitions. This distinction allows
provenance to separate an application such as Stencila CLI or Python from a
configured AI agent that may have instructions, model preferences, and tool
permissions.


# Analogues

The following external types, elements, or nodes are similar to a `ActionAgent`:

- schema.org [`agent`](https://schema.org/agent): Close analogue for the expected range of schema.org `Action.agent`, extended with Stencila software and AI agent types.

# Members

The `ActionAgent` type has these members:

- [`Person`](./person.md)
- [`Organization`](./organization.md)
- [`SoftwareApplication`](./software-application.md)
- [`Agent`](./agent.md)

# Bindings

The `ActionAgent` type is represented in:

- [JSON-LD](https://stencila.org/ActionAgent.jsonld)
- [JSON Schema](https://stencila.org/ActionAgent.schema.json)
- Python type [`ActionAgent`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ActionAgent`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/action_agent.rs)
- TypeScript type [`ActionAgent`](https://github.com/stencila/stencila/blob/main/ts/src/types/ActionAgent.ts)

***

This documentation was generated from [`ActionAgent.yaml`](https://github.com/stencila/stencila/blob/main/schema/ActionAgent.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
