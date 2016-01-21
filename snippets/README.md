# Snippet

Snippets are defined in JSON files in the [`snippets`](snippets) subdirectory. This allows them to be developed in parrallel to the consumers of snippets, pricipally the `web` module, but potentially other Stencila modules in the future. It is also a more natural place for people interested in contributing new snippets and fixing existing ones to contribute. Also, it allows them to be fully version controlled.

When this repository is build the snippets contained in here will be submitted to the Stencila Hub so that they can be searched and served in production.

See the [`schema.yml`](schema.yml) file in this directory which defines a schema for the JSON snippets. You can edit that file using the [Swagger Editor](http://editor.swagger.io) (you'll have to paste it back and forth!).

The example [`snippets/1-sum.json`](snippets/1-sum.json) provides examples of alternative snippet code generation methods that will produce the same code.
