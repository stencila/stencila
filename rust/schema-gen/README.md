# Stencila `schema-gen`

**Generation of types, schemas and documentation from the Stencila Schema**

## Purpose

This Rust crate generates the following from the YAML files in `../../schema`:

  - [x] Markdown documentation: [`src/docs_types.rs`](src/docs_types.rs) and [`src/docs_codecs.rs`](src/docs_codecs.rs)

  - [x] Python types: [`src/python.rs`](src/python.rs)

  - [x] Rust types: [`src/rust.rs`](src/rust.rs)

  - [x] TypeScript types: [`src/typescript.rs`](src/typescript.rs)

  - [x] JSON-LD context: [`src/json_ld.rs`](src/json_ld.rs)

  - [x] JSON Schema: [`src/json_schema.rs`](src/json_schema.rs)

## Development

To add a new type of generated output:

- Add a new module with a `impl Schemas` which adds the necessary methods to generate whatever you want to generate (see existing modules as examples)

- Add the new module to `src/lib.rs`

- In `src/main.rs` add:

  - a new variant to `enum What` to enable the new output

  - a new case for the `match` in `async fn main()` to handle the new variant

- Test by generating the new output. By default all outputs are generated. To generated only some, list them e.g.

```sh
cargo run -p schema-gen -- docs rust
```

Because this crate depends upon the Rust types it generates (for generating documentation), if there are any errors in that Rust code, it is not possible to recompile. To avoid this circular dependency, run generation without the `docs` feature enabled:

```sh
cargo run -p schema-gen --no-default-features
```

## Continuous deployment

As part of the GitHub `build.yml` workflow, all generations will be run on each push and any changes to generated files committed.
