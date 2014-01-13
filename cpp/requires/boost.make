# Stencila Makefile for [Boost C++ libraries](http://www.boost.org/)

include $(dir $(lastword $(MAKEFILE_LIST)))/../../variables.make

BOOST_VERSION = 1_55_0

boost_$(BOOST_VERSION).tar.bz2:
	wget http://prdownloads.sourceforge.net/boost/boost_$(BOOST_VERSION).tar.bz2

boost_$(BOOST_VERSION): boost_$(BOOST_VERSION).tar.bz2
	tar --bzip2 -xf boost_$(BOOST_VERSION).tar.bz2
	touch boost_$(BOOST_VERSION)

# Boost is built with some options to override defaults
#  	--prefix=.  - so that boost installs into its own directory  (boost_$(BOOST_VERSION))
#  	cxxflags=-fPIC - so that the statically compiled library has position independent code for use in shared libraries
# 	link=static - so that get statically compiled instead of dynamically compiled libraries
boost_$(BOOST_VERSION)/lib/libboost_system.a: boost_$(BOOST_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	cd boost_$(BOOST_VERSION); \
	./bootstrap.sh; \
	./b2 --prefix=. cxxflags=-fPIC link=static install
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
	./b2 --prefix=. --layout=system release toolset=gcc cxxflags=-fPIC link=static install
endif
	touch boost_$(BOOST_VERSION)/lib/libboost_system.a

include/boost: boost_$(BOOST_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../boost_$(BOOST_VERSION)/include/boost include/boost
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/boost boost_$(BOOST_VERSION)/include/boost
endif
	touch include/boost

# Link to each of the statically compiled libraries
lib/libboost_system.a: boost_$(BOOST_VERSION)/lib/libboost_system.a
	for file in $$(ls boost_$(BOOST_VERSION)/lib/*.a); do ln -sf ../$$file lib; done

boost: include/boost lib/libboost_system.a
