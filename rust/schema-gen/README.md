# Stencila `schema-gen`

**Generation of types, schemas and documentation from the Stencila Schema**

## Purpose

This Rust crate generates the following from the YAML files in `../../schema`:

  - [ ] Markdown documentation: [`src/docs.rs`](src/docs.rs)

  - [ ] Python types: [`src/python.rs`](src/python.rs)

  - [x] Rust types: [`src/rust.rs`](src/rust.rs)

  - [ ] TypeScript types: [`src/typescript.rs`](src/typescript.rs)

  - [ ] JSON-LD context: [`src/json_ld.rs`](src/json_ld.rs)

  - [ ] JSON Schema: [`src/json_schema.rs`](src/json_schema.rs)

## Adding a new type of generated output

- Add a new module with a `impl Schemas` which adds the necessary methods to generate whatever you want to generate (see existing modules as examples)

- Add the new module to `src/lib.rs`

- In `src/main.rs` add:

  - a new variant to `What` to enable the new output

  - a new case for the `match` in `async fn main()` to handle the new varant

- Test by generating the new output. By default all outputs are generated. To generated only some, list them e.g.

```sh
cargo run -p schema-gen -- docs rust
```

## CI

As part of the GitHub `build.yml` workflow, all generations will be run on each push and any changes to generated files committed.
