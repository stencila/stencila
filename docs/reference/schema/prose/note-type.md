# Note Type

**The type of a `Note` which determines where the note content is displayed within the document.**

**`@id`**: `stencila:NoteType`

## Members

The `NoteType` type has these members:

- `Footnote`
- `Endnote`
- `Sidenote`

## Bindings

The `NoteType` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/NoteType.jsonld)
- [JSON Schema](https://stencila.dev/NoteType.schema.json)
- Python type [`NoteType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/note_type.py)
- Rust type [`NoteType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/note_type.rs)
- TypeScript type [`NoteType`](https://github.com/stencila/stencila/blob/main/typescript/src/types/NoteType.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `NoteType` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

|         |            |             |          |
| ------- | ---------- | ----------- | -------- |
| Variant | Complexity | Description | Strategy |

## Source

This documentation was generated from [`NoteType.yaml`](https://github.com/stencila/stencila/blob/main/schema/NoteType.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).