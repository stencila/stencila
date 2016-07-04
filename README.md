<div align="center">
	<a href="https://stenci.la/about">
		<img src="http://static.stenci.la/img/logo-name-tagline-500.png" alt="Stencila" style="max-width:200px">
	</a>
</div>

[![Join the chat at https://gitter.im/stencila/stencila](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Issues](https://img.shields.io/github/issues-raw/badges/shields.svg)](http://waffle.io/stencila/stencila)
[![Ready](https://badge.waffle.io/stencila/stencila.svg?label=1+-+Ready&title=ready)](http://waffle.io/stencila/stencila)
[![Doing](https://badge.waffle.io/stencila/stencila.svg?label=2+-+Doing&title=doing)](http://waffle.io/stencila/stencila)
[![Travis](https://travis-ci.org/stencila/stencila.svg?branch=master)](https://travis-ci.org/stencila/stencila)
[![Appveyor](https://ci.appveyor.com/api/projects/status/github/stencila/stencila?branch=master&svg=true)](https://ci.appveyor.com/project/nokome/stencila)
[![codecov](https://codecov.io/gh/stencila/stencila/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/stencila)<a href="#codecov-footnote"><sup> Note</sup></a>

### About

Stencila is a platform for creating documents that are driven by data,

- Stencils : like traditional templates for embedding data analysis and presentation code but designed to allow what-you-see-is-what-you-get editing while still maintaining reproducibility (think [RMarkdown](http://rmarkdown.rstudio.com/) meets [Jade](http://jade-lang.com/) meets Google Docs)

- Sheets : like traditional spreadsheets but with cells that are expressions in the host language (i.e. R or Python or ...) and built from the ground up for transparency, testability and version control while still maintaining accessibility (think [R](https://www.r-project.org/) meets Google Sheets meets [git](https://git-scm.com/))

The core engine is written in C++ with thin wrappers into host languages, e.g. R and Python (Javascript, Julia and more languages to come), and browser based user interfaces. Stencila is designed to be used locally (i.e on your computer) or remotely (i.e. in the cloud, on someone else's computer). To install locally see the instructions below or build one of the packages yourself.

### Status

Things are still *very* "beta" so please give us your suggestions by creating an [issue](https://github.com/stencila/stencila/issues) or chatting with us on [Gitter](https://gitter.im/stencila/stencila). See [releases](https://github.com/stencila/stencila/releases) for what we've been working on lately and [milestones](https://github.com/stencila/stencila/milestones) for what we've got planned.

The various modules are in various states of development for various operating systems. Here's a summary of what is and isn't working on [Travis CI](https://travis-ci.org/stencila/stencila) (for Linux and Mac) and [Appveyor](https://ci.appveyor.com/project/nokome/stencila) (Windows): ✓ indicates a Makefile recipe is implemented and working, a ✗ means its been implemented and is failing (an hopefully has an associated issue number), blank means it hasn't been attempted. See the relevant module `Makefile` for what `setup`, `build`, `test`, `cover` and `publish` actually do. 

Code coverage statistics for each module are available on [Codecov](https://codecov.io/gh/stencila/stencila). <a name="codecov-note">Note: </a> Code coverage is currently very low. We're working on improving that but also don't place too high a priority on code coverage while version < 1.0.0 and thus the API is changing (because tests need to get rewritten a lot!).

               | Linux | Mac   | Win. 
-------------- |:-----:|:-----:|:-----:
**C++**        |
`setup`        | ✓     | ✓     | ✓     
`build`        | ✓     | ?     | ✓     
`test`         | ✓     |       | ✗ #199   
`cover`        | ✓     |       |      
`publish`      | ✓     |       | 
               |
**Node.js**    | 4.4   |       |
`setup`        | ✓     | ✓     |     
`build`        | ✓     | ✗ #196|     
`test`         | ✓     |       |
`cover`        | ✓     |       |      
`publish`      | ✓     |       | 
               |
**Python**     | 2.7   |       |
`setup`        | ✓     | ✓     | ✓     
`build`        | ✓     | ✓     | ✓     
`test`         | ✓     |       | ✗ #200
`cover`        | ✓     |       |      
`publish`      | ✓     |       | 
               |
**R**          | 3.2, 3.3|     |
`setup`        | ✓     | ✓     |     
`build`        | ✓     | ✗ #197|     
`test`         | ✓     |       |
`cover`        | ✓     |       |      
`publish`      | ✓     |       |


### Installing

Some one-liners for installing Stencila for supported languages...

- Node.js : `npm install stencila/stencila`
- Python : `pip install http://get.stenci.la/py/stencila-0.23.0-cp27-none-linux_x86_64.whl`
- R : `install.packages('stencila', repos='http://get.stenci.la/r')`

These will only work if `publish` has a ✓ for your operating system in the table above. If a build is not yet available for your operating system you could use one of the [Docker images](docker) ... or, you could [help out](#contributing) with getting the builds to work :)

See https://stenci.la/builds for the latest builds with version specific download instructions.

All Stencila components expose a Web API (HTTP and WebSocket) so you can interact with them remotely on someone else's computer (aka "the cloud"). If you want to get a quick taste of what Stencils and Sheets look and feel like, you can play with live examples, or create your own, over on the Stencila hub at https://stenci.la.

### Contributing

We always appreciate any help with Stencila development! The [issues list](https://github.com/stencila/stencila/issues) is a good place for contributing ideas. Or, visit the kanban board at [waffle.io/stencila/stencila](https://waffle.io/stencila/stencila) to see which issues are ready to be tackled and what's being worked on.

### Versioning, milestones and roadmap

We are using [semantic version numbers](http://semver.org/) so versions like "0.y.z" indicate that we're still in initial development phase i.e "it's is still early days so the API will change frequently!". Don't rely on API stability until the release of version 1.0.0.

At this point in time we don't have a well defined long term roadmap - instead, as we go, we're seeing what people find useful, what they think needs more work, and iterating accordingly. But we do have a short term roadmap which is usually defined by one or two [release milestones](https://github.com/stencila/stencila/milestones) targeted for the following 1-2 months.
