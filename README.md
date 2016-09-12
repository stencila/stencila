## `stencila/web`

[![Build status](https://travis-ci.org/stencila/web.svg?branch=master)](https://travis-ci.org/stencila/web)
[![Code coverage](https://codecov.io/gh/stencila/web/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/web)
[![Dependency status](https://david-dm.org/stencila/web.svg)](https://david-dm.org/stencila/web)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

### Development

Most development tasks can be run directly using Javascript tooling (`npm` etc) or via `make` wrapper recipes.

Task                                                    |`npm` et al            | `make`          |
------------------------------------------------------- |-----------------------|-----------------|    
Install and setup dependencies                          | `npm install`         | `make setup`
Run the development server                              | `npm start`           | `make serve`
Check code for lint                                     | `npm run lint`        | `make lint`
Run all tests                                           | `npm test`            | `make test`
Run unit tests only                                     | `npm run test-unit`   | `make test-unit`
Run functional tests only                               | `npm run test-fun`    | `make test-fun`
Run tests with coverage                                 | `npm run cover`       | `make cover`
Build                                                   | `gulp build`          | `make build`
Clean                                                   | `rm -rf build`        | `make clean`

#### Testing

Unit tests (`*.test.js`) and functional tests (`*.fun.js`) live in the `tests` folder and are written using the [`tape`](https://github.com/substack/tape) test harness. The functional tests use [`nightmare`](https://github.com/segmentio/nightmare) to simulate user flows in the browser. They run more slowly, so you might not want to run them as often as the unit tests.

