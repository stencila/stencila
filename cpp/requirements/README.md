## Stencila C++ requirements

### Libraries

The Stencila C++ library has a number of required dependencies. At present these are:

* [Boost](http://www.boost.org/) libraries
* [cpp-netlib](http://cpp-netlib.org/) networking library
* [libarchive](http://www.libarchive.org/) multi-format archive and compression library
* [pugixml](http://pugixml.org/) XML processing library
* [rapidjson](http://code.google.com/p/rapidjson/) JSON library
* [sqlite] (http://www.sqlite.org/) SQL database engine
* [tidy-html5](http://w3c.github.io/tidy-html5/) HTML tidying library

The version numbers for each of these is specified in the Makefile.

In addition, [OpenSSL](http://www.openssl.org/) is required for the `ssl` and `crypto` libraries. Linux usually has packages for this: e.g.

```sh
sudo apt-get install libssl-dev
```

### Linking

The current strategy is to link statically to these libraries and distribute a large dynamic library for each Stencila package (e.g R and Python packages). This requires that all libraries are compiled with Position Independent Code (i.e. -fPIC gcc flag). An alternative would be to link dynamically to these libraries and then do some dependency checking on the user's machine to see which dynamic libaries need to be installed. Whilst distributing a large static library is not ideal, at present it is preferred over doing the more complex dependency checking which includes ensuring the right version of libraries is available. See the folowing for discussion:

* http://stackoverflow.com/questions/1412080/distributing-with-boost-library
* http://tldp.org/HOWTO/Program-Library-HOWTO/shared-libraries.html

### Makefile

This directory provides a `Makefile` for making it easier to install the above libraries. Example usage:

```sh
make boost sqlite
```

The `Makefile` installs required libraries in the Stencila `cpp/requirements` directory. We have done this to avoid creating clashes between what Stencila requires and what is already on your system. The file `Makefile.common` in the `cpp` directory defines compiler options for including `cpp/requirements/include` and `cpp/requirements/lib` in the compiler search path.

If you already have one or more of these packages on your system (for example in `/usr/local/include` and `/usr/local/lib`) and you don't want to build them again then you might get away with the version that you already have. But you might not. Try it, if it doesn't work you can always come back here!

The Makefile downloads and builds each requirement and so itself has some prerequisites:

* wget - for downloading files
* tar, unzip - for unpacking tar and zip files
* cmake - for building cpp-netlib
* make and gcc - for building boost, cpp-netlib, sqlite etc

If you are on Ubuntu, or another Linux with apt-get, you can get all these files with:

```sh
sudo apt-get install wget tar unzip cmake make gcc
```



