## `stencila/web`

[![Build status](https://travis-ci.org/stencila/web.svg?branch=master)](https://travis-ci.org/stencila/web)
[![Code coverage](https://codecov.io/gh/stencila/web/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/web)
[![Dependency status](https://david-dm.org/stencila/web.svg)](https://david-dm.org/stencila/web)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

Web browser user interfaces for Stencila components.

### Development

Most development tasks can be run directly using Javascript tooling (`npm` etc) or via `make` wrapper recipes.

Task                                                    |`npm` et al            | `make`          |
------------------------------------------------------- |-----------------------|-----------------|    
Install and setup dependencies                          | `npm install`         | `make setup`
Run the development server                              | `npm start`           | `make run`
Check code for lint                                     | `npm run lint`        | `make lint`
Run all tests                                           | `npm test`            | `make test`
Run unit tests only                                     | `npm run test-unit`   | `make test-unit`
Run functional tests only                               | `npm run test-fun`    | `make test-fun`
Run tests with coverage                                 | `npm run cover`       | `make cover`
Build                                                   | `gulp build`          | `make build`
Clean                                                   | `rm -rf build`        | `make clean`


#### Developing

After installing dependencies run `npm start` to start the development server and head over to [http://localhost:5000](http://localhost:5000) where there are links to test pages. On these test pages, the Javascript and SCSS are bundled on the fly so that any changes you make are available with a browser refresh.

#### Testing

Unit tests (`*.test.js`) and functional tests (`*.fun.js`) live in the `tests` folder and are written using the [`tape`](https://github.com/substack/tape) test harness. The functional tests use [`nightmare`](https://github.com/segmentio/nightmare) to simulate user flows in the browser. They run more slowly, so you might not want to run them as often as the unit tests.

The script`tests/one.js` runs a single test file. Like `tests/unit.js` and `tests/functional.js` it runs `require('babel-register')` so that transpilation is done on the fly. You can run it using any of the following commands, providing the test file either as an absolute path, or as a path relative to the `tests` folder. e.g.

```
make test-one/collab/ChangeStore.test.js
node tests/one collab/ChangeStore.test.js
npm run test-one -- collab/ChangeStore.test.js
```


