<div align="center">
	<a href="https://stenci.la/about">
		<img src="http://static.stenci.la/img/logo-name-tagline-500.png" alt="Stencila" style="max-width:200px">
	</a>
</div>

Stencila is a platform for creating documents that are driven by data. At present we have two types of documents,

- Stencils : like traditional templates for embedding data analysis and presentation code but designed to allow what-you-see-is-what-you-get editing while still maintaining reproducibility (think [RMarkdown](http://rmarkdown.rstudio.com/) meets [Jade](http://jade-lang.com/) meets Google Docs)

- Sheets : like traditional spreadsheets but with cells that are expressions in the host language (i.e. R or Python or ...) and built from the ground up for transparency, testability and version control while still maintaining accessibility (think [R](https://www.r-project.org/) meets Google Sheets meets [git](https://git-scm.com/))

The core engine is written in C++ with thin wrappers into host languages, e.g. R and Python (Javascript, Julia and more languages to come), and browser based user interfaces. Stencila is designed to be used locally (i.e on your computer) or remotely (i.e. in the cloud, on someone else's computer). To install locally see the instructions below or build one of the packages yourself. If you just want to quickly see what this is all about, go to the hub at https://stenci.la where you can activate a stencil or a sheet to play around with (on someone else's computer!)

Things are still very "beta" so please give us your suggestions by creating an [issue](https://github.com/stencila/stencila/issues) or chatting with us on [Gitter](https://gitter.im/stencila/stencila).

[![Join the chat at https://gitter.im/stencila/stencila](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Issues](https://img.shields.io/github/issues-raw/badges/shields.svg)](http://waffle.io/stencila/stencila)
[![Ready](https://badge.waffle.io/stencila/stencila.svg?label=1+-+Ready&title=ready)](http://waffle.io/stencila/stencila)
[![Doing](https://badge.waffle.io/stencila/stencila.svg?label=2+-+Doing&title=doing)](http://waffle.io/stencila/stencila)
[![Travis](https://travis-ci.org/stencila/stencila.svg?branch=master)](https://travis-ci.org/stencila/stencila)
[![Appveyor](https://ci.appveyor.com/api/projects/status/github/stencila/stencila?branch=master&svg=true)](https://ci.appveyor.com/project/nokome/stencila)
[![codecov](https://codecov.io/gh/stencila/stencila/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/stencila)

### Installing

Head on over to the [releases page](https://github.com/stencila/stencila/releases) for instructions on installing the Stencila package for R or Python. At this stage, builds are only available for Linux 64 bit. If you are on a different operating system you can try building one of the packages yourself (instructions below). We have successfully built the R package on Windows 7 and hope to make this, and an OS X build, part of our regular build set soon.

Alternatively, you can use one of the Docker images: [`stencila/ubuntu-14.04-python-2.7`](https://hub.docker.com/r/stencila/ubuntu-14.04-python-2.7/) or [`stencila/ubuntu-14.04-r-3.2`](https://hub.docker.com/r/stencila/ubuntu-14.04-r-3.2/) (see [docker/README.md](docker/README.md) for more...)

Stencila components are placed in a "store" on your machine. The default Stencila store is `~/.stencila` but you can use other store directories and specify them in a semicolon separated list in an environment variable `STENCILA_STORES`.

### Contributing

We appreciate any help with Stencila development! The [issues list](https://github.com/stencila/stencila/issues) is a good place for contributing ideas. Or, visit the kanban board at [waffle.io/stencila/stencila](https://waffle.io/stencila/stencila) to see which issues are ready to be tackled and what's being worked on.

Occasionally, bounties are put on issues. Rather than duplicating effort we ask 

Please also not that we have a [code of conduct](CONDUCT.md).

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

* [Ubuntu 12.04](setup/ubuntu-build-12.04.sh)
* [Ubuntu 14.04](setup/ubuntu-build-14.04.sh)

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

### Versioning

We are using [semantic version numbers](http://semver.org/) so versions like "0.y.z" indicate that the library is still in initial development phase. It is still early days so the API will change frequently. Don't rely on API stability until the release of version 1.0.0.

