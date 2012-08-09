
BOOST_VERSION = 1_50_0
CPP_NETLIB_VERSION = 0.9.4
PUGIXML_VERSION = 1.2
RAPIDJSON_VERSION = 0.1
SMHASHER_VERSION = 147
SQLITE_VERSION = 3071300

all: boost cpp-netlib pugixml rapidjson smhasher sqlite

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

# Boost is made with some options to overide defaults
#  --prefix=.  - so that boost installs into its own directory  (boost_$(BOOST_VERSION))
# link=static - so that get statically compiled instead of dynamically compile libraries
boost_$(BOOST_VERSION)/lib/libboost_system.a: boost_$(BOOST_VERSION)
	cd boost_$(BOOST_VERSION); \
	./bootstrap.sh; \
	./b2 --prefix=. link=static install;
	
# Link to boost include directory and each of the statically compiled libraries
boost: boost_$(BOOST_VERSION)/lib/libboost_system.a include lib
	ln -sfT ../boost_$(BOOST_VERSION)/include/boost include/boost
	for file in $$(ls boost_$(BOOST_VERSION)/lib/*.a); do ln -sf ../$$file lib; done

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
cpp-netlib: cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/libcppnetlib-client-connections.a include lib
	ln -sfT ../cpp-netlib-$(CPP_NETLIB_VERSION)/boost/ include/cpp-netlib
	for file in $$(ls cpp-netlib-$(CPP_NETLIB_VERSION)/libs/network/src/*.a); do ln -sf ../$$file lib; done
	
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
	g++ -fpic -c pugixml.cpp; \
	ar rcs libpugixml.a pugixml.o

pugixml: pugixml-$(PUGIXML_VERSION)/src/libpugixml.a include lib
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/pugixml.hpp include/pugixml.hpp
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/pugiconfig.hpp include/pugiconfig.hpp
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/libpugixml.a lib/libpugixml.a 
	
####################################################

rapidjson-$(RAPIDJSON_VERSION).zip:
	wget http://rapidjson.googlecode.com/files/rapidjson-$(RAPIDJSON_VERSION).zip
	
rapidjson-$(RAPIDJSON_VERSION): rapidjson-$(RAPIDJSON_VERSION).zip
	unzip -qo rapidjson-$(RAPIDJSON_VERSION).zip
	mv rapidjson rapidjson-$(RAPIDJSON_VERSION)

rapidjson: rapidjson-$(RAPIDJSON_VERSION) include
	ln -sfT ../rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson include/rapidjson

####################################################

smhasher-$(SMHASHER_VERSION):
	svn checkout http://smhasher.googlecode.com/svn/trunk/@$(SMHASHER_VERSION) smhasher-$(SMHASHER_VERSION)

smhasher: smhasher-$(SMHASHER_VERSION) include
	ln -sfT ../smhasher-$(SMHASHER_VERSION) include/smhasher
	
####################################################

sqlite-amalgamation-$(SQLITE_VERSION).zip:
	wget http://www.sqlite.org/sqlite-amalgamation-$(SQLITE_VERSION).zip;
	
sqlite-amalgamation-$(SQLITE_VERSION): sqlite-amalgamation-$(SQLITE_VERSION).zip
	unzip -qo sqlite-amalgamation-$(SQLITE_VERSION).zip

sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a: sqlite-amalgamation-$(SQLITE_VERSION)
	cd sqlite-amalgamation-$(SQLITE_VERSION); \
	gcc -fpic -c sqlite3.c; \
	ar rcs libsqlite3.a sqlite3.o; \
	
sqlite: sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a include lib
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/sqlite3.h include/sqlite3.h
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/sqlite3ext.h include/sqlite3ext.h
	ln -sf ../sqlite-amalgamation-$(SQLITE_VERSION)/libsqlite3.a lib/libsqlite3.a

	