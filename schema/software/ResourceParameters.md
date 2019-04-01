## Description

`ResourceParameters` is a generic class for representing any kind of resource that has the concept of a requested or reserved amount and a limit. For example, reserving an amount of memory but not going over the limit. Both `resourceRequested` and `resourceLimit` are optional and can be omitted as appropriate for a particular use case.

The values are just numbers and the application parsing application can interpret these however it needs. For example, for a memory limit these may be bytes, for a CPU limit they could be the proportion of CPU to use.