<div align="center">
	<a href="https://stenci.la/about">
		<img src="http://static.stenci.la/img/logo-name-tagline-500.png" alt="Stencila" style="max-width:200px">
	</a>
	<br>
	<a href="#about">About</a>&nbsp;.&nbsp;<a href="#packages">Packages</a>&nbsp;.&nbsp;<a href="#discuss">Discuss</a>&nbsp;.&nbsp;<a href="#develop">Develop</a>
	<br>
</div>

### About

[Stencila](http://stenci.la) is a platform for creating, collaborating on, and sharing data driven content. Content that is **transparent** and **reproducible**. Content that is **composable** and **versionable**. Content that is **accessible** to a variety of users.

### Packages

Stencila is implemented as an ecosystem of software packages, each exposing the platform to different types of users and use cases. So, for example if you're a Python coder, you'll probably be most interested in the `py` package. Or, if you just want a quick way to get started with a desktop client, check out the `electron` package. This repository doesn't contain any active code itself, it's just here as a central place for shared documentation and issues.

We're currently undertaking a major rewrite of the platform, moving to a more modular and decoupled architecture. It's still a work in progress.

Currently, four packages expose the platform in alternative host languages (packages for other languages, including Julia and Go, are planned)

- [node](https://github.com/stencila/node) : Stencila for Node.js
- [py](https://github.com/stencila/py) : Stencila for Python
- [r](https://github.com/stencila/r) : Stencila for R
- [cpp](https://github.com/stencila/cpp): Stencila for C++

All these packages make use of the browser based user interfaces provided by the `web` package: 

- [web](https://github.com/stencila/web) : Stencila in the browser

A cross platform desktop application is provided by the `electron` package:

- [electron](https://github.com/stencila/electron) : Stencila on the desktop

### Discuss

We love feedback. Create a [new issue](issues/new), add to [existing issues](stencila/issues) or chat with members of the community on [Gitter](https://gitter.im/stencila/stencila).

### Develop

If you would like to help out with development, great! Please see the roadmaps and issues for each of the repos:

               | Build status | Test coverage 
-------------- |:------------:|:-------------:
[cpp](https://github.com/stencila/cpp) | [![Build status](https://travis-ci.org/stencila/cpp.svg?branch=master)](https://travis-ci.org/stencila/cpp) | [![Test coverage](https://codecov.io/gh/stencila/cpp/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/cpp)
[electron](https://github.com/stencila/electron) | [![Build status](https://travis-ci.org/stencila/electron.svg?branch=master)](https://travis-ci.org/stencila/electron) | [![Test coverage](https://codecov.io/gh/stencila/electron/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/electron)
[node](https://github.com/stencila/node) | [![Build status](https://travis-ci.org/stencila/node.svg?branch=master)](https://travis-ci.org/stencila/node) | [![Test coverage](https://codecov.io/gh/stencila/node/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/node)
[py](https://github.com/stencila/py) | [![Build status](https://travis-ci.org/stencila/py.svg?branch=master)](https://travis-ci.org/stencila/py) | [![Test coverage](https://codecov.io/gh/stencila/py/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/py)
[r](https://github.com/stencila/r) | [![Build status](https://travis-ci.org/stencila/r.svg?branch=master)](https://travis-ci.org/stencila/r) | [![Test coverage](https://codecov.io/gh/stencila/r/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/r)
[web](https://github.com/stencila/web) | [![Build status](https://travis-ci.org/stencila/web.svg?branch=master)](https://travis-ci.org/stencila/web) | [![Test coverage](https://codecov.io/gh/stencila/web/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/web)

### Museum

As with many software projects, Stencila has evolved a lot as we have learned what works and what doesn't. Two independent branches have been preserved in this repo for historical interest (and because they contain a significant amount of work and some useful code!):

#### [`triassic`](https://github.com/stencila/stencila/tree/triassic) c. 2012-2013. 305 commits

#### [`jurassic`](https://github.com/stencila/stencila/tree/jurassic) c. 2014-2016. 2414 commits
