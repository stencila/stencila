<img src="http://static.stenci.la/img/logo-name-tagline-500.png" alt="Stencila" style="max-width:300px">

[![Issues](https://img.shields.io/github/issues-raw/badges/shields.svg)](http://waffle.io/stencila/stencila)
[![Ready](https://badge.waffle.io/stencila/stencila.svg?label=1+-+Ready&title=ready)](http://waffle.io/stencila/stencila)
[![Doing](https://badge.waffle.io/stencila/stencila.svg?label=2+-+Doing&title=doing)](http://waffle.io/stencila/stencila)
[![Build](https://travis-ci.org/stencila/stencila.svg?branch=develop)](https://travis-ci.org/stencila/stencila)

### Quick start

Stencila is a platform for creating documents that are driven by data. The core engine is in C++ with thin wrappers in to R and Python (more languages to come). Here's a taste of what Stencila does, illustrated using the R package, but with similar functionality available in the other packages.

Install the R package from our R repository:

```r
install.packages('stencila',repo='http://get.stenci.la/r')
```

There's an optional command line interface which you can install into your path:

```r
require(stencila)
stencila:::install()
```

Stencila is based around stencils, you can create a stencil from strings, or files, of HTML or Cila (our stencil authoring language):

```r
# An empty stencil
s <- Stencil()
# A stencil from an HTML string
s <- Stencil('html://The date is <span data-text="date()">')
# A stencil from a Cila file
s <- Stencil('file://stencil.cila')
```

Internally, stencils are represented by an XHTML tree structure that can be converted to/from HTML and Cila. You can set a stencil's content using HTML and get it back as Cila:

```r
# Set the stencil's content using HTML
s$html('The date is <span data-text="date()">')
# Get the stencil's content as Cila
s$cila()
[1] "The date is {text date()}"
```

Stencils are similar to other template formats and can be rendered to produce textual and graphical outputs. The key difference with stencils is that the output gets embedded within the stencil itself. Or, to express it the other way around, the rendered document retains the template logic.

```r
# Load some data on irises
data(iris)
# Create a stencil that calculates a summary statistic
s <- Stencil('cila://The correlation between petal length and petal width was {text with(iris,cor(Petal.Length,Petal.Width))}.')
# Render it
s$render()
# Get it's rendered content as HTML
s$html()
[1] "<p>\n\tThe correlation between petal length and petal width was <span data-text=\"with(iris,cor(Petal.Length,Petal.Width))\">0.962865431402796</span>.\n</p>"
```

Most templating engines separate the source (the template) from the target (the document). Stencils are [automorphic](https://en.wikipedia.org/wiki/Automorphism) the source (the stencil) is also the target (the stencil). And because stencil's are natively XHTML, this means that you can view, edit and re-render them in place, in your browser:

```r
s$view()
```

<img src="http://static.stenci.la/img/stencila-readme-screenshot.png">

(We're currently doing some major refactoring of the front end Javascript so, depending on when you try this out, stencil editing may not work too well/at all).


### Installing

Head on over to the [releases page](https://github.com/stencila/stencila/releases) for instructions on installing the Stencila package for R or Python. Stencila components are placed in a "store" on your machine. The default Stencila store is `~/.stencila` but you can use other store directories and specify them in a semicolon separated list in an environment variable `STENCILA_STORES`.

### Versioning

We are using [semantic version numbers](http://semver.org/) so versions like "0.y.z" indicate that the library is still in initial development phase. It is still early days so the API will change frequently. Don't rely on API stability until the release of version 1.0.0.

### Contributing

We appreciate any help with Stencila development! The [issues list](https://github.com/stencila/stencila/issues) is a good place for contributing ideas. Even better, visit the kanban board at [waffle.io/stencila/stencila](https://waffle.io/stencila/stencila) or [huboard.com/stencila/stencila](https://huboard.com/stencila/stencila) to see which issues are ready to be tackled.

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

The C++ libraries requires the usual build tool suspects e.g. make, cmake, git. Stencila is developed and tested using [`g++`](https://gcc.gnu.org/). A recent version of `g++` (>=4.8) supporting features of the C++11 standard is necessary. 

##### Linux

Provisioning scripts are available for Linux which install the necessary build tools. These scripts can be run on your machine or simply perused to see what is needed. Currently, provisioning scripts are provided for:

* [Ubuntu 12.04](provision-ubuntu-build-12.04.sh)
* [Ubuntu 14.04](provision-ubuntu-build-14.04.sh)

##### Windows

For Microsoft Windows we recommend building under [MSYS2](http://msys2.github.io/) which provides a up-to-date and convenient way of compiling using the [MinGW-w64](http://mingw-w64.sourceforge.net/) Windows port of `g++`. This section provides a guide to setting up a toolchain for building Stencila packages. Where possible, we use the MSYS2 package manager, `pacman` to install required dependencies. If a package is not available for `pacman` (see the list [here](https://github.com/Alexpux/MSYS2-packages)) then we recommend installing via the `choco` package manager. If the `choco` package is out of date then we resort to manual installation.  

- Install [MYSYS2](https://msys2.github.io/) and [Chocolatey](https://chocolatey.org).

- Open the MYSY2 shell and install necessary build tools and libraries using `pacman`:

	```shell
	pacman -S base-devel cmake gcc git mingw-w64-x86_64-python2-pip msys2-devel unzip zip
	pacman -S openssl libssh2
	```

- Install R using the Windows installer provided at http://www.r-project.org/ and make sure the R binary directory is added to the path e.g.

	```shell
	setx path "%path%;C:\Program Files\R\R-3.2.1\bin"
	```

- Install required R packages:

	```shell
	Rscript -e "install.packages(c('Rcpp','roxygen2','svUnit','XML'),repos='http://cran.us.r-project.org')"
	```

- If you want to deliver any of the Stencila packages to http://get.stenci.la (using Makefile recipes such as `cpp-deliver`, `r-deliver`) you will need to install the Amazon Web Services Command Line Interface. Although there is a Windows Installer is seems easier to install the `awscli` package into the MSYS2 environment as in Linux:

	```shell
	pip install awscli
	```

	For some reason, `aws configure` within an MSYS2 shell does not seem to work, so configure `aws` by adding the files described [here](http://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html#cli-config-files):

	```shell
	mkdir ~/.aws
	nano ~/.aws/credentials
	nano ~/.aws/config
	```

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

#### Building and testing on Windows

If you are running Windows in a virtual machine (e.g. VirtualBox) for better performance it is recommended to use a build  directory that is on the VM (instead of the default `stencila/build/OS/ARCH/VERSION` directory that is within a shared folder on the host). Within the `MinGW-w64 Win64` shell create a build directory on the guest VM, change into the `stencila` directory on the host (in this example, mapped to drive Z:) and then specify the build directory when invoking `make`:

```shell
mkdir -p c:/build
cd /z
make cpp-package BUILD=c:/build
```

For some reason, the Makefile recipe `r-package` causes an error on Windows:

```shell
$ make r-package BUILD=c:/build
cd /c/build/r/3.2; R CMD INSTALL --build stencila
* installing to library 'C:/Program Files/R/R-3.2.1/library'
Error: ERROR: no permission to install to directory 'C:/Program Files/R/R-3.2.1/library'
Makefile:950: recipe for target '/c/build/r/3.2/stencila_0.15.1.zip' failed
make: *** [/c/build/r/3.2/stencila_0.15.1.zip] Error 1
```

To work around this, run the `MinGW-w64 Win64 Shell` *as an administrator* and then run the build command from there e.g.

```shell
cd /c/build/r/3.2
R CMD INSTALL --build stencila
```
