# Stencila C++ requirements

The Stencila C++ library has a number of required dependencies. At present these are:

* [Boost](http://www.boost.org/) libraries
* [cpp-netlib](http://cpp-netlib.github.com/) networking library
* [rapidjson](http://code.google.com/p/rapidjson/) JSON parser/generator
* [smhasher](http://code.google.com/p/smhasher/) hashing library
* [sqlite] (http://www.sqlite.org/) SQL database engine

This directory provides a `Makefile` for making it easier to install these prerequisites. 
The Makefile downloads and builds each requirement and so itself has some prerequisites:

* wget - for downloading files
* svn - for getting smhasher
* tar, unzip - for unpacking tar and zip files
* cmake - for building cpp-netlib
* make and gcc - for building boost, cpp-netlib, sqlite etc

If you are on Ubuntu, or another Linux with apt-get, you can get all these files with:

```sh
sudo apt-get install wget svn tar unzip cmake make gcc
```

Example usage:

```sh
make smhasher sqlite
```

The `Makefile` installs prerequisite packages in the Stencila `cpp/requirements` directory. 
We have done this to avoid creating clashes between what Stencila requires and what is already on your system.
The file `Makefile.common` in the `cpp` directory defines compiler options for including `cpp/requirements/include` and `cpp/requirements/lib` in the compiler search path

If you already have one or more of these packages on your system (for example in `/usr/local/include` and `/usr/local/lib`) and you don't want to build them again then you 
might get away with the version that you already have. But you might not. Try it, if it doesn't work you can always come back here!





