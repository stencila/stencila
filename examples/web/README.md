# Examples of Stencila documents for end-to-end tests of Web Components

These examples are used for tests of the Web Components developed in the `web` module of this repository.

## Organization

Each subfolder (e.g. `paragraph`) tests the Web Component for the corresponding document node type (e.g. the `<stencila-paragraph>` component in `web/src/nodes/paragraph.ts`).

Within each subfolder there are example documents, generally each having only one example node, but with varying states. For example, `paragraph/simple.smd` contains a single paragraph with no properties other than `content` and `paragraph/authors.smd` has a single paragraph with both `content` and `authors`.

## Adding examples

Please keep examples small and focussed on a specific variation of a node type. 

If appropriate, include a YAML header in with a `description` of the example (we use the header for this so it can be changed without changing the screenshot of the example). 

See existing examples for conventions used for both content and file naming. For most node types there will be:

- a `simple.smd` file which has a single example with only the properties that can be represented using Markdown.

- other `.smd` files, with sidecar JSON files, for examples with other properties that can not be represented using Markdown e.g `authors`, `provenance`, execution details.

The recommended way to add examples with a JSON sidecar file is to use the `new` command and then open, edit, run, and save the document from within VSCode with the Stencila extension enabled:

```sh
# Using a development version of the CLI
cargo run -p cli new paragraph/simple.smd --sidecar json

# Using an installed version of the CLI
stencila new paragraph/simple.smd --sidecar json
```

## Running tests

You can run the end-to-end tests on these examples,

```sh
# From the root of the repo using Make
make -C web e2e

# From the root of this repo using NPM
npm -w web run e2e

# From within the `web` module
npm run e2e
```

> ![WARN]
>
> The `playright` test harness starts the Stencila CLI `serve` command
> in this folder, and then does a health check of the server. If there
> is no `main.*`, `index.*`, or `README.*` in this folder then the server
> will return a 404 and `playright` will keep waiting.
> So, remove this file with care.
