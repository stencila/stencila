# Stencila `schema-gen`

**Generation of types, schemas and documentation from the Stencila Schema**

This Rust crate generates the following from the YAML files in `../../schema`:

- [ ] Markdown documentation: [`src/docs.rs`](src/docs.rs)

- [ ] Python types: [`src/python.rs`](src/python.rs)

- [x] Rust types: [`src/rust.rs`](src/rust.rs)

- [ ] TypeScript types: [`src/typescript.rs`](src/typescript.rs)

- [ ] JSON-LD context: [`src/json_ld.rs`](src/json_ld.rs)

- [ ] JSON Schema: [`src/json_schema.rs`](src/json_schema.rs)

To add another type of generation:

- add a new module with a `impl Schemas` which adds the necessary methods to generate whatever you want to generate (see existing modules as examples)

- add the new module to `src/lib.rs`

- in `src/main.rs` add:

    - an option to `Args` to be able to turn off the new generation

    - an `if` block in `async fn main()` to enable the new generation

As part of the GitHub `build.yml` workflow, all generations will be run on each push and any changes to generated files committed.
