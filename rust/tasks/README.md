## Rust types

This crate includes Rust types for `Taskfile` and associated types (e.g. `Task`, `Precondition`). These types are kept as simple as possible (e.g. using `String` of `enum`) and have a focus on maintaining serialization compatibility with Task and the `Taskfile` schema v3.

We initially investigated automatically generating these types from the [`taskfile.json`](https://json.schemastore.org/taskfile.json) JSON Schema using [`schemafy`](https://docs.rs/schemafy/latest/schemafy/). There were some minor incompatibilities between the structure of that schema and `schemafy` (which could be worked around with some restructuring). However, we decided to manually write Rust types based on https://taskfile.dev/api/#schema because it gives more flexibility (e.g. for serialization).
