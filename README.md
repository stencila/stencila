## `stencila/js` : Stencila for Javascript

[![Build status](https://travis-ci.org/stencila/js.svg?branch=master)](https://travis-ci.org/stencila/js)
[![Code coverage](https://codecov.io/gh/stencila/js/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/js)
[![Dependency status](https://david-dm.org/stencila/js.svg)](https://david-dm.org/stencila/js)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

This package contains code that is shared amongst other Stencila Javascript-based packages: [`node`](https://github.com/stencila/node) (the package for Node.js) and [`web`](https://github.com/stencila/node) (the package for web browsers):

- data `pack` and `unpack` functions for transferring data between Javascript and other languages
- a `JsSession` class for executing code in Javascript

### Install

```
npm install stencila-js --save
```

### Discover

Documentation is available at https://stencila.github.io/js.


### Discuss

We love feedback. Create a [new issue](https://github.com/stencila/js/issues/new), add to [existing issues](https://github.com/stencila/js/issues) or [chat](https://gitter.im/stencila/stencila) with members of the community.


### Develop

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
