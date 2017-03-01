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

### Packages

Stencila is implemented as an ecosystem of software packages, each exposing the platform to different types of users and use cases. For example, if you just want a quick way to get started, check out the [desktop application](https://github.com/stencila/electron); but if you're a Python coder, the [`py`](https://github.com/stencila/py) package, will probably be more your thing. This repository doesn't contain any active code itself, it's just here as a central place for shared documentation and issues.

We're currently undertaking a major rewrite of the platform, moving to a more modular and decoupled architecture. It's still a work in progress. So if you're confused or what more info [talk to us](https://gitter.im/stencila/stencila). (Really, do it! :smile:)

A cross platform desktop application is provided by the `electron` package:

- [electron](https://github.com/stencila/electron) : Stencila on the desktop

Browser-based user interfaces are provided by the `ui` package: 

- [ui](https://github.com/stencila/ui) : Stencila in the browser

Currently, five packages expose the platform in alternative host languages (packages for other languages, including Julia and Go, are planned)

- [js](https://github.com/stencila/js) : Stencila for Javascript (code shared by the `node` and `ui` packages)
- [node](https://github.com/stencila/node) : Stencila for Node.js
- [py](https://github.com/stencila/py) : Stencila for Python
- [r](https://github.com/stencila/r) : Stencila for R
- [cpp](https://github.com/stencila/cpp): Stencila for C++

These packages are in various stages of development. Together they act as an ecosystem of [peers with differing capabilities](http://blog.stenci.la/diverse-peers/). The `node` package is the current focus for implementation mainly because it can be easily integrated into the `electron` package and made available as a desktop application. But we aim to implement the same capabilities in the other language packages so that eventually, each can be used standalone.

### Discuss

We love feedback. Chat with members of the community on [Gitter](https://gitter.im/stencila/stencila). For general suggestions you can create a [new issue](issues/new) or add to [existing issues](stencila/issues) in this repo. For package specific issues, please go to Issues page for the package repository (links below). 

### Contribute

Want to help out with development? Great, there's a lot to do! To get started, read our contributor [code of conduct](CONDUCT.md), then [get in touch](https://gitter.im/stencila/stencila) or checkout the [platform-wide, cross-repository kanban board](https://github.com/orgs/stencila/projects/1).

#### Development status

Stencila is made of of several `Component` classes. Examples of component classes include `Document`, `Sheet`, and `Folder`. These classes are implemented separately in each of the language packages (e.g. the `py` package). The long term aim is to have all component classes implemented in each of the language packages. That would allow each of those packages to be used standalone (in the meantime, because the packages can act as a [network of peers](http://blog.stenci.la/diverse-peers/), you can workaround this by installing one of the other packages). The `ui` package is focused on user interfaces for each component class and delegate other functionality to implementations in the language packages.

                            | Description                                   | `node`       | `py`         | `r`          | `ui`         
--------------              |--------------                                 |:------------:|:------------:|:------------:|:------------:
`Component`                 | Base class for all components					| ✓            | ✓            | ✓            |
`Host`                      | Serves and orchestrates other components      | ✓            | ✓            | ✓            | ✓            
`Folder`                    | A filesystem folder                           | ✓            |              |              | ✓            
`Document`                  | A document                                    | ✓            | ✓            |              | ✓            
`Sheet`                     | A spreadsheet                                 |              |              |              |             
`Session`                   | Base class for sessions                       | ✓            | ✓            | ✓            | ✓            
&nbsp;&nbsp;`BashSession`   | A Bash session                                | ✓            |              |              | -            
&nbsp;&nbsp;`JsSession`     | A Javascript session                          | ✓            | -            | -            | ✓           
&nbsp;&nbsp;`PySession`     | A Python session                              | -            | ✓            | -            | -            
&nbsp;&nbsp;`RSession`      | A R session                                   | -            | -            | ✓            | -            
&nbsp;&nbsp;`SqliteSession` | A SQLite session                              |              |              | ✓            | -            
`pack`,`unpack`             | Data serialization                            | ✓            | ✓            | ✓            | ✓          

Key: ✓ = implemented to some degree, - = will probably never be implemented, ^ = provided by the above


#### Build status, test coverage and issues

               | Version      | Build status | Test coverage | Issues
-------------- |:------------:|:------------:|:-------------:|:------:
[cpp](https://github.com/stencila/cpp) | | [![Build status](https://travis-ci.org/stencila/cpp.svg?branch=master)](https://travis-ci.org/stencila/cpp) | [![Test coverage](https://codecov.io/gh/stencila/cpp/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/cpp) | [![Issues](http://img.shields.io/github/issues/stencila/cpp.svg)]( https://github.com/stencila/cpp/issues )
[electron](https://github.com/stencila/electron) | | [![Build status](https://travis-ci.org/stencila/electron.svg?branch=master)](https://travis-ci.org/stencila/electron) | - | [![Issues](http://img.shields.io/github/issues/stencila/electron.svg)]( https://github.com/stencila/electron/issues )
[js](https://github.com/stencila/js) | [![NPM](http://img.shields.io/npm/v/stencila-js.svg?style=flat)](https://www.npmjs.com/package/stencila-js) | [![Build status](https://travis-ci.org/stencila/js.svg?branch=master)](https://travis-ci.org/stencila/js) | [![Test coverage](https://codecov.io/gh/stencila/js/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/js) | [![Issues](http://img.shields.io/github/issues/stencila/js.svg)]( https://github.com/stencila/js/issues )
[node](https://github.com/stencila/node) | [![NPM](http://img.shields.io/npm/v/stencila.svg?style=flat)](https://www.npmjs.com/package/stencila) | [![Build status](https://travis-ci.org/stencila/node.svg?branch=master)](https://travis-ci.org/stencila/node) | [![Test coverage](https://codecov.io/gh/stencila/node/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/node) | [![Issues](http://img.shields.io/github/issues/stencila/node.svg)]( https://github.com/stencila/node/issues )
[py](https://github.com/stencila/py) | | [![Build status](https://travis-ci.org/stencila/py.svg?branch=master)](https://travis-ci.org/stencila/py) | [![Test coverage](https://codecov.io/gh/stencila/py/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/py) | [![Issues](http://img.shields.io/github/issues/stencila/py.svg)]( https://github.com/stencila/py/issues )
[r](https://github.com/stencila/r) | | [![Build status](https://travis-ci.org/stencila/r.svg?branch=master)](https://travis-ci.org/stencila/r) | [![Test coverage](https://codecov.io/gh/stencila/r/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/r) | [![Issues](http://img.shields.io/github/issues/stencila/r.svg)]( https://github.com/stencila/r/issues )
[ui](https://github.com/stencila/ui) | [![NPM](http://img.shields.io/npm/v/stencila-ui.svg?style=flat)](https://www.npmjs.com/package/stencila-ui) | [![Build status](https://travis-ci.org/stencila/ui.svg?branch=master)](https://travis-ci.org/stencila/ui) | [![Test coverage](https://codecov.io/gh/stencila/ui/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/ui) | [![Issues](http://img.shields.io/github/issues/stencila/ui.svg)]( https://github.com/stencila/ui/issues )

#### Branches

[HISTORY.md](HISTORY.md) provides some background to the [`triassic`](https://github.com/stencila/stencila/tree/triassic) and [`jurassic`](https://github.com/stencila/stencila/tree/jurassic) branches in this repo.
