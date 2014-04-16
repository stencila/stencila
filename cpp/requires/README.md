## Stencila C++ requirements

The Stencila C++ module has a number of required dependencies. At present these are:

* [Boost](http://www.boost.org/) libraries
* [libgit2](http://libgit2.github.com/)
* [pugixml](http://pugixml.org/)
* [rapidjson](https://code.google.com/p/rapidjson/)
* [tidy-html5](http://w3c.github.com/tidy-html5/)
* [WebSocket++](https://github.com/zaphoyd/websocketpp)

Version numbers for each of these is specified in their respective Makefiles (e.g [`boost.make`](boost.make))

The Makefiles install required libraries in the Stencila `cpp/requires` directory. We have done this to avoid creating clashes between what Stencila requires and what is already on your system. The file `Makefile.common` in the `cpp` directory defines compiler options for including `cpp/requires/include` and `cpp/requires/lib` in the compiler search path.

If you already have one or more of these packages on your system (for example in `/usr/local/include` and `/usr/local/lib`) and you don't want to build them again then you might get away with the version that you already have. But you might not.

The Makefiles download and build each requirement and so themselves have some prerequisites:

* wget - for downloading files
* tar, unzip - for unpacking tar and zip files
* make and gcc - for building boost, sqlite etc

If you are on Ubuntu, or another Linux with apt-get, you can get all these with:

```sh
sudo apt-get install wget tar unzip cmake make gcc
```

We link statically to these libraries and distribute a large dynamic library for each Stencila package (e.g R and Python packages). This requires that all libraries are compiled with Position Independent Code (i.e. -fPIC gcc flag). An alternative would be to link dynamically to these libraries and then do some dependency checking on the user's machine to see which dynamic libaries need to be installed. Whilst distributing a large static library is not ideal, at present it is preferred over doing the more complex dependency checking which includes ensuring the right version of libraries is available. See the folowing for discussion:

* http://stackoverflow.com/questions/1412080/distributing-with-boost-library
* http://tldp.org/HOWTO/Program-Library-HOWTO/shared-libraries.html
