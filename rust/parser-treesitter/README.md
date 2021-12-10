# `parser-treesitter`

This crate contains utility functions and `struct`s for developing Stencila programming language parsers that are based upon [Tree-sitter](https://tree-sitter.github.io/tree-sitter/).

See these sibling crates as examples:

- [`parser-bash`](../parser-bash)
- [`parser-js`](../parser-js)
- [`parser-py`](../parser-py)
- [`parser-r`](../parser-r)
- [`parser-ts`](../parser-ts)

When developing language queries the `tree-sitter` CLI is very useful:

1. Install and setup `tree-sitter` including running `tree-sitter init-config` (this is a [good guide](https://dcreager.net/tree-sitter/getting-started/) to that.)

2. Clone the repo for the language grammar e.g.

   ```sh
   mkdir -p ~/src
   cd ~/src
   git clone https://github.com/r-lib/tree-sitter-r
   ```

3. Parse fixture files to glean the structure of the AST for the language e.g.

   ```sh
   tree-sitter parse ../../fixtures/fragments/r/imports.R
   ```

4. Create a query (you can `include_str!` or copy-paste it into Rust code later) and test it against the query files e.g.

   ```sh
   tree-sitter query src/query.txt ../../fixtures/fragments/r/imports.R
   ```
