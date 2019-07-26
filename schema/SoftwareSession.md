---
title: SoftwareSession
authors: []
---

include: ../built/SoftwareSession.schema.md
:::
Represents a runtime session with the resources and image that is required by software to execute.

| Entity          | type           | The name of the type and all descendant types.            | string |
| --------------- | -------------- | --------------------------------------------------------- | ------ |
| Entity          | id             | The identifier for this item.                             | string |
| Thing           | alternateNames | Alternate names (aliases) for the item.                   | array  |
| Thing           | description    | A description of the item.                                | string |
| Thing           | meta           | Metadata associated with this item.                       | object |
| Thing           | name           | The name of the item.                                     | string |
| Thing           | url            | The URL of the item.                                      | string |
| SoftwareSession | volumeMounts   | Volumes to mount in the session.                          | array  |
| SoftwareSession | cpuResource    |                                                           |        |
| SoftwareSession | memoryResource |                                                           |        |
| SoftwareSession | environment    | Definition of the environment to execute this session in. |        |

:::

# Related

## Open Containers Initiative

This is "inspired" by the [OCI Runtime Config Schema](https://github.com/opencontainers/runtime-spec/blob/master/schema/config-schema.json)

There are a number of properties that did not seem relevant to our use as well as some references that did not seem to be required. We try to stay consistent with the naming used but have dereferenced and combined the config schema and [Linux Schema](https://github.com/opencontainers/runtime-spec/blob/master/schema/defs-linux.json) to use the cpu and memory limits.
