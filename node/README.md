# `node` : Stencila for [Node.js](https://nodejs.org)

This package is in preliminary development. An overview:

- the YAML component class definitions in the `../meta` module are used to generate wrapper classes and documentation.

- [`wrap.js`](wrap.js) generates `build/{class}.hpp` (wrapper class) and `build/{class}.json` (JsDoc documentation) using the [Nunjunks](https://mozilla.github.io/nunjucks/) templates [`class.hxx`](class.hxx) and [`class.jsx`](class.jsx) respectively.

- the `{class}-extras.hpp` files are `#include`d into the generated classes to allow for additional and/or Node specific method implementations

- we use `nan` [Native Abstractions for Node.js](https://github.com/nodejs/nan) for "making add-on development for Node.js easier across versions" 

- we use `node-gyp` for building the C++ extension and `node-pre-gyp` for "easy binary deployment of C++ addons"

- [`context.hpp`](context.hpp) implements the C++ interface to the Javascript execution context implemented in [`context.js`](context.js)

- there's some [tests](tests) and a [Makefile](Makefile) with a few common recipes, or just use `npm test` etc if that's more your thing
