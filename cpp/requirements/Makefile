include  $(dir $(lastword $(MAKEFILE_LIST)))/../cpp.mk

BOOST_VERSION = 1_53_0
CPP_NETLIB_VERSION = 0.9.4
LIBARCHIVE_VERSION = 3.1.2
PUGIXML_VERSION = 1.2
RAPIDJSON_VERSION = 0.11
SMHASHER_VERSION = 147
SQLITE_VERSION = 3071502

# Specify that symlinks time should be checked, not the target file time.
# If this option is not specified then symlinks get remade
# Note also in rules below that the rules for creating symlinks do NOT depend upon
# the 'include' and 'lib' rules : if they do then you get cyclic updating because the symlinks
# change the times of those directories
MAKEFLAGS += --check-symlink-times

all: boost cpp-netlib libarchive pugixml rapidjson smhasher sqlite tidy-html5

.PHONY: boost cpp-netlib libarchive pugixml rapidjson smhasher sqlite tidy-html5

####################################################
# include and lib directories 
# 
# Where soft links to C++ files and compiles libraries go

include: 
	mkdir -p include

lib:
	mkdir -p lib

####################################################
# boost
#

boost_$(BOOST_VERSION).tar.bz2:
	wget http://prdownloads.sourceforge.net/boost/boost_$(BOOST_VERSION).tar.bz2

boost_$(BOOST_VERSION): boost_$(BOOST_VERSION).tar.bz2
	tar --bzip2 -xf boost_$(BOOST_VERSION).tar.bz2
	touch boost_$(BOOST_VERSION)

# Boost is built with some options to override defaults
#  	--prefix=.  - so that boost installs into its own directory  (boost_$(BOOST_VERSION))
# 	link=static - so that get statically compiled instead of dynamically compile libraries
boost_$(BOOST_VERSION)/lib/libboost_system.a: boost_$(BOOST_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	cd boost_$(BOOST_VERSION); \
	./bootstrap.sh; \
	./b2 --prefix=. link=static install
endif
# Under MSYS some differences are required
#	- bootstrap.sh must be called with mingw specified as toolset otherwise errors occur
#	- project-config.jam must be edited to fix the [error](http://stackoverflow.com/a/5244844/1583041) produced by the above command
#	- b2 must be called with "system" layout of library names and header locations (otherwise it defaults to 'versioned' on Windows)
#	- b2 must be called with "release" build otherwise defaults to debug AND release, which with "system" causes an error (http://boost.2283326.n4.nabble.com/atomic-building-with-layout-system-mingw-bug-7482-td4640920.html)
ifeq ($(STENCILA_PLATFORM), msys)
	cd boost_$(BOOST_VERSION); \
	./bootstrap.sh --with-toolset=mingw; \
	sed -i "s/mingw/gcc/g" project-config.jam; \
	./b2 --prefix=. --layout=system release toolset=gcc link=static install
endif
	touch boost_$(BOOST_VERSION)/lib/libboost_system.a

include/boost: boost_$(BOOST_VERSION)/lib/libboost_system.a include boost_$(BOOST_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../boost_$(BOOST_VERSION)/include/boost include/boost
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/boost boost_$(BOOST_VERSION)/include/boost
endif
	touch include/boost

# Link to each of the statically compiled libraries
lib/libboost_system.a: lib boost_$(BOOST_VERSION)/lib/libboost_system.a
	for file in $$(ls boost_$(BOOST_VERSION)/lib/*.a); do ln -sf ../$$file lib; done

boost: include/boost lib/libboost_system.a

####################################################
# cpp-netlib
#

cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz:
	wget --no-check-certificate https://github.com/downloads/cpp-netlib/cpp-netlib/cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz

cpp-netlib-$(CPP_NETLIB_VERSION): cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz
	tar xzf cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz

# cpp-netlib needs to be compiled with OPENSSL_NO_SSL2 defined because SSL2 is insecure and depreciated and on
# some systems (e.g. Ubuntu) OpenSSL is compiled with no support for it
CPP_NETLIB_CMAKE_FLAGS := -DCMAKE_BUILD_TYPE=Debug  -DCMAKE_C_COMPILER=$(CC) -DCMAKE_CXX_COMPILER=$(CXX) -DCMAKE_CXX_FLAGS=-DOPENSSL_NO_SSL2
# Under MSYS some additional CMake flags need to be specified
# The "-I/usr/include" in -DCMAKE_CXX_FLAGS seems uncessary but it's not
ifeq ($(STENCILA_PLATFORM), msys)
CPP_NETLIB_CMAKE_FLAGS += -G "MSYS Makefiles" -DOPENSSL_INCLUDE_DIR=/usr/include/ -DOPENSSL_LIBRARIES=/usr/lib/ -DCMAKE_CXX_FLAGS="-DOPENSSL_NO_SSL2 -I/usr/include"
endif
cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/libcppnetlib-client-connections.a: boost_$(BOOST_VERSION) cpp-netlib-$(CPP_NETLIB_VERSION)
	cd cpp-netlib-$(CPP_NETLIB_VERSION); \
	export BOOST_ROOT=../boost_$(BOOST_VERSION) ; \
	cmake $(CPP_NETLIB_CMAKE_FLAGS); \
	make cppnetlib-client-connections cppnetlib-server-parsers cppnetlib-uri

# Link to cpp-netlib include directory and each of the statically compiled libraries
# cpp-netlib is going through process of inclusion in boost and needs to be available in the include path "boost/network"
include/boost/network: boost_$(BOOST_VERSION) cpp-netlib-$(CPP_NETLIB_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../../../cpp-netlib-$(CPP_NETLIB_VERSION)/boost/network boost_$(BOOST_VERSION)/include/boost/network
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction boost_$(BOOST_VERSION)/include/boost/network cpp-netlib-$(CPP_NETLIB_VERSION)/boost/network
endif
	touch include/boost/network

lib/libcppnetlib-client-connections.a: cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/libcppnetlib-client-connections.a
	for file in $$(ls cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/*.a); do ln -sf ../$$file lib; done

cpp-netlib: include lib include/boost/network lib/libcppnetlib-client-connections.a

####################################################
# libarchive
#
# Build instructions: https://github.com/libarchive/libarchive/wiki/BuildInstructions

libarchive-$(LIBARCHIVE_VERSION).tar.gz:
	wget http://www.libarchive.org/downloads/libarchive-$(LIBARCHIVE_VERSION).tar.gz

libarchive-$(LIBARCHIVE_VERSION): libarchive-$(LIBARCHIVE_VERSION).tar.gz
	tar xzf libarchive-$(LIBARCHIVE_VERSION).tar.gz

libarchive-$(LIBARCHIVE_VERSION)/.libs/libarchive.a:
	cd libarchive-$(LIBARCHIVE_VERSION);\
	./configure;\
	make;\
	make check;

include/libarchive: include libarchive-$(LIBARCHIVE_VERSION)/libarchive
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../libarchive-$(LIBARCHIVE_VERSION)/libarchive include/libarchive
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/libarchive ../libarchive-$(LIBARCHIVE_VERSION)/libarchive
endif
	touch include/libarchive

lib/libarchive.a: lib libarchive-$(LIBARCHIVE_VERSION)/.libs/libarchive.a
	ln -sfT ../libarchive-$(LIBARCHIVE_VERSION)/.libs/libarchive.a lib/libarchive.a
	touch lib/libarchive.a

libarchive: include/libarchive lib/libarchive.a

####################################################
# pugixml
#
# Installation instructions: http://pugixml.googlecode.com/svn/tags/latest/docs/manual/install.html

pugixml-$(PUGIXML_VERSION).tar.gz:
	wget http://pugixml.googlecode.com/files/pugixml-$(PUGIXML_VERSION).tar.gz
	
pugixml-$(PUGIXML_VERSION): pugixml-$(PUGIXML_VERSION).tar.gz
	mkdir -p pugixml-$(PUGIXML_VERSION)
	cd pugixml-$(PUGIXML_VERSION) && tar xzf ../pugixml-$(PUGIXML_VERSION).tar.gz

pugixml-$(PUGIXML_VERSION)/src/libpugixml.a: pugixml-$(PUGIXML_VERSION)
	cd pugixml-$(PUGIXML_VERSION)/src; \
	$(CXX) -fpic -c pugixml.cpp; \
	$(AR) rcs libpugixml.a pugixml.o

include/pugixml.hpp: pugixml-$(PUGIXML_VERSION)
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/pugixml.hpp include/pugixml.hpp
	touch include/pugixml.hpp

include/pugiconfig.hpp: pugixml-$(PUGIXML_VERSION)
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/pugiconfig.hpp include/pugiconfig.hpp
	touch include/pugiconfig.hpp

lib/libpugixml.a: pugixml-$(PUGIXML_VERSION)/src/libpugixml.a
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/libpugixml.a lib/libpugixml.a

pugixml: include lib include/pugixml.hpp include/pugiconfig.hpp lib/libpugixml.a

####################################################

rapidjson-$(RAPIDJSON_VERSION).zip:
	wget http://rapidjson.googlecode.com/files/rapidjson-$(RAPIDJSON_VERSION).zip

rapidjson-$(RAPIDJSON_VERSION): rapidjson-$(RAPIDJSON_VERSION).zip
	unzip -qo rapidjson-$(RAPIDJSON_VERSION).zip
	mv rapidjson rapidjson-$(RAPIDJSON_VERSION)

include/rapidjson: rapidjson-$(RAPIDJSON_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson include/rapidjson
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/rapidjson rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson
	touch include/rapidjson
endif
	
rapidjson: include include/rapidjson

####################################################

smhasher-$(SMHASHER_VERSION):
	#Instead of having to install Subversion to do
	#	svn checkout http://smhasher.googlecode.com/svn/trunk/@$(SMHASHER_VERSION) smhasher-$(SMHASHER_VERSION)
	#Use the github mirror
	git clone git://github.com/haberman/smhasher.git smhasher-$(SMHASHER_VERSION)

include/smhasher: smhasher-$(SMHASHER_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../smhasher-$(SMHASHER_VERSION) include/smhasher
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/smhasher smhasher-$(SMHASHER_VERSION)
	touch include/smhasher
endif

smhasher: include include/smhasher

####################################################

sqlite-amalgamation-$(SQLITE_VERSION).zip:
	wget http://www.sqlite.org/sqlite-amalgamation-$(SQLITE_VERSION).zip;

sqlite-amalgamation-$(SQLITE_VERSION): sqlite-amalgamation-$(SQLITE_VERSION).zip
	unzip -qo sqlite-amalgamation-$(SQLITE_VERSION).zip

sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a: sqlite-amalgamation-$(SQLITE_VERSION)
	cd sqlite-amalgamation-$(SQLITE_VERSION); \
	$(CC) -fpic -c sqlite3.c; \
	$(AR) rcs libsqlite3.a sqlite3.o

include/sqlite3.h: sqlite-amalgamation-$(SQLITE_VERSION)
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/sqlite3.h include/sqlite3.h
	touch include/sqlite3.h

include/sqlite3ext.h: sqlite-amalgamation-$(SQLITE_VERSION)
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/sqlite3ext.h include/sqlite3ext.h
	touch include/sqlite3ext.h

lib/libsqlite3.a: sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a lib/libsqlite3.a

sqlite: include lib include/sqlite3.h include/sqlite3ext.h lib/libsqlite3.a

###################################################
# tidy-html5
# http://w3c.github.com/tidy-html5/
#
# Note that we only "make ../../lib/libtidy.a" and not "make all" because the latter is not required
# On MSYS apply path to prevent linker error asscoiated with "GetFileSizeEx"

tidy-html5.zip:
	wget --no-check-certificate -Otidy-html5.zip  https://github.com/w3c/tidy-html5/archive/master.zip

tidy-html5-master: tidy-html5.zip
	unzip tidy-html5.zip
ifeq ($(STENCILA_PLATFORM), msys)
	patch tidy-html5-master/src/mappedio.c tidy-html5-src-mappedio.c-patch.diff
endif

tidy-html5-master/lib/libtidy.a:
	cd tidy-html5-master/build/gmake && make ../../lib/libtidy.a

include/tidy-html5: tidy-html5-master
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../tidy-html5-master/include include/tidy-html5
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/tidy-html5 tidy-html5-master/include
	touch include/tidy-html5
endif

lib/libtidy-html5.a: tidy-html5-master/lib/libtidy.a
	ln -sf ../tidy-html5-master/lib/libtidy.a lib/libtidy-html5.a

tidy-html5: include include/tidy-html5 lib lib/libtidy-html5.a
