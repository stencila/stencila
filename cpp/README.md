# Stencila C++ library

Documentation for the Stencila C++ library (generated using Doxygen) is available [here](http://stencila.github.io/stencila/cpp).

Make will build C++ in `build/OS/ARCH/VERSION/cpp`. There are separate Makefile recipes related to the C++ library. e.g

* `make cpp-requires` : build all requirements for the C++ package
* `make cpp-requires-boost` etc : build the required Boost libraries
* `make cpp-package` : build the C++ package

## Tool chain requirements

The C++ libraries requires the usual build tool suspects e.g. make, cmake, git. Stencila is developed and tested using [`g++`](https://gcc.gnu.org/). A recent version of `g++` (>=4.8) supporting features of the C++11 standard is necessary. 

### Linux

Provisioning scripts are available for Linux which install the necessary build tools. These scripts can be run on your machine or simply perused to see what is needed. Currently, provisioning scripts are provided for:

* [Ubuntu 12.04](../setup/ubuntu-build-12.04.sh)
* [Ubuntu 14.04](../setup/provision-ubuntu-build-14.04.sh)

### Windows

For Microsoft Windows we recommend building under [MSYS2](http://msys2.github.io/) which provides a up-to-date and convenient way of compiling using the [MinGW-w64](http://mingw-w64.sourceforge.net/) Windows port of `g++`. This section provides a guide to setting up a toolchain for building Stencila packages. Where possible, we use the MSYS2 package manager, `pacman` to install required dependencies. If a package is not available for `pacman` (see the list [here](https://github.com/Alexpux/MSYS2-packages)) then we recommend installing via the `choco` package manager. If the `choco` package is out of date then we resort to manual installation.  

Install [MYSYS2](https://msys2.github.io/) open the MYSY2 shell and install necessary build tools and libraries using `pacman`:

```sh
pacman -S base-devel cmake gcc git mingw-w64-x86_64-python2-pip msys2-devel unzip zip
pacman -S openssl libssh2
```

If you want to publish any of the Stencila packages to http://get.stenci.la (using Makefile recipes such as `make publish`) you will need to install the Amazon Web Services Command Line Interface. Although there is a Windows Installer is seems easier to install the `awscli` package into the MSYS2 environment as in Linux:

```shell
pip install awscli
```

For some reason, `aws configure` within an MSYS2 shell does not seem to work, so configure `aws` by adding the files described [here](http://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html#cli-config-files):

```shell
mkdir ~/.aws
nano ~/.aws/credentials
nano ~/.aws/config
```

## Library requirements

The Stencila C++ package has a number of required dependencies. At present these are:

* [Boost](http://www.boost.org/)
* [libgit2](http://libgit2.github.com/)
* [pugixml](http://pugixml.org/)
* [rapidjson](https://code.google.com/p/rapidjson/)
* [tidy-html5](http://w3c.github.com/tidy-html5/)
* [WebSocket++](https://github.com/zaphoyd/websocketpp)

The [Makefile](Makefile) defines the required version of each of these libraries and `make cpp-requires` will build and install them in the `cpp/requires` directory.We have taken this approach of local installs to avoid clashes with different versions that may already be globally installed on your machine. Local installation is unusual for C/C++ libraries but allows for better management of dependencies and is a modern approach used in other languages (e.g. [virtualenv for Python](http://virtualenv.readthedocs.org/en/latest/virtualenv.html) and the [Node package manager](https://www.npmjs.org/doc/cli/npm-install.html))

We link statically to these libraries and distribute a large dynamic library for each Stencila package (e.g R and Python packages). On Linux this requires that all libraries are compiled with Position Independent Code (i.e. -fPIC gcc flag). An alternative would be to link dynamically to these libraries and then do some dependency checking on the user's machine to see which dynamic libaries need to be installed. Whilst distributing a large static library is not ideal, at present it is preferred over doing the more complex dependency checking which includes ensuring the right version of libraries is available. See [here](http://stackoverflow.com/questions/1412080/distributing-with-boost-library) and [here](http://tldp.org/HOWTO/Program-Library-HOWTO/shared-libraries.html) for further discussion.

