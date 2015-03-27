![Stencila](http://static.stenci.la/img/logo-name-400x88.png)

Documents powered by data

### Status

Issues:
[![Ideas](https://badge.waffle.io/stencila/stencila.svg?label=0+-+Ideas&title=ideas)](http://waffle.io/stencila/stencila)
[![Ready](https://badge.waffle.io/stencila/stencila.svg?label=1+-+Ready&title=ready)](http://waffle.io/stencila/stencila)
[![Doing](https://badge.waffle.io/stencila/stencila.svg?label=2+-+Doing&title=doing)](http://waffle.io/stencila/stencila)

Build:
[![Build](https://travis-ci.org/stencila/stencila.svg?branch=develop)](https://travis-ci.org/stencila/stencila)

### Quick start

Stencila components (e.g. stencils, themes) are git repositories. The first thing to do is clone some of the core repositories into a Stencila "store" directory:

```sh
mkdir ~/.stencila
cd ~/.stencila
git clone https::/stenci.la/core/themes/base core/themes/base
git clone https::/stenci.la/core/stencils/themes/default core/stencils/themes/default
git clone https::/stenci.la/core/stencils/examples/kitchensink core/stencils/examples/kitchensink
```

It is important to clone into the correct directories, as shown above, to get the Stencila components' address space right. The default Stencila store is `~/.stencila` but you can use other store directories and specify them in a semicolon separated list in an environment variable `STENCILA_STORES`.

Then install the package file you downloaded from the [releases page](https://github.com/stencila/stencila/releases) or built yourself...

R
```R
install.packages('Rcpp')
install.packages('stencila_0.10.tar.gz',repos=NULL)
```

Python 2.7
```py
sudo pip install stencila-0.10_-cp27-none-linux_x86_64.whl
```

Load and view a stencil for editing...for example, in R,

```R
require(stencila)
s <- Stencil('core/stencils/examples/kitchensink')
s$view()
```

### Releases and versioning

Head on over to the [releases page](https://github.com/stencila/stencila/releases) for more info on progress so far. It is still very early days so the API will change frequently. We are using [semantic version numbers](http://semver.org/) so versions like "0.y.z" indicate that the library is still in initial development phase. Don't rely on API stability until the release of version 1.0.0.

### Contributing

We appreciate any help with Stencila development! The [issues list](https://github.com/stencila/stencila/issues) is a good place for contributing ideas. Even better, visit the Stencila [Kanban board](https://huboard.com/stencila/stencila) at [waffle.io/stencila/stencila](https://waffle.io/stencila/stencila) or [huboard.com/stencila/stencila](https://huboard.com/stencila/stencila) to see which issues are ready to be tackled.

### Building

#### Building quick start

Running `make` will build the C++, Python and R packages in a subdirectory corresponding to `build/OS/ARCH/VERSION`. There are separate Makefile shortcuts. e.g

* `make cpp-requires` : build all requirements for the C++ package
* `make cpp-requires-boost` etc : build the required Boost libraries
* `make cpp-package` : build the C++ package
* `make py-tests` : run Python tests suites
* `make r-package` : build the R package
* `make r-tests` : run R test suites
* `make r-install` : install the R package on the host machine

These shortcut tasks should build necessary dependencies e.g. `r-tests` first builds the R package.

#### Tool chain requirements

Stencila is developed and tested using [`g++`](https://gcc.gnu.org/). A recent version of `g++` (>=4.8) supporting features of the C++11 standard is necessary. For Microsoft Windows we recommend building under [MSYS2](http://msys2.github.io/) which provides a up-to-date and convenient way of compiling using the [MinGW-w64](http://mingw-w64.sourceforge.net/) Windows port of `g++`.

Provisioning scripts are available to install the necessary build tools e.g. g++, make, cmake. These scripts can be run on your machine or consulted to see what is needed. Currently, provisioning scripts are provided for:

* [Ubuntu 14.04](provision-ubuntu-14.04.sh)
* [MSYS2](provision-msys2.sh)

#### C++ package requirements

The Stencila C++ package has a number of required dependencies. At present these are:

* [Boost](http://www.boost.org/)
* [libgit2](http://libgit2.github.com/)
* [pugixml](http://pugixml.org/)
* [rapidjson](https://code.google.com/p/rapidjson/)
* [tidy-html5](http://w3c.github.com/tidy-html5/)
* [WebSocket++](https://github.com/zaphoyd/websocketpp)

The [Makefile](Makefile) defines the required version of each of these libraries and `make cpp-requires` will build and install them in the `cpp/requires` directory.We have taken this approach of local installs to avoid clashes with different versions that may already be globally installed on your machine. Local installation is unusual for C/C++ libraries but allows for better management of dependencies and is a modern approach used in other languages (e.g. [virtualenv for Python](http://virtualenv.readthedocs.org/en/latest/virtualenv.html) and the [Node package manager](https://www.npmjs.org/doc/cli/npm-install.html))

We link statically to these libraries and distribute a large dynamic library for each Stencila package (e.g R and Python packages). On Linux this requires that all libraries are compiled with Position Independent Code (i.e. -fPIC gcc flag). An alternative would be to link dynamically to these libraries and then do some dependency checking on the user's machine to see which dynamic libaries need to be installed. Whilst distributing a large static library is not ideal, at present it is preferred over doing the more complex dependency checking which includes ensuring the right version of libraries is available. See [here](http://stackoverflow.com/questions/1412080/distributing-with-boost-library) and [here](http://tldp.org/HOWTO/Program-Library-HOWTO/shared-libraries.html) for further discussion.

#### Building and testing with Vagrant

[Vagrant](https://www.vagrantup.com/) is a tool for creating lightweight, reproducible, and portable development environments. If you want to build Stencila for different operating systems or architectures we recommend using Vagrant. The [Vagrantfile](Vagrantfile) includes multiple virtual machine (VM) configurations and uses the provisioning scripts to setup each VM with the tools needed to build Stencila. See the comments at the top of the [Vagrantfile](Vagrantfile) for instructions.

