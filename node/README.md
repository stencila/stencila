# `node` : Stencila for [Node.js](https://nodejs.org)

## Installing

This module is not yet published to the npm registry but you can install it direct from the Github repo:

```
npm install stencila/stencila
```

Note that npm does not supporting install sub-directories of a git repo, so when you do an install this way you'll also install the Stencila `web` module.


## Using

This is still in very initial development, but here's a quick snippet of what's currently working...

```js
> var stencila = require('stencila-node')
undefined
> var s = stencila.Stencil('.')
undefined
> // Set Cila content and render it
> s.cila('exec\n\t_scope_.x=6*7;\n\np {text x}')
> s.render()
{}
> // What does the content look like now?
> s.cila();
'exec &b2TLp2r\n\t_scope_.x=6*7;\n\n{text x 42}'
> s.html()
'<pre data-exec="exec" data-hash="b2TLp2r">\n_scope_.x=6*7;\n</pre><p><span data-text="x">42</span></p>'
> // View it in the browser
> s.view()
```

## Developing

We'd love any help progressing this! To get started, here's an overview what's in this module:

- the YAML component class definitions in the `../meta` module are used to generate C++ wrapper classes and documentation.

- [`wrap.js`](wrap.js) generates `build/{class}.hpp` (wrapper class) and `build/{class}.json` (JsDoc documentation) using the [Nunjunks](https://mozilla.github.io/nunjucks/) templates [`class.hxx`](class.hxx) and [`class.jsx`](class.jsx) respectively.

- the `{class}-extras.hpp` files are `#include`d into the generated classes to allow for additional and/or Node specific method implementations

- we use `nan` [Native Abstractions for Node.js](https://github.com/nodejs/nan) for "making add-on development for Node.js easier across versions" 

- we use `node-gyp` for building the C++ extension and `node-pre-gyp` for "easy binary deployment of C++ addons"

- [`context.hpp`](context.hpp) implements the C++ interface to the Javascript execution context implemented in [`context.js`](context.js)

- there's some [tests](tests) and a [Makefile](Makefile) with a few common recipes (e.g. `make build`, `make test`), or just use `npm test` etc if that's more your thing


## Versioning

Our standard version numbering scheme includes a build portion (i.e. `major.minor.patch+build`) as permitted under Semver 2.0.0 (http://semver.org). Unfortunately npm only supports `major.minor.patch`. Also, the version number goes in `package.json` which is committed and used by `node-pre-gyp` when installing the package (to get the right binary extension). Rather than commit a change to `package.json` for every build we are currently just using the `major.minor.patch` portion of the version number and republishing the binary extension for each build.
