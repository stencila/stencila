include ../shared.make

BOOST_VERSION := 1_60_0

resources/boost_$(BOOST_VERSION).tar.bz2:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ http://prdownloads.sourceforge.net/boost/boost_$(BOOST_VERSION).tar.bz2

build/requires/boost: resources/boost_$(BOOST_VERSION).tar.bz2
	mkdir -p build/requires
	rm -rf build/requires/boost
	tar --bzip2 -xf $< -C build/requires
	mv build/requires/boost_$(BOOST_VERSION) build/requires/boost
	touch $@

# TODO
#   Need to add the building of libboost_python3.a. This gets built if we add the lines
#		# Python configuration
#		using python : 2.6 ;
#		using python : 2.7 ;
#		using python : 3.2 ;
#   to the project-config.jam.
#   Should use context.env.PYTHON_VERSIONS to do this
#   See http://www.boost.org/doc/libs/1_55_0/libs/python/doc/building.html#id34
#   
#   An alternative may to be to not use a project-config.jam and instead use a hand coded user-config.jam
#   based on one that bootstrap.sh produces.

# Boost is configured with:
#   --with-toolset=mingw - for build under MinGW64 shell
#   --with-libraries - so that only those libraries that are needed are built
BOOST_BOOTSTRAP_FLAGS := --with-libraries=filesystem,python,regex,system,test,thread
ifeq ($(OS), win)
	BOOST_BOOTSTRAP_FLAGS += gcc --with-toolset=mingw
endif

# Boost is built with:
#   -d0 		- supress all informational messages (reduces verbosity which is useful on CI servers)
#   --prefix=.  - so that boost installs into its own directory
#   link=static - so that get statically compiled instead of dynamically compiled libraries
BOOST_B2_FLAGS := -d0 --prefix=. link=static install
ifeq ($(OS), win)
	# system" layout of library names and header locations (otherwise it defaults to 'versioned' on Windows)
	# "release" build otherwise defaults to debug AND release, which with "system" causes an 
	#     error (http://boost.2283326.n4.nabble.com/atomic-building-with-layout-system-mingw-bug-7482-td4640920.html)
	BOOST_B2_FLAGS += --layout=system release toolset=gcc
else
	# cxxflags=-fPIC - so that the statically compiled library has position independent code for use in shared libraries
	BOOST_B2_FLAGS += cxxflags=-fPIC
endif

build/requires/boost-built.flag: build/requires/boost
	cd $< ; ./bootstrap.sh gcc $(BOOST_BOOTSTRAP_FLAGS)
ifeq ($(OS), win)
	# Under MinGW64 shell, project-config.jam must be edited to fix [this error](http://stackoverflow.com/a/5244844/1583041) 
	# The spaces are important so that we don't clobber the python setup in this config file
	sed -i "s!mingw !gcc !" $</project-config.jam
	# Under MinGW64 shell, pyconfig.h root can't be found unless we explicitly define include dir
	sed -i "s!using python : 2.7 .*\$$!using python : 2.7 : C:/msys64/mingw64 : C:/msys64/mingw64/include/python2.7 ;!" $</project-config.jam
endif
	cd $< ; ./b2 $(BOOST_B2_FLAGS)
	touch $@

boost: build/requires/boost-built.flag
