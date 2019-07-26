---
title: ResourceParameters
authors: []
---

include: ../built/ResourceParameters.schema.md
:::
Describes limits or requested amounts for a particular resource (e.g. memory or CPU).

| Entity             | type              | The name of the type and all descendant types.                              | string |
| ------------------ | ----------------- | --------------------------------------------------------------------------- | ------ |
| Entity             | id                | The identifier for this item.                                               | string |
| Thing              | alternateNames    | Alternate names (aliases) for the item.                                     | array  |
| Thing              | description       | A description of the item.                                                  | string |
| Thing              | meta              | Metadata associated with this item.                                         | object |
| Thing              | name              | The name of the item.                                                       | string |
| Thing              | url               | The URL of the item.                                                        | string |
| ResourceParameters | resourceLimit     | The maximum amount of the resource that can be used.                        | number |
| ResourceParameters | resourceRequested | The amount of the resource that has been requested (and possibly reserved). | number |

:::

`ResourceParameters` is a generic class for representing any kind of resource that has the concept of a requested or reserved amount and a limit. For example, reserving an amount of memory but not going over the limit. Both `resourceRequested` and `resourceLimit` are optional and can be omitted as appropriate for a particular use case.

The values are just numbers and the application parsing application can interpret these however it needs. For example, for a memory limit these may be bytes, for a CPU limit they could be the proportion of CPU to use.
