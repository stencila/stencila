## `stencila/ui` : Stencila user interface components

[![NPM](http://img.shields.io/npm/v/stencila-ui.svg?style=flat)](https://www.npmjs.com/package/stencila-ui)
[![Build status](https://travis-ci.org/stencila/ui.svg?branch=master)](https://travis-ci.org/stencila/ui)
[![Code coverage](https://codecov.io/gh/stencila/ui/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/ui)
[![Dependency status](https://david-dm.org/stencila/ui.svg)](https://david-dm.org/stencila/ui)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

This repository is part of the Stencila platform. See our umbrella repository [stencila/stencila](https://github.com/stencila/stencila) for more details.

This repos provides in-browser, user interface components. It also offers a lightweight development environment with service dependencies stubbed out. That allows us iterate quickly on the UI and integrate later.

### Development

```
git clone https://github.com/stencila/ui.git
cd ui
npm install
npm run start
```

Now you can access the different interfaces in the browser:

- http://localhost:4000/examples/document
- http://localhost:4000/examples/sheet


Most development tasks can be run directly using Javascript tooling (`npm`)

Task                                                    | Command               |
------------------------------------------------------- |-----------------------|
Install and setup dependencies                          | `npm install`         |
Run the development server                              | `npm start`           |
Clean and rebuild                                       | `npm run build`       |


### Discuss

We love feedback. Create a [new issue](https://github.com/stencila/ui/issues/new), add to [existing issues](https://github.com/stencila/ui/issues) or [chat](https://gitter.im/stencila/stencila) with members of the community.
