## `stencila/js` : Stencila for Javascript

[![NPM](http://img.shields.io/npm/v/stencila-js.svg?style=flat)](https://www.npmjs.com/package/stencila-js)
[![Build status](https://travis-ci.org/stencila/js.svg?branch=master)](https://travis-ci.org/stencila/js)
[![Code coverage](https://codecov.io/gh/stencila/js/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/js)
[![Dependency status](https://david-dm.org/stencila/js.svg)](https://david-dm.org/stencila/js)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

This package contains code that is shared amongst other Stencila Javascript-based packages: [`node`](https://github.com/stencila/node) (the package for Node.js) and [`web`](https://github.com/stencila/node) (the package for web browsers):

- a `JsSession` class for executing code in Javascript
- data `pack` and `unpack` functions for transferring data over the wire and between languages

See [this blog post](http://blog.stenci.la/chunks-n-funcs/) for more on the approach and how it's used within Stencila Documents.

### Install

```
npm install stencila-js --save
```

### Use

```js
// Create a session
let session = new JsSession()

// Evaluate an expression...
session.execute('6*7') // { errors: {}, output: { type: 'int', format: 'text', value: '42' } }

// Output is the value of the last line,
session.execute('let x = 6\nx*7') // { errors: {}, output: { type: 'int', format: 'text', value: '42' } }

// If the last line is blank there is no output (this is intended for code chunks that have side effects e.g. set up data),
session.execute('let x = 6\nx*7\n\n') // { errors: {}, output: null }

// You can specify input variables (that are local to that call) as a data pack,
session.execute('Math.PI*radius', {radius:pack(21.4)}) // { errors: {}, output: { type: 'flt', format: 'text', value: '67.23008278682157' } }
session.execute('radius') // { errors: { '1': 'ReferenceError: radius is not defined' }, output: null }

// You can also assign global variables which are available in subsequent calls,
session.execute('globals.foo = "bar"\n\n') # { errors: {}, output: null }
session.execute('foo') # { errors: {}, output: { type: 'str', format: 'text', value: 'bar' } }
```

More documentation is available at https://stencila.github.io/js.


### Discuss

We love feedback. Create a [new issue](https://github.com/stencila/js/issues/new), add to [existing issues](https://github.com/stencila/js/issues) or [chat](https://gitter.im/stencila/stencila) with members of the community.


### Develop

Want to help out with development? Great, there's a lot to do! To get started, read our contributor [code of conduct](CONDUCT.md), then [get in touch](https://gitter.im/stencila/stencila) or checkout the [platform-wide, cross-repository kanban board](https://github.com/orgs/stencila/projects/1).

Most development tasks can be run directly from `npm` or via `make` wrapper recipes.

Task                                                    |`npm`                  | `make`          |
------------------------------------------------------- |-----------------------|-----------------|    
Install and setup dependencies                          | `npm install`         | `make setup`
Check code for lint                                     | `npm run lint`        | `make lint`
Run tests                                               | `npm test`            | `make test`
Run tests with coverage                                 | `npm run cover`       | `make cover`
Build documentation                                     | `npm run docs`        | `make docs`
Serve and watch docs for updates                        | `npm run docs-serve`  | `make docs-serve`
Clean                                                   |                       | `make clean`

Tests live in the `tests` folder and are written using the [`tape`](https://github.com/substack/tape) test harness. And, in another breathtaking display of naming logic, documentation lives in the `docs` folder. Docs are published using Github Pages, so to update them after making changes run `make docs`, commit the updated docs and do a `git push`.
