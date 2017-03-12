<div align="center">
	<a href="https://stenci.la/about">
		<img src="https://raw.githubusercontent.com/stencila/stencila/master/images/logo-name.png" alt="Stencila">
	</a>
	<br>
	<a href="#about">About</a>&nbsp;|&nbsp;<a href="#packages">Packages</a>&nbsp;|&nbsp;<a href="#discuss">Discuss</a>&nbsp;|&nbsp;<a href="#develop">Develop</a>
	<br>
</div>

### About

[Stencila](http://stenci.la) is a platform for creating, collaborating on, and sharing data driven content. Content that is **transparent** and **reproducible**, like [RMarkdown](https://github.com/rstudio/rmarkdown) and [Jupyter](http://jupyter.org/). Content that can be **versioned** and **composed** just like we do with open source software using tools like [CRAN](https://cran.r-project.org/web/packages/available_packages_by_name.html) and [NPM](https://www.npmjs.com/). And above all, content that is **accessible** to non-coders, like [Google Docs](https://en.wikipedia.org/wiki/Google_Docs,_Sheets_and_Slides) and [Microsoft Office](https://en.wikipedia.org/wiki/Microsoft_Office).

<img src="https://raw.githubusercontent.com/stencila/stencila/master/images/document-screenshot.png" alt="Stencila Document">

### Use

Stencila is designed to be used by different types of users and in different use cases. For example, if you just want a quick way to get started, check out the [desktop application](https://github.com/stencila/desktop); but if you're a Python coder, the [`py`](https://github.com/stencila/py) package, will probably be more your thing. This repository doesn't contain any active code itself, it's just here as a central place for shared documentation and issues.

We're currently undertaking a major rewrite of the platform, moving to a more modular and decoupled architecture. It's still a work in progress. So if you're confused or what more info [talk to us](https://gitter.im/stencila/stencila). (Really, do it! :smile:)

A cross platform desktop application is provided by the `desktop` package:

- [desktop](https://github.com/stencila/desktop) : Stencila on the desktop

Currently, five packages expose the platform in alternative host languages (packages for other languages, including Julia and Go, are planned)

- [node](https://github.com/stencila/node) : Stencila for Node.js
- [py](https://github.com/stencila/py) : Stencila for Python
- [r](https://github.com/stencila/r) : Stencila for R
- [cpp](https://github.com/stencila/cpp): Stencila for C++

### Learn

Documentation is available at https://stencila.github.io/stencila.

### Discuss

We love feedback. Chat with members of the community on [Gitter](https://gitter.im/stencila/stencila). For general suggestions you can create a [new issue](issues/new) or add to [existing issues](stencila/issues) in this repo. For package specific issues, please go to Issues page for the package repository (links below). 

### Contribute

Want to help out with development? Great, there's a lot to do! To get started, read our contributor [code of conduct](CONDUCT.md), then [get in touch](https://gitter.im/stencila/stencila) or checkout the [platform-wide, cross-repository kanban board](https://github.com/orgs/stencila/projects/1).

Most development tasks can be run directly from `npm` or via `make` wrapper recipes.

Task                                                    |`npm`                  | `make`          |
------------------------------------------------------- |-----------------------|-----------------|    
Install and setup dependencies                          | `npm install`         | `make setup`
Check code for lint                                     | `npm run lint`        | `make lint`
Run tests                                               | `npm test`            | `make test`
Run tests in the browser                                | `npm run test-bundle` | `make test-bundle`
Run tests with coverage                                 | `npm run cover`       | `make cover`
Build browser bundle                                    | `npm run build`       | `make build`
Build documentation                                     | `npm run docs`        | `make docs`
Serve and watch docs for updates                        | `npm run docs-serve`  | `make docs-serve`
Clean                                                   |                       | `make clean`

Tests live in the `tests` folder and are written using the [`tape`](https://github.com/substack/tape) test harness.

And, in further breathtaking displays of naming logic, documentation lives in the `docs` folder and uses [documentation.js](http://documentation.js.org). Docs are published using Github Pages, so to update them after making changes run `make docs`, commit the updated docs and do a `git push`.


#### Build status, test coverage and issues

               | Version      | Build status | Test coverage | Issues
-------------- |:------------:|:------------:|:-------------:|:------:
*[stencila](https://github.com/stencila/stencila)* | | [![Build status](https://travis-ci.org/stencila/stencila.svg?branch=master)](https://travis-ci.org/stencila/stencila) | - | [![Issues](http://img.shields.io/github/issues/stencila/stencila.svg)]( https://github.com/stencila/stencila/issues )
[cpp](https://github.com/stencila/cpp) | | [![Build status](https://travis-ci.org/stencila/cpp.svg?branch=master)](https://travis-ci.org/stencila/cpp) | [![Test coverage](https://codecov.io/gh/stencila/cpp/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/cpp) | [![Issues](http://img.shields.io/github/issues/stencila/cpp.svg)]( https://github.com/stencila/cpp/issues )
[desktop](https://github.com/stencila/desktop) | | [![Build status](https://travis-ci.org/stencila/desktop.svg?branch=master)](https://travis-ci.org/stencila/desktop) | - | [![Issues](http://img.shields.io/github/issues/stencila/desktop.svg)]( https://github.com/stencila/desktop/issues )
[node](https://github.com/stencila/node) | [![NPM](http://img.shields.io/npm/v/stencila.svg?style=flat)](https://www.npmjs.com/package/stencila) | [![Build status](https://travis-ci.org/stencila/node.svg?branch=master)](https://travis-ci.org/stencila/node) | [![Test coverage](https://codecov.io/gh/stencila/node/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/node) | [![Issues](http://img.shields.io/github/issues/stencila/node.svg)]( https://github.com/stencila/node/issues )
[py](https://github.com/stencila/py) | | [![Build status](https://travis-ci.org/stencila/py.svg?branch=master)](https://travis-ci.org/stencila/py) | [![Test coverage](https://codecov.io/gh/stencila/py/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/py) | [![Issues](http://img.shields.io/github/issues/stencila/py.svg)]( https://github.com/stencila/py/issues )
[r](https://github.com/stencila/r) | | [![Build status](https://travis-ci.org/stencila/r.svg?branch=master)](https://travis-ci.org/stencila/r) | [![Test coverage](https://codecov.io/gh/stencila/r/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/r) | [![Issues](http://img.shields.io/github/issues/stencila/r.svg)]( https://github.com/stencila/r/issues )

#### Branches

[HISTORY.md](HISTORY.md) provides some background to the [`triassic`](https://github.com/stencila/stencila/tree/triassic) and [`jurassic`](https://github.com/stencila/stencila/tree/jurassic) branches in this repo.
