
BOOST_VERSION = 1_50_0
CPP_NETLIB_VERSION = 0.9.4
PUGIXML_VERSION = 1.2
RAPIDJSON_VERSION = 0.1
SMHASHER_VERSION = 147
SQLITE_VERSION = 3071300

# Specify that symlinks time should be checked, not the target file time.
# If this option is not specified then symlinks get remade
# Note also in rules below that the rules for creating symlinks do NOT depend upon
# the 'include' and 'lib' rules : if they do then you get cyclic updating because the symlinks
# change the times of those directories
MAKEFLAGS += --check-symlink-times

all: boost pugixml rapidjson smhasher sqlite

.PHONY: boost cpp-netlib pugixml rapidjson smhasher sqlite

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

# Boost is made with some options to override defaults
#  --prefix=.  - so that boost installs into its own directory  (boost_$(BOOST_VERSION))
# link=static - so that get statically compiled instead of dynamically compile libraries
ifeq ($(OS), linux)
	boost_$(BOOST_VERSION)/lib/libboost_system.a: boost_$(BOOST_VERSION)
		cd boost_$(BOOST_VERSION); \
		./bootstrap.sh; \
		./b2 --prefix=. link=static install;
endif
ifeq ($(OS), win)
	boost_$(BOOST_VERSION)/lib/libboost_system.a: boost_$(BOOST_VERSION)
		cd boost_$(BOOST_VERSION); \
		./bootstrap.sh --with-toolset=mingw; \
		#Fix the [error](http://stackoverflow.com/a/5244844/1583041) in the `project-config.jam` produced by bootstrap
		sed -i "s/mingw/gcc/g" project-config.jam; \
		#Tell boost that you want it to use `x86_64-w64-mingw32-g++` for `gcc`.
		#This has to be done by specifying the full path in `user-config.jam` otherwise `b2` does not seem to be able to find it.
		#(I tried several ways of setting the path to include the relevant bin directories but nothing worked).
		echo 'using gcc : : "C:/MinGW/mingw64/bin/x86_64-w64-mingw32-g++"' > tools/build/v2/user-config.jam ;\
		#Specify "gcc" toolset (as defined above) and "system" layout of library names and header locations (otherwise it defaults to 'versioned' on Windows)
		./b2 --prefix=. --layout=system toolset=gcc link=static install;
endif


include/boost: boost_$(BOOST_VERSION)
	ln -sfT ../boost_$(BOOST_VERSION)/include/boost include/boost

# Link to each of the statically compiled libraries
lib/libboost_system.a: boost_$(BOOST_VERSION)/lib/libboost_system.a
	for file in $$(ls boost_$(BOOST_VERSION)/lib/*.a); do ln -sf ../$$file lib; done

boost: include lib include/boost lib/libboost_system.a

####################################################
# cpp-netlib
#

cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz:
	wget https://github.com/downloads/cpp-netlib/cpp-netlib/cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz

cpp-netlib-$(CPP_NETLIB_VERSION): cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz
	tar xzf cpp-netlib-$(CPP_NETLIB_VERSION).tar.gz

cpp-netlib-$(CPP_NETLIB_VERSION)/Makefile: cpp-netlib-$(CPP_NETLIB_VERSION)
	cd cpp-netlib-$(CPP_NETLIB_VERSION); \
	export BOOST_ROOT=../boost_1_50_0 ; cmake -DCMAKE_BUILD_TYPE=Debug  -DCMAKE_C_COMPILER=gcc -DCMAKE_CXX_COMPILER=g++ .

cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/libcppnetlib-client-connections.a: cpp-netlib-$(CPP_NETLIB_VERSION)/Makefile
	cd cpp-netlib-$(CPP_NETLIB_VERSION); \
	make

# Link to cpp-netlib include directory and each of the statically compiled libraries
# cpp-netlib is going through process of inclusion in boost
# Until cpp-netlib is fully integrated into boost we use "cpp-netlib/network' instead of "boost/network"
include/cpp-netlib: cpp-netlib-$(CPP_NETLIB_VERSION)
	ln -sfT ../cpp-netlib-$(CPP_NETLIB_VERSION)/boost/ include/cpp-netlib

lib/libcppnetlib-client-connections.a: cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/libcppnetlib-client-connections.a
	for file in $$(ls cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/*.a); do ln -sf ../$$file lib; done

cpp-netlib: include lib include/cpp-netlib

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

include/pugiconfig.hpp: pugixml-$(PUGIXML_VERSION)
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/pugiconfig.hpp include/pugiconfig.hpp

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
	ln -sfT ../rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson include/rapidjson

rapidjson: include include/rapidjson

####################################################

smhasher-$(SMHASHER_VERSION):
	#Instead of having to install Subversion to do
	#	svn checkout http://smhasher.googlecode.com/svn/trunk/@$(SMHASHER_VERSION) smhasher-$(SMHASHER_VERSION)
	#Use the github mirror
	git clone git://github.com/haberman/smhasher.git

include/smhasher: smhasher-$(SMHASHER_VERSION)
	ln -sfT ../smhasher-$(SMHASHER_VERSION) include/smhasher

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

include/sqlite3ext.h: sqlite-amalgamation-$(SQLITE_VERSION)
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/sqlite3ext.h include/sqlite3ext.h

lib/libsqlite3.a: sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a lib/libsqlite3.a

sqlite: include lib include/sqlite3.h include/sqlite3ext.h lib/libsqlite3.a
