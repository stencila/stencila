all: cpp-package py-package r-package

# Get root directory for Stencila project
ROOT := $(realpath .)

# Get the operating system  e.g. linux
OS := $(shell ./config.py os)
# Get the machine architecture e.g i386, x86_64
ARCH := $(shell ./config.py arch)
# Get Stencila commit
COMMIT :=  $(shell ./config.py commit)
# Get Stencila version
VERSION :=  $(shell ./config.py version)
# Is this a dirty build (i.e. changes since last commit)?
DIRTY := $(findstring dirty,$(VERSION))

# Build directory uses a heirarchy based on the 
# operating system and machine architecture.
ifndef BUILD
	BUILD := build/$(OS)/$(ARCH)
endif

# Resources directory for downloads of dependencies
# that are independent of build
ifndef RESOURCES
	RESOURCES := build/resources
endif

# A function for notify the Stencila hub that
# a build has been delivered
define DELIVERY_NOTIFY
	curl -u "Token:$$STENCILA_TOKEN" \
	  -X POST -H "Content-Type: application/json" -H "Accept: application/json" -d "{ \
	    \"package\": \"$1\", \
	    \"flavour\": \"$2\", \
	    \"platform\": \"$3\", \
	    \"url\": \"$4\", \
	    \"version\": \"$(VERSION)\", \
	    \"commit\": \"$(COMMIT)\" \
	   }" "https://stenci.la/builds"
endef

# Show important Makefile variables
vars:
	@echo ROOT: $(ROOT)
	@echo OS: $(OS)
	@echo ARCH: $(ARCH)
	@echo COMMIT: $(COMMIT)
	@echo VERSION: $(VERSION)
	@echo DIRTY: $(DIRTY)
	@echo BUILD: $(BUILD)
	@echo RESOURCES: $(RESOURCES)
	@echo CXX: $(CXX)

#################################################################################################
# Symbolic links to current build
# 
# Useful for automatically collecting the latest build products

.PHONY: build/current
build/current:
	@mkdir -p $(BUILD)
	@ln -sfT $(OS)/$(ARCH) build/current
build-current: build/current

# During development symlink the `stencila.js` into the 
# Stencila store so it can be served by the embedded server.
# Call this with STORE variable e.g.
#    make build-serve STORE=../../store
build-serve: build/current
	ln -sfT $(ROOT)/build/current $(STORE)/build

#################################################################################################
# C++ requirements

# Collect necessary include and lib directories and library names
CPP_REQUIRES_INC_DIRS := 
CPP_REQUIRES_LIB_DIRS := 
CPP_REQUIRES_LIBS :=

BOOST_VERSION := 1_60_0

$(RESOURCES)/boost_$(BOOST_VERSION).tar.bz2:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ http://prdownloads.sourceforge.net/boost/boost_$(BOOST_VERSION).tar.bz2

$(BUILD)/cpp/requires/boost: $(RESOURCES)/boost_$(BOOST_VERSION).tar.bz2
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $(BUILD)/cpp/requires/boost
	tar --bzip2 -xf $< -C $(BUILD)/cpp/requires
	mv $(BUILD)/cpp/requires/boost_$(BOOST_VERSION) $(BUILD)/cpp/requires/boost
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
#   --with-libraries - so that only those libraries that are needed are built
BOOST_BOOTSTRAP_FLAGS := --with-libraries=atomic,chrono,date_time,filesystem,program_options,python,regex,system,test,timer,thread
ifeq ($(OS), win)
	# bootstrap.sh must be called with mingw specified as toolset otherwise errors occur
	BOOST_BOOTSTRAP_FLAGS += --with-toolset=mingw
endif

# Boost is built with:
#   --d0 		- supress all informational messages (reduces verbosity which is useful on CI servers)
#   --prefix=.  - so that boost installs into its own directory
#   link=static - so that get statically compiled instead of dynamically compiled libraries
BOOST_B2_FLAGS := -d0 --prefix=. link=static install
ifeq ($(OS), linux)
	# cxxflags=-fPIC - so that the statically compiled library has position independent code for use in shared libraries
	BOOST_B2_FLAGS += cxxflags=-fPIC
endif
ifeq ($(OS), win)
	# b2 must be called with "system" layout of library names and header locations (otherwise it defaults to 'versioned' on Windows)
	# b2 must be called with "release" build otherwise defaults to debug AND release, which with "system" causes an 
	#   error (http://boost.2283326.n4.nabble.com/atomic-building-with-layout-system-mingw-bug-7482-td4640920.html)
	BOOST_B2_FLAGS += --layout=system release toolset=gcc
endif

$(BUILD)/cpp/requires/boost-built.flag: $(BUILD)/cpp/requires/boost
	cd $< ; ./bootstrap.sh $(BOOST_BOOTSTRAP_FLAGS)
ifeq ($(OS), win)
	# Under MSYS, project-config.jam must be edited to fix [this error](http://stackoverflow.com/a/5244844/1583041) 
	# The spaces are important so that we don't clobber the python setup in this config file
	sed -i "s!mingw !gcc !" $</project-config.jam
endif
	cd $< ; ./b2 $(BOOST_B2_FLAGS)
	touch $@

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/boost/include
CPP_REQUIRES_LIB_DIRS += -L$(BUILD)/cpp/requires/boost/lib
CPP_REQUIRES_LIBS += boost_filesystem boost_system boost_regex boost_thread 

cpp-requires-boost: $(BUILD)/cpp/requires/boost-built.flag


LIBGIT2_VERSION := 0.23.4

$(RESOURCES)/libgit2-$(LIBGIT2_VERSION).zip:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/libgit2/libgit2/archive/v$(LIBGIT2_VERSION).zip

$(BUILD)/cpp/requires/libgit2: $(RESOURCES)/libgit2-$(LIBGIT2_VERSION).zip
	mkdir -p $(BUILD)/cpp/requires
	unzip -qo $< -d $(BUILD)/cpp/requires
	rm -rf $@
	mv $(BUILD)/cpp/requires/libgit2-$(LIBGIT2_VERSION) $@
	touch $@

# For build options see https://libgit2.github.com/docs/guides/build-and-link/
#  	BUILD_CLAR=OFF - do not build tests
#  	BUILD_SHARED_LIBS=OFF - do not build shared library
LIBGIT2_CMAKE_FLAGS := -DBUILD_CLAR=OFF -DBUILD_SHARED_LIBS=OFF
ifeq ($(OS), linux)
	LIBGIT2_CMAKE_FLAGS += -DCMAKE_C_FLAGS=-fPIC
endif
ifeq ($(OS), win)
	LIBGIT2_CMAKE_FLAGS += -G "MSYS Makefiles"
endif
$(BUILD)/cpp/requires/libgit2-built.flag: $(BUILD)/cpp/requires/libgit2
	cd $< ;\
	  mkdir -p build ;\
	  cd build ;\
	  cmake .. $(LIBGIT2_CMAKE_FLAGS);\
	  cmake --build .
	touch $@

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/libgit2/include
CPP_REQUIRES_LIB_DIRS += -L$(BUILD)/cpp/requires/libgit2/build
CPP_REQUIRES_LIBS += git2

cpp-requires-libgit2: $(BUILD)/cpp/requires/libgit2-built.flag



LIBZIP_VERSION := 1.1.2

$(RESOURCES)/libzip-$(LIBZIP_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget -O $@ http://www.nih.at/libzip/libzip-$(LIBZIP_VERSION).tar.gz

$(BUILD)/cpp/requires/libzip: $(RESOURCES)/libzip-$(LIBZIP_VERSION).tar.gz
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $@
	tar xzf $< -C $(BUILD)/cpp/requires
	mv $(BUILD)/cpp/requires/libzip-$(LIBZIP_VERSION) $@
	touch $@

$(BUILD)/cpp/requires/libzip/lib/.libs/libzip.a: $(BUILD)/cpp/requires/libzip
	cd $<  && ./configure --disable-shared --enable-static --with-pic && make

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/libzip/lib
CPP_REQUIRES_LIB_DIRS += -L$(BUILD)/cpp/requires/libzip/lib/.libs
CPP_REQUIRES_LIBS += zip

cpp-requires-libzip: $(BUILD)/cpp/requires/libzip/lib/.libs/libzip.a



CPP_NETLIB_VERSION := 0.11.2

$(RESOURCES)/cpp-netlib-$(CPP_NETLIB_VERSION)-final.tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/cpp-netlib/cpp-netlib/archive/cpp-netlib-$(CPP_NETLIB_VERSION)-final.tar.gz
	
$(BUILD)/cpp/requires/cpp-netlib: $(RESOURCES)/cpp-netlib-$(CPP_NETLIB_VERSION)-final.tar.gz
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $@
	tar xzf $< -C $(BUILD)/cpp/requires
	mv $(BUILD)/cpp/requires/cpp-netlib-cpp-netlib-$(CPP_NETLIB_VERSION)-final $(BUILD)/cpp/requires/cpp-netlib
	touch $@

# cpp-netlib needs to be compiled with OPENSSL_NO_SSL2 defined because SSL2 is insecure and depreciated and on
# some systems (e.g. Ubuntu) OpenSSL is compiled with no support for it
CPP_NETLIB_CMAKE_FLAGS := -DCMAKE_BUILD_TYPE=Debug  -DCMAKE_C_COMPILER=gcc -DCMAKE_CXX_COMPILER=g++ -DCMAKE_CXX_FLAGS="-DOPENSSL_NO_SSL2 -O2 -fPIC"
# Under MSYS some additional CMake flags need to be specified
# The "-I/usr/include" in -DCMAKE_CXX_FLAGS seems uncessary but it's not
ifeq ($(OS), win)
CPP_NETLIB_CMAKE_FLAGS += -G "MSYS Makefiles" -DOPENSSL_INCLUDE_DIR=/usr/include/ -DOPENSSL_LIBRARIES=/usr/lib/ -DCMAKE_CXX_FLAGS="-DOPENSSL_NO_SSL2 -O2 -fPIC -I/usr/include"
endif
$(BUILD)/cpp/requires/cpp-netlib/libs/network/src/libcppnetlib-client-connections.a: $(BUILD)/cpp/requires/cpp-netlib
	cd $(BUILD)/cpp/requires/cpp-netlib; \
		export BOOST_ROOT=../boost ; \
		cmake $(CPP_NETLIB_CMAKE_FLAGS); \
		make cppnetlib-client-connections cppnetlib-server-parsers cppnetlib-uri

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/cpp-netlib/
CPP_REQUIRES_LIB_DIRS += -L$(BUILD)/cpp/requires/cpp-netlib/libs/network/src
CPP_REQUIRES_LIBS += cppnetlib-client-connections cppnetlib-uri

cpp-requires-cpp-netlib: $(BUILD)/cpp/requires/cpp-netlib/libs/network/src/libcppnetlib-client-connections.a


PUGIXML_VERSION := 1.7

$(RESOURCES)/pugixml-$(PUGIXML_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/zeux/pugixml/archive/v$(PUGIXML_VERSION).tar.gz

$(BUILD)/cpp/requires/pugixml: $(RESOURCES)/pugixml-$(PUGIXML_VERSION).tar.gz
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $@
	tar xzf $< -C $(BUILD)/cpp/requires
	mv $(BUILD)/cpp/requires/pugixml-$(PUGIXML_VERSION) $(BUILD)/cpp/requires/pugixml
	touch $@

PUGIXML_CXX_FLAGS := -O2
ifeq ($(OS), linux)
	PUGIXML_CXX_FLAGS += -fPIC
endif
$(BUILD)/cpp/requires/pugixml/src/libpugixml.a: $(BUILD)/cpp/requires/pugixml
	cd $</src ;\
	  $(CXX) $(PUGIXML_CXX_FLAGS) -c pugixml.cpp ;\
	  $(AR) rcs libpugixml.a pugixml.o

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/pugixml/src
CPP_REQUIRES_LIB_DIRS += -L$(BUILD)/cpp/requires/pugixml/src
CPP_REQUIRES_LIBS += pugixml

cpp-requires-pugixml: $(BUILD)/cpp/requires/pugixml/src/libpugixml.a


JSONCPP_VERSION := 1.6.5

$(RESOURCES)/jsoncpp-$(JSONCPP_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/open-source-parsers/jsoncpp/archive/$(JSONCPP_VERSION).tar.gz

$(BUILD)/cpp/requires/jsoncpp/dist: $(RESOURCES)/jsoncpp-$(JSONCPP_VERSION).tar.gz
	mkdir -p $(BUILD)/cpp/requires
	tar xzf $< -C $(BUILD)/cpp/requires
	cd $(BUILD)/cpp/requires/ ;\
		rm -rf jsoncpp ;\
		mv -f jsoncpp-$(JSONCPP_VERSION) jsoncpp ;\
		cd jsoncpp ;\
			python amalgamate.py ;
	touch $@

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/jsoncpp/dist

cpp-requires-jsoncpp: $(BUILD)/cpp/requires/jsoncpp/dist


TIDYHTML5_VERSION := 5.1.25

$(RESOURCES)/tidy-html5-$(TIDYHTML5_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/htacg/tidy-html5/archive/$(TIDYHTML5_VERSION).tar.gz

$(BUILD)/cpp/requires/tidy-html5: $(RESOURCES)/tidy-html5-$(TIDYHTML5_VERSION).tar.gz
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $@
	tar xzf $< -C $(BUILD)/cpp/requires
	mv $(BUILD)/cpp/requires/tidy-html5-$(TIDYHTML5_VERSION) $(BUILD)/cpp/requires/tidy-html5
	touch $@

TIDYHTML5_CMAKE_FLAGS = -DCMAKE_C_FLAGS="-O2 -fPIC"
ifeq ($(OS), win)
TIDYHTML5_CMAKE_FLAGS += -G "MSYS Makefiles"
endif
$(BUILD)/cpp/requires/tidy-html5-built.flag: $(BUILD)/cpp/requires/tidy-html5
	cd $(BUILD)/cpp/requires/tidy-html5/build/cmake ;\
	  cmake $(TIDYHTML5_CMAKE_FLAGS) ../..
ifeq ($(OS), linux)
	cd $(BUILD)/cpp/requires/tidy-html5/build/cmake ;\
		make
endif
ifeq ($(OS), win)
	cd $(BUILD)/cpp/requires/tidy-html5/build/cmake ;\
		cmake --build . --config Release
	# Under MSYS2 there are lots of multiple definition errors for localize symbols in the library
	objcopy --localize-symbols=cpp/requires/tidy-html5-localize-symbols.txt $(BUILD)/cpp/requires/tidy-html5/build/cmake/libtidys.a
endif
	touch $@

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/tidy-html5/include
CPP_REQUIRES_LIB_DIRS += -L$(BUILD)/cpp/requires/tidy-html5/build/cmake
CPP_REQUIRES_LIBS += tidys

cpp-requires-tidy-html5: $(BUILD)/cpp/requires/tidy-html5-built.flag


WEBSOCKETPP_VERSION := 0.7.0

$(RESOURCES)/websocketpp-$(WEBSOCKETPP_VERSION).zip:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/zaphoyd/websocketpp/archive/$(WEBSOCKETPP_VERSION).zip

$(BUILD)/cpp/requires/websocketpp-built.flag: $(RESOURCES)/websocketpp-$(WEBSOCKETPP_VERSION).zip
	rm -rf $(BUILD)/cpp/requires/websocketpp
	unzip -qo $< -d $(BUILD)/cpp/requires
	cd $(BUILD)/cpp/requires ;\
	  mv websocketpp-$(WEBSOCKETPP_VERSION) websocketpp ;\
	  touch websocketpp
	touch $@

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/websocketpp

cpp-requires-websocketpp: $(BUILD)/cpp/requires/websocketpp-built.flag


$(BUILD)/cpp/requires: \
	cpp-requires-boost \
	cpp-requires-cpp-netlib \
	cpp-requires-libgit2 \
	cpp-requires-libzip \
	cpp-requires-pugixml \
	cpp-requires-jsoncpp \
	cpp-requires-tidy-html5 \
	cpp-requires-websocketpp

cpp-requires: $(BUILD)/cpp/requires

# List of other libraries required. These are not included `libstencila.a`
CPP_OTHER_LIBS := z crypto ssl
ifeq ($(OS), linux)
	CPP_OTHER_LIBS += rt pthread curl
endif
ifeq ($(OS), win)
	CPP_OTHER_LIBS += ws2_32 mswsock ssh2
endif

#################################################################################################
# C++ helpers
# These helpers are currently used by the C++ module via system calls. As such they are not required
# to compile Stencila modules but rather provide additional functionality. In the long term the
# system calls to these helpers will be replaced by integrating C++ compatible libraries or replacement code

# PhantomJS is used in `stencil-formats.cpp` for translating ASCIIMath to MathML and for
# creating thumbnails.
# Instead of using PhantomJS, the translation from ASCIIMath to MathML could be done by porting the ASCIIMath.js code to C++
cpp-helpers-phantomjs:
	cd /usr/local/share ;\
		sudo wget https://bitbucket.org/ariya/phantomjs/downloads/phantomjs-1.9.8-linux-x86_64.tar.bz2 ;\
		sudo tar xjf phantomjs-1.9.8-linux-x86_64.tar.bz2 ;\
		sudo ln -s /usr/local/share/phantomjs-1.9.8-linux-x86_64/bin/phantomjs /usr/local/share/phantomjs ;\
		sudo ln -s /usr/local/share/phantomjs-1.9.8-linux-x86_64/bin/phantomjs /usr/local/bin/phantomjs ;\

# Sass is used for `make`ing themes (compiling SCSS into minified CSS)
# Instead of using node-sass, libsass could be used in C++ directly
cpp-helpers-sass:
	sudo npm install node-sass -g

# UglifyJS is used for `make`ing themes (compiling JS into minified JS)
cpp-helpers-uglifyjs:
	sudo npm install uglify-js -g

#################################################################################################
# Stencila C++ library

# Get version compiled into library
CPP_VERSION_CPP := $(BUILD)/cpp/library/version.cpp
CPP_VERSION_O := $(BUILD)/cpp/library/version.o
CPP_VERSION_COMPILED := $(shell grep -s -Po "(?<=Stencila::version = \")([^\"]+)" $(CPP_VERSION_CPP))

# Delete version.cpp if it is out of date
ifneq ($(CPP_VERSION_COMPILED),$(VERSION))
DUMMY := $(shell rm -f $(CPP_VERSION_CPP))
endif

# Create version.cpp file with current version and commit
$(CPP_VERSION_CPP):
	@mkdir -p $(dir $@)
	@echo "#include <stencila/version.hpp>" > $(CPP_VERSION_CPP)
	@echo "const std::string Stencila::version = \"$(VERSION)\";" >> $(CPP_VERSION_CPP)
	@echo "const std::string Stencila::commit = \"$(COMMIT)\";" >> $(CPP_VERSION_CPP)

# Compile version object file
$(CPP_VERSION_O): $(CPP_VERSION_CPP)
	@mkdir -p $(dir $@)
	$(CXX) $(CPP_LIBRARY_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -o$@ -c $<
cpp-library-version: $(CPP_VERSION_O)

cpp-library-vars:
	@echo VERSION: $(VERSION)
	@echo COMMIT: $(COMMIT)
	@echo CPP_VERSION_COMPILED: $(CPP_VERSION_COMPILED)


CPP_LIBRARY_FLAGS := --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2
ifeq ($(OS), linux)
	CPP_LIBRARY_FLAGS +=-fPIC
endif

# General object file builds
$(BUILD)/cpp/library/objects/stencila-%.o: cpp/stencila/%.cpp
	@mkdir -p $(BUILD)/cpp/library/objects
	$(CXX) $(CPP_LIBRARY_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -o$@ -c $<

# Various pattern rules for library object files...

# Generate syntax parser using Lemon
$(BUILD)/cpp/library/generated/syntax-%-parser.cpp: cpp/stencila/syntax-%.y
	@mkdir -p $(dir $@)
	lemon $<
	mv cpp/stencila/syntax-$*.h $(dir $@)
	mv cpp/stencila/syntax-$*.c $@
	rm cpp/stencila/syntax-$*.out

# Compile syntax parser
$(BUILD)/cpp/library/objects/stencila-syntax-%-parser.o: $(BUILD)/cpp/library/generated/syntax-%-parser.cpp
	@mkdir -p $(dir $@)
	$(CXX) $(CPP_LIBRARY_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -I$(BUILD)/cpp/library/generated -Wno-unused-variable -o$@ -c $<	

# Generate syntax lexer using Flex
$(BUILD)/cpp/library/generated/syntax-%-lexer.cpp: cpp/stencila/syntax-%.l
	@mkdir -p $(dir $@)
	flex --outfile $@ --header-file=$(dir $@)syntax-$*-lexer.hpp $<

# Compile syntax lexer
$(BUILD)/cpp/library/objects/stencila-syntax-%-lexer.o: $(BUILD)/cpp/library/generated/syntax-%-lexer.cpp $(BUILD)/cpp/library/generated/syntax-%-parser.cpp
	@mkdir -p $(dir $@)
	$(CXX) $(CPP_LIBRARY_FLAGS) -Icpp -I$(BUILD)/cpp/library/generated -o$@ -c $<	

# Generate the `parse()` method by using the template .cxx file
$(BUILD)/cpp/library/generated/syntax-%-parse.cpp: cpp/stencila/syntax-parser-parse.cxx
	@mkdir -p $(dir $@)
	sed -e 's!{lang}!$*!' -e 's!{lang-title}!\u$*!' cpp/stencila/syntax-parser-parse.cxx > $@

# Compile lexer and parser source
$(BUILD)/cpp/library/objects/stencila-syntax-%-parse.o: $(BUILD)/cpp/library/generated/syntax-%-parse.cpp \
														$(BUILD)/cpp/library/objects/stencila-syntax-%-lexer.o \
	 		  											$(BUILD)/cpp/library/objects/stencila-syntax-%-parser.o
	$(CXX) $(CPP_LIBRARY_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -I$(BUILD)/cpp/library/generated -o$@ -c $<

# List of parsing related object files used in library and tests
CPP_PARSER_YS := $(notdir $(wildcard cpp/stencila/syntax-*.y))
CPP_PARSER_OS := $(patsubst syntax-%.y, $(BUILD)/cpp/library/objects/stencila-syntax-%-lexer.o,$(CPP_PARSER_YS)) \
				$(patsubst syntax-%.y, $(BUILD)/cpp/library/objects/stencila-syntax-%-parser.o,$(CPP_PARSER_YS)) \
				$(patsubst syntax-%.y, $(BUILD)/cpp/library/objects/stencila-syntax-%-parse.o,$(CPP_PARSER_YS))

# List of all Stencila library object files
CPP_LIBRARY_OS := $(CPP_PARSER_OS) \
					   $(patsubst %.cpp,$(BUILD)/cpp/library/objects/stencila-%.o,$(notdir $(wildcard cpp/stencila/*.cpp))) \
					   $(CPP_VERSION_O)
cpp-library-objects: $(CPP_LIBRARY_OS)

# Extract object files from requirement libraries
# Care may be required to ensure no name clashes in object files
define CPP_LIBRARY_EXTRACT
	mkdir -p $(BUILD)/cpp/library/objects/$2
	cd $(BUILD)/cpp/library/objects/$2 ;\
		ar x $(realpath $(BUILD)/cpp/requires)/$1 ;\
		for filename in *.o*; do mv $$filename ../$2-$$filename; done ;
	rm -rf $(BUILD)/cpp/library/objects/$2
endef

$(BUILD)/cpp/library/objects/requires-objects.flag: $(BUILD)/cpp/requires
	$(call CPP_LIBRARY_EXTRACT,boost/lib/libboost_system.a,boost-system)
	$(call CPP_LIBRARY_EXTRACT,boost/lib/libboost_filesystem.a,boost-filesystem)
	$(call CPP_LIBRARY_EXTRACT,boost/lib/libboost_regex.a,boost-regex)
	$(call CPP_LIBRARY_EXTRACT,boost/lib/libboost_thread.a,boost-thread)
	$(call CPP_LIBRARY_EXTRACT,cpp-netlib/libs/network/src/libcppnetlib-client-connections.a,cppnetlib-client-connections)
	$(call CPP_LIBRARY_EXTRACT,cpp-netlib/libs/network/src/libcppnetlib-uri.a,cppnetlib-uri)
	$(call CPP_LIBRARY_EXTRACT,libgit2/build/libgit2.a,git2)
	$(call CPP_LIBRARY_EXTRACT,libzip/lib/.libs/libzip.a,zip)
	$(call CPP_LIBRARY_EXTRACT,pugixml/src/libpugixml.a,pugixml)
	$(call CPP_LIBRARY_EXTRACT,tidy-html5/build/cmake/libtidys.a,tidy)
	touch $@
cpp-requires-objects: $(BUILD)/cpp/library/objects/requires-objects.flag

# Archive all object files (Stencila .cpp files and those extracted from requirements libraries)
# into a single static library.
# Output list of contents to `files.txt` and `symbols.txt` for checking
$(BUILD)/cpp/library/libstencila.a: cpp-library-objects cpp-requires-objects
	cd $(BUILD)/cpp/library ;\
		$(AR) rc libstencila.a `find . -name "*.o"` ;\
		$(AR) t libstencila.a > files.txt ;\
		nm -gC libstencila.a > symbols.txt
cpp-library-staticlib: $(BUILD)/cpp/library/libstencila.a

cpp-library: cpp-library-staticlib

cpp-library-clean:
	rm -rf $(BUILD)/cpp/library

#################################################################################################
# Stencila C++ package
CPP_PACKAGE := stencila-$(OS)-$(ARCH)-$(VERSION).tar.gz
CPP_PACKAGE_BUILD := $(BUILD)/cpp/package/$(CPP_PACKAGE)

# Copy over Stencila header files
CPP_STENCILA_HPPS := $(wildcard cpp/stencila/*.hpp)
CPP_PACKAGE_HPPS := $(patsubst %.hpp,$(BUILD)/cpp/package/stencila/stencila/%.hpp,$(notdir $(CPP_STENCILA_HPPS)))
$(BUILD)/cpp/package/stencila/stencila/%.hpp: cpp/stencila/%.hpp
	@mkdir -p $(BUILD)/cpp/package/stencila/stencila
	cp $< $@

# Zip it up
$(CPP_PACKAGE_BUILD): $(CPP_PACKAGE_HPPS) $(BUILD)/cpp/library/libstencila.a
	cp $(BUILD)/cpp/library/libstencila.a $(BUILD)/cpp/package/stencila
	cd $(BUILD)/cpp/package ; tar czf stencila-$(OS)-$(ARCH)-$(VERSION).tar.gz stencila
cpp-package: $(CPP_PACKAGE_BUILD)

# Deliver C++ package to get.stenci.la
cpp-deliver: $(CPP_PACKAGE_BUILD)
ifeq (dirty,$(DIRTY))
	$(error Delivery is not done for dirty versions: $(VERSION). Commit or stash and try again.)
else
	aws s3 cp $(CPP_PACKAGE_BUILD) s3://get.stenci.la/cpp/$(CPP_PACKAGE) --cache-control max-age=31536000
endif


#################################################################################################
# Stencila C++ tests

# Compile options for tests include:
# 		-g (debug symbols)
# 		-O0 (no optimizations, so coverage is valid)
# 		--coverage (for coverage instrumentation)
CPP_TEST_COMPILE := $(CXX) --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function \
                       -g -O0 --coverage -Icpp $(CPP_REQUIRES_INC_DIRS)

CPP_TEST_LIB_DIRS := $(CPP_REQUIRES_LIB_DIRS)

CPP_TEST_LIBS := $(CPP_REQUIRES_LIBS) $(CPP_OTHER_LIBS) boost_unit_test_framework boost_timer boost_chrono gcov
CPP_TEST_LIBS := $(patsubst %, -l%,$(CPP_TEST_LIBS))

# Compile a test file into an object file
# $(realpath $<) is used for consistency of paths in coverage reports
CPP_TEST_OS := $(patsubst %.cpp,$(BUILD)/cpp/tests/%.o,$(notdir $(wildcard cpp/tests/*.cpp)))
$(BUILD)/cpp/tests/%.o: cpp/tests/%.cpp
	@mkdir -p $(BUILD)/cpp/tests
	$(CPP_TEST_COMPILE) -o$@ -c $(realpath $<)

# Compile a stencila source file into an object file
# This needs to be done (instead of linking to libstencila.a) so that coverage statistics
# can be generated for these files
# $(realpath $<) is used for consistency of paths in coverage reports
CPP_TEST_STENCILA_OS := $(CPP_PARSER_OS) \
						$(patsubst %.cpp,$(BUILD)/cpp/tests/stencila/%.o,$(notdir $(wildcard cpp/stencila/*.cpp))) \
						$(CPP_VERSION_O)
$(BUILD)/cpp/tests/stencila/%.o: cpp/stencila/%.cpp
	@mkdir -p $(BUILD)/cpp/tests/stencila
	$(CPP_TEST_COMPILE) -o$@ -c $(realpath $<)

# Input files (typically text files) used for tests
CPP_TEST_INPUTS := $(BUILD)/cpp/tests/stencil-cila-html.txt \
				   $(BUILD)/cpp/tests/stencil-cila-render.cila \
				   $(BUILD)/cpp/tests/html-doc-1.html
$(BUILD)/cpp/tests/%: cpp/tests/%
	cp -f $< $@

# Compile a single test file into an executable
$(BUILD)/cpp/tests/%.exe: $(BUILD)/cpp/tests/%.o $(BUILD)/cpp/tests/tests.o $(CPP_TEST_STENCILA_OS)
	$(CPP_TEST_COMPILE) -o$@ $^ $(CPP_TEST_LIB_DIRS) $(CPP_TEST_LIBS)

# Compile all test files into an executable
$(BUILD)/cpp/tests/tests.exe: $(CPP_TEST_OS) $(CPP_TEST_STENCILA_OS)
	$(CPP_TEST_COMPILE) -o$@ $^ $(CPP_TEST_LIB_DIRS) $(CPP_TEST_LIBS)

# Make test executable precious so they are kept despite
# being intermediaries for test runs
.PRECIOUS: $(BUILD)/cpp/tests/%.exe

# Run a test
# Limit memory to prevent bugs like infinite recursion from filling up the
# machine's memory. This needs to be quite high for some tests. 2Gb = 2,097,152 kb
$(BUILD)/cpp/tests/%: $(BUILD)/cpp/tests/%.exe $(CPP_TEST_INPUTS)
	cd $(BUILD)/cpp/tests/ ;\
		ulimit -v 2097152; (./$(notdir $<)) || (exit 1)

# Run a single test suite by specifying in command line e.g.
# 	make cpp-test-stencil-cila
# Creates a symlink so the debugger picks this test as the one
# to debug
cpp-test-%: $(BUILD)/cpp/tests/%.exe $(CPP_TEST_INPUTS)
	cd $(BUILD)/cpp/tests/ ;\
		ln -sfT $*.exe test-to-debug ;\
		ulimit -v 2097152 ;\
		(./$*.exe) || (exit 1)

# Run quick tests only
cpp-tests-quick: $(BUILD)/cpp/tests/tests.exe $(CPP_TEST_INPUTS)
	cd $(BUILD)/cpp/tests/ ;\
		ulimit -v 2097152; (./tests.exe --run_test=*_quick/*) || (exit 1)

# Run all tests
cpp-tests: $(BUILD)/cpp/tests/tests

# Run all tests and report results and coverage to XML files
# Requires python, xsltproc and [gcovr](http://gcovr.com/guide.html):
#   sudo apt-get install xsltproc
#   sudo pip install gcovr
# Use of 
#   gcovr --root $(ROOT) --filter='.*/cpp/stencila/.*'
# below seems to be necessary when there are different source and build directories to
# only produce coverage reports for files in 'cpp/stencila' 

# Run all tests and generate coverage stats
cpp-tests-coverage: $(BUILD)/cpp/tests/tests.exe
	cd $(BUILD)/cpp/tests ;\
	  # Run all tests \
	  ./tests.exe;\
	  # Produce coverage stats using gcovr helper for gcov \
	  gcovr --root $(ROOT) --filter='.*/cpp/stencila/.*'

# Run all tests and report results to Junit compatible XML files and coverage X
# to Cobertura comparible XML files
$(BUILD)/cpp/tests/boost-test-to-junit.xsl: cpp/tests/boost-test-to-junit.xsl
	cp $< $@
cpp-tests-xml: $(BUILD)/cpp/tests/tests.exe $(BUILD)/cpp/tests/boost-test-to-junit.xsl
	cd $(BUILD)/cpp/tests ;\
	  # Run all tests with reporting to XML file \
	  ./tests.exe --report_format=xml --report_level=detailed --log_format=xml --log_level=test_suite > boost-test-out.xml 2>&1 ;\
	  # Because redirecting stdout and stderr to one file need to wrap in an outer tag \
	  python -c "print '<xml>',file('boost-test-out.xml').read(),'</xml>'" > boost-test.xml ;\
	  # Convert to Junit XML format \
	  xsltproc --output junit.xml boost-test-to-junit.xsl boost-test.xml ;\
	  # Produce coverage report \
	  gcovr --root $(ROOT) --filter='.*/cpp/stencila/.*' --xml --output=coverage.xml

# Run all tests and create coverage to HTML files
# Useful for examining coverage during local development 
cpp-tests-html: $(BUILD)/cpp/tests/tests.exe
	cd $(BUILD)/cpp/tests ;\
	  # Run all tests \
	  ./tests.exe;\
	  # Produce coverage report \
	  gcovr --root $(ROOT) --filter='.*/cpp/stencila/.*' --html --html-details --output=coverage.html

cpp-tests-clean:
	rm -rf $(BUILD)/cpp/tests


#################################################################################################
# C++ documentation

$(BUILD)/cpp/docs/Doxyfile: cpp/docs/Doxyfile
	@mkdir -p $(BUILD)/cpp/docs
	cp $< $@

$(BUILD)/cpp/docs/%.css: cpp/docs/%.css
	@mkdir -p $(BUILD)/cpp/docs
	cp $< $@

$(BUILD)/cpp/docs/%.html: cpp/docs/%.html
	@mkdir -p $(BUILD)/cpp/docs
	cp $< $@

cpp-docs: $(BUILD)/cpp/docs/Doxyfile $(BUILD)/cpp/docs/doxy.css \
	      $(BUILD)/cpp/docs/doxy-header.html $(BUILD)/cpp/docs/doxy-footer.html
	cd $(BUILD)/cpp/docs ;\
	  sed -i 's!PROJECT_NUMBER = .*$$!PROJECT_NUMBER = $(VERSION)!' Doxyfile ;\
	  sed -i 's!INPUT = .*$$!INPUT = $(ROOT)/cpp/stencila/ $(ROOT)/cpp/README.md!' Doxyfile ;\
	  sed -i 's!USE_MDFILE_AS_MAINPAGE = .*$$!USE_MDFILE_AS_MAINPAGE = $(ROOT)/cpp/README.md!' Doxyfile ;\
	  doxygen Doxyfile

# Requires a branch called "gh-pages":
#	git checkout --orphan gh-pages
#	git rm -rf .
# and the "ghp-import" script
# 	sudo pip install ghp-import
cpp-docs-publish: cpp-docs
	mkdir -p $(BUILD)/pages/cpp
	cp -fr $(BUILD)/cpp/docs/html/. $(BUILD)/pages/cpp
	ghp-import -m "Updated pages" -p $(BUILD)/pages

# Remove everything except C++ requirements
cpp-scrub:
	rm -rf $(BUILD)/cpp/library $(BUILD)/cpp/tests $(BUILD)/cpp/docs

# Remove everything
cpp-clean:
	rm -rf $(BUILD)/cpp

#################################################################################################
# Stencila Docker images
#
# When doing `docker push` note that it's necessary to push both version and latest tags:
#   http://container-solutions.com/docker-latest-confusion/
#   https://github.com/docker/docker/issues/7336

# R
$(BUILD)/docker/ubuntu-14.04-r-3.2/image.txt: docker/ubuntu-14.04-r-3.2/Dockerfile docker/stencila-session.r r-package
	@mkdir -p $(dir $@)
	cp docker/ubuntu-14.04-r-3.2/Dockerfile $(dir $@)
	cp docker/stencila-session.r $(dir $@)
	cp $(BUILD)/r/3.2/stencila_$(VERSION).tar.gz $(dir $@)/stencila.tar.gz
	docker build --tag stencila/ubuntu-14.04-r-3.2:$(VERSION) $(dir $@)
	docker tag --force stencila/ubuntu-14.04-r-3.2:$(VERSION) stencila/ubuntu-14.04-r-3.2:latest
	echo "stencila/ubuntu-14.04-r-3.2:$(VERSION)" > $@

docker-r-build: $(BUILD)/docker/ubuntu-14.04-r-3.2/image.txt

docker-r-deliver: docker-r-build
	docker push stencila/ubuntu-14.04-r-3.2:$(VERSION)
	docker push stencila/ubuntu-14.04-r-3.2:latest
	$(call DELIVERY_NOTIFY,docker,ubuntu-14.04-r-3.2)


# Python
$(BUILD)/docker/ubuntu-14.04-py-2.7/image.txt: docker/ubuntu-14.04-py-2.7/Dockerfile docker/stencila-session.py py-package
	@mkdir -p $(dir $@)
	cp docker/ubuntu-14.04-py-2.7/Dockerfile $(dir $@)
	cp docker/stencila-session.py $(dir $@)
	cp $(BUILD)/py/2.7/dist/$$(cat $(BUILD)/py/2.7/latest.txt) $(dir $@)/stencila.whl
	docker build --tag stencila/ubuntu-14.04-py-2.7:$(VERSION) $(dir $@)
	docker tag --force stencila/ubuntu-14.04-py-2.7:$(VERSION) stencila/ubuntu-14.04-py-2.7:latest
	echo "stencila/ubuntu-14.04-py-2.7:$(VERSION)" > $@

docker-py-build: $(BUILD)/docker/ubuntu-14.04-py-2.7/image.txt

docker-py-deliver: docker-py-build
	docker push stencila/ubuntu-14.04-py-2.7:$(VERSION)
	docker push stencila/ubuntu-14.04-py-2.7:latest
	$(call DELIVERY_NOTIFY,docker,ubuntu-14.04-py-2.7)


#################################################################################################
# Stencila Javascript package

REQUIREJS_VERSION := 2.1.17

$(RESOURCES)/require-$(REQUIREJS_VERSION).js:
	@mkdir -p $(RESOURCES)
	wget -O$@ http://requirejs.org/docs/release/$(REQUIREJS_VERSION)/comments/require.js

REQUIREJS_TEXT_VERSION := 2.0.14

$(RESOURCES)/require-text-$(REQUIREJS_TEXT_VERSION).tar.gz:
	@mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ https://github.com/requirejs/text/archive/$(REQUIREJS_TEXT_VERSION).tar.gz

# Make the text.js plugin "inlineable"
$(BUILD)/js/requires/text-$(REQUIREJS_TEXT_VERSION)/text.js: $(RESOURCES)/require-text-$(REQUIREJS_TEXT_VERSION).tar.gz
	@mkdir -p $(BUILD)/js/requires
	tar xzf $< -C $(BUILD)/js/requires
	sed -i "s/define(\['module'\]/define('text',\['module'\]/g" $@
	
JQUERY_VERSION := 2.1.4

$(RESOURCES)/jquery-$(JQUERY_VERSION).js:
	@mkdir -p $(RESOURCES)
	wget -O$@ http://code.jquery.com/jquery-$(JQUERY_VERSION).js

JQUERY_COOKIE_VERSION := 1.4.1

$(RESOURCES)/jquery.cookie-$(JQUERY_COOKIE_VERSION).min.js:
	@mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ https://github.com/carhartl/jquery-cookie/releases/download/v$(JQUERY_COOKIE_VERSION)/jquery.cookie-$(JQUERY_COOKIE_VERSION).min.js

JQUERY_HOTKEYS_VERSION := 0.2.0

$(RESOURCES)/jquery.hotkeys-$(JQUERY_HOTKEYS_VERSION).tar.gz:
	@mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ https://github.com/jeresig/jquery.hotkeys/archive/$(JQUERY_HOTKEYS_VERSION).tar.gz

$(BUILD)/js/requires/jquery.hotkeys-$(JQUERY_HOTKEYS_VERSION)/jquery.hotkeys.js: $(RESOURCES)/jquery.hotkeys-$(JQUERY_HOTKEYS_VERSION).tar.gz
	@mkdir -p $(BUILD)/js/requires
	tar xzf $< -C $(BUILD)/js/requires
	touch $@

# Build a minified file of all JS requirements for `stencila.js`
# RequireJS is concatenated last to avoid conflicts with JQuery and plugins
$(BUILD)/js/requires.min.js: \
	        $(RESOURCES)/jquery-$(JQUERY_VERSION).js \
	        $(RESOURCES)/jquery.cookie-$(JQUERY_COOKIE_VERSION).min.js \
	        $(BUILD)/js/requires/jquery.hotkeys-$(JQUERY_HOTKEYS_VERSION)/jquery.hotkeys.js \
			$(RESOURCES)/require-$(REQUIREJS_VERSION).js \
			$(BUILD)/js/requires/text-$(REQUIREJS_TEXT_VERSION)/text.js
	@mkdir -p $(BUILD)/js
	uglifyjs $^ --compress --mangle --comments 	> $@

JASMINE_VERSION := 2.3.4

$(RESOURCES)/jasmine-standalone-$(JASMINE_VERSION).zip:
	@mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ https://github.com/jasmine/jasmine/releases/download/v$(JASMINE_VERSION)/jasmine-standalone-$(JASMINE_VERSION).zip
	
$(BUILD)/js/requires/jasmine-$(JASMINE_VERSION): $(RESOURCES)/jasmine-standalone-$(JASMINE_VERSION).zip
	@mkdir -p $(BUILD)/js/requires
	unzip -qoj $< 'lib/jasmine-$(JASMINE_VERSION)/*' -d $(BUILD)/js/requires/jasmine-$(JASMINE_VERSION)

$(BUILD)/js/requires/jasmine-$(JASMINE_VERSION)/mock-ajax.js: $(BUILD)/js/requires/jasmine-$(JASMINE_VERSION)
	wget --no-check-certificate -O$@ https://raw.github.com/jasmine/jasmine-ajax/master/lib/mock-ajax.js

# Run Javascript tests
js-tests: build/current $(BUILD)/js/requires.min.js $(BUILD)/js/requires/jasmine-$(JASMINE_VERSION) $(BUILD)/js/requires/jasmine-$(JASMINE_VERSION)/mock-ajax.js
	(phantomjs js/tests/spec-runner.js js/tests/spec-runner.html)  || (exit 1)

# Provide files needed to serve files from the Javascript modules during development
# You must run the make taks `build-serve` see above.
js-develop: $(BUILD)/js/requires.min.js
	ln -sfT $(ROOT)/js/stencila.js $(BUILD)/js/stencila.js

JS_MIN := $(BUILD)/js/stencila-$(VERSION).min.js
$(JS_MIN) : $(BUILD)/js/requires.min.js js/stencila.js
	uglifyjs $^ --compress --mangle > $@
js-build: $(JS_MIN)

# Deliver Javascript to get.stenci.la
# The `stencila-latest.min.js` should not be cached
js-deliver: $(JS_MIN)
ifeq (dirty,$(DIRTY))
	$(error Delivery is not done for dirty versions: $(VERSION). Commit or stash and try again.)
else
	aws s3 cp $(JS_MIN) s3://get.stenci.la/js/ --content-type application/json --cache-control max-age=31536000
	aws s3 cp s3://get.stenci.la/js/stencila-$(VERSION).min.js s3://get.stenci.la/js/stencila-latest.min.js
endif

js-clean:
	rm -f $(BUILD)/js/requires.min.js


#################################################################################################
# Stencila Python package

# If PY_VERSION is not defined then get it
ifndef PY_VERSION
  PY_VERSION := $(shell ./config.py py_version)
endif

PY_BUILD := $(BUILD)/py/$(PY_VERSION)

ifeq ($(OS), linux)
  PY_INCLUDE_DIR := /usr/include/python$(PY_VERSION)
  PY_EXE := python$(PY_VERSION)
endif

PY_BOOST_PYTHON_LIB := boost_python
#ifeq $(or $(if $(OS),3.0), $(if $(OS),3.0)
#	PY_BOOST_PYTHON_LIB += 3
#endif

PY_PACKAGE_PYS := $(patsubst %.py,$(PY_BUILD)/stencila/%.py,$(notdir $(wildcard py/stencila/*.py)))
PY_PACKAGE_OBJECTS := $(patsubst %.cpp,$(PY_BUILD)/objects/%.o,$(notdir $(wildcard py/stencila/*.cpp)))

PY_CXX_FLAGS := --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2 -fPIC

PY_SETUP_EXTRA_OBJECTS := $(patsubst $(PY_BUILD)/%,%,$(PY_PACKAGE_OBJECTS))
PY_SETUP_LIB_DIRS := ../../cpp/library ../../cpp/requires/boost/lib
PY_SETUP_LIBS := stencila $(PY_BOOST_PYTHON_LIB) python$(PY_VERSION) $(CPP_OTHER_LIBS)

# Stencila version number used in the Python wheel file name
# Replace dashes with underscores
VERSION_PY_WHEEL := $(subst -,_,$(VERSION))

# Print Python related Makefile variables; useful for debugging
py-vars:
	@echo PY_VERSION : $(PY_VERSION)
	@echo PY_BUILD : $(PY_BUILD)

$(PY_BUILD)/stencila/%.py: py/stencila/%.py
	@mkdir -p $(PY_BUILD)/stencila
	cp $< $@

$(PY_BUILD)/objects/%.o: py/stencila/%.cpp
	@mkdir -p $(PY_BUILD)/objects
	$(CXX) $(PY_CXX_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -I$(PY_INCLUDE_DIR) -o$@ -c $<

# Copy setup.py to build directory and run it from there
# Create and touch a `dummy.cpp` for setup.py to build
# Record name of the wheel to file for reading by other build tasks
$(PY_BUILD)/latest.txt: py/setup.py py/scripts/stencila-py $(PY_PACKAGE_PYS) $(PY_PACKAGE_OBJECTS) $(BUILD)/cpp/library/libstencila.a
	cp py/setup.py $(PY_BUILD)
	mkdir -p $(PY_BUILD)/scripts
	cp py/scripts/stencila-py $(PY_BUILD)/scripts/stencila-py
	cd $(PY_BUILD)/ ;\
		export \
			VERSION=$(VERSION) \
			EXTRA_OBJECTS='$(PY_SETUP_EXTRA_OBJECTS)' \
			LIBRARY_DIRS='$(PY_SETUP_LIB_DIRS)' \
			LIBRARIES='$(PY_SETUP_LIBS)' ;\
		touch dummy.cpp ;\
		$(PY_EXE) setup.py bdist_wheel
	cd $(PY_BUILD)/dist; echo `ls -rt *.whl | tail -n1` > ../latest.txt

py-package: $(PY_BUILD)/latest.txt

# Create a virtual environment to be used for testing with the Python version
# Using a virtual environment allows the Stencila wheel to be installed locally,
# i.e. without root privalages, and also does not affect the host machines Python setup 
$(PY_BUILD)/testenv/bin/activate:
	@mkdir -p $(PY_BUILD);
	cd $(PY_BUILD) ;\
		virtualenv --python=python$(PY_VERSION) --no-site-packages testenv

$(PY_BUILD)/testenv/lib/python$(PY_VERSION)/site-packages/stencila: $(PY_BUILD)/testenv/bin/activate $(PY_BUILD)/latest.txt
	@mkdir -p $(PY_BUILD);
	cd $(PY_BUILD) ;\
		. testenv/bin/activate ;\
		pip install --upgrade --force-reinstall dist/`cat latest.txt`

py-tests: py/tests/tests.py $(PY_BUILD)/testenv/lib/python$(PY_VERSION)/site-packages/stencila
	cp py/tests/tests.py $(PY_BUILD)/testenv
	cd $(PY_BUILD)/testenv ;\
		. bin/activate ;\
		(python tests.py)||(exit 1)

py-install: $(PY_BUILD)/testenv/bin/activate $(PY_BUILD)/latest.txt
	cd $(PY_BUILD) ;\
		sudo pip install --upgrade --force-reinstall dist/`cat latest.txt`

py-clean:
	rm -rf $(PY_BUILD)

# Deliver Python package to get.stenci.la
py-deliver: py-package
ifeq (dirty,$(DIRTY))
	$(error Delivery is not done for dirty versions: $(VERSION). Commit or stash and try again.)
else
	$(eval PY_WHEEL := $(shell cat $(PY_BUILD)/latest.txt))
	aws s3 cp $(PY_BUILD)/dist/$(PY_WHEEL) s3://get.stenci.la/py/
	$(call DELIVERY_NOTIFY,py,$(PY_VERSION),$(OS)/$(ARCH),http://get.stenci.la/py/$(PY_WHEEL))
endif

#################################################################################################
# Stencila R package

# If R_VERSION is not defined then get it
ifndef R_VERSION
  # Version number excludes any patch number
  R_VERSION := $(shell Rscript -e "cat(R.version\$$major,strsplit(R.version\$$minor,'\\\\.')[[1]][1],sep='.')" )
endif

# Shortcut to the R build directory
R_BUILD := $(BUILD)/r/$(R_VERSION)

# The R version can not include any of the non numeric suffixes (commit and/or dirty)
R_PACKAGE_VERSION := $(firstword $(subst -, ,$(VERSION)))

# Define other platform specific variables...
ifeq ($(OS),linux)
R_PACKAGE_EXT := tar.gz
R_DLL_EXT := so
R_REPO_DIR := src/contrib
R_REPO_TYPE := source
endif
ifeq ($(OS),win)
R_PACKAGE_EXT := zip
R_DLL_EXT := dll
R_REPO_DIR := bin/windows/contrib/$(R_VERSION)
R_REPO_TYPE := win.binary
endif

# Path to files delivered to http:://get.stenci.la
R_UNIQUE_PATH := $(OS)/$(ARCH)/$(R_VERSION)/stencila-$(R_PACKAGE_VERSION)
R_DLL_PATH := r/dll/$(R_UNIQUE_PATH).zip
R_BUNDLE_PATH := r/bundle/$(R_UNIQUE_PATH).$(R_PACKAGE_EXT)

# Platform dependent variables
R_CPPFLAGS := $(shell R CMD config --cppflags)
R_LDFLAGS := $(shell R CMD config --ldflags)
RCPP_CXXFLAGS := $(shell Rscript -e "Rcpp:::CxxFlags()")
RCPP_LDFLAGS :=  $(shell Rscript -e "Rcpp:::LdFlags()")

# Print R related Makefile variables; useful for debugging
r-vars:
	@echo R_VERSION : $(R_VERSION)
	@echo R_BUILD : $(R_BUILD)
	@echo R_PACKAGE_VERSION : $(R_PACKAGE_VERSION)
	@echo R_PACKAGE_EXT : $(R_PACKAGE_EXT)
	@echo R_DLL_EXT : $(R_DLL_EXT)
	@echo R_DLL_PATH : $(R_DLL_PATH)
	@echo R_BUNDLE_PATH : $(R_BUNDLE_PATH)
	@echo R_REPO_DIR : $(R_REPO_DIR)
	@echo R_REPO_TYPE : $(R_REPO_TYPE)
	@echo R_CPPFLAGS : $(R_CPPFLAGS)
	@echo R_LDFLAGS : $(R_LDFLAGS)
	@echo RCPP_CXXFLAGS : $(RCPP_CXXFLAGS)
	@echo RCPP_LDFLAGS : $(RCPP_LDFLAGS)

R_COMPILE_FLAGS := --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2 \
				-Icpp $(CPP_REQUIRES_INC_DIRS) $(R_CPPFLAGS) $(RCPP_CXXFLAGS)
ifeq ($(OS),linux)
R_COMPILE_FLAGS += -fPIC
endif

# Compile each cpp file
R_PACKAGE_OBJECTS := $(patsubst %.cpp,$(R_BUILD)/objects/%.o,$(notdir $(wildcard r/stencila/*.cpp)))
$(R_BUILD)/objects/%.o: r/stencila/%.cpp
	@mkdir -p $(R_BUILD)/objects
	$(CXX) $(R_COMPILE_FLAGS) -o$@ -c $<
	
# Build DLL
R_DLL_LIBS := stencila $(CPP_OTHER_LIBS)
R_DLL_LIBS := $(patsubst %, -l%,$(R_DLL_LIBS))
$(R_BUILD)/stencila.$(R_DLL_EXT): $(R_PACKAGE_OBJECTS) $(BUILD)/cpp/library/libstencila.a
	$(CXX) -shared -o$@ $^ $(R_LDFLAGS) $(RCPP_LDFLAGS) -L$(BUILD)/cpp/library $(R_DLL_LIBS)
r-dll: $(R_BUILD)/stencila.$(R_DLL_EXT)

# Check DLL can be loaded
r-dll-check: $(R_BUILD)/stencila.$(R_DLL_EXT)
	Rscript -e "dyn.load('$(R_BUILD)/stencila.$(R_DLL_EXT)')"

# Build DLL zip file
ifeq ($(OS),win)
# Extra DLLs needed on windows. These should be available from the MSYS2 install.
# List of extra DLLs required can be determined by running Dependency Walker
# (http://www.dependencywalker.com/) on stencila.dll
R_DLL_EXTRA := $(patsubst %, /c/msys64/mingw64/bin/%, libeay32.dll libgcc_s_seh-1.dll libstdc++-6.dll libwinpthread-1.dll ssleay32.dll zlib1.dll)
endif
$(R_BUILD)/stencila-dll.zip: r-dll-check
	rm -f $@
	zip -j $@ $(R_BUILD)/stencila.$(R_DLL_EXT) $(R_DLL_EXTRA)
r-dll-zip: $(R_BUILD)/stencila-dll.zip

# Copy over DLL zip file
R_PACKAGE_DLL := $(R_BUILD)/stencila/inst/bin/stencila-dll.zip
$(R_PACKAGE_DLL): $(R_BUILD)/stencila-dll.zip
	@mkdir -p $(R_BUILD)/stencila/inst/bin
	cp $< $@

# Copy over `stencila-r`
R_PACKAGE_CLI := $(R_BUILD)/stencila/inst/bin/stencila-r
$(R_PACKAGE_CLI): r/stencila-r
	@mkdir -p $(R_BUILD)/stencila/inst/bin
	cp $< $@

# Copy over each R file
R_PACKAGE_RS := $(patsubst %, $(R_BUILD)/stencila/R/%, $(notdir $(wildcard r/stencila/*.R)))
$(R_BUILD)/stencila/R/%.R: r/stencila/%.R
	@mkdir -p $(R_BUILD)/stencila/R
	cp $< $@

# Copy over each unit test file
R_PACKAGE_TESTS := $(patsubst %, $(R_BUILD)/stencila/inst/unitTests/%, $(notdir $(wildcard r/tests/*.R) $(wildcard r/tests/*.xlsx)))
$(R_BUILD)/stencila/inst/unitTests/%: r/tests/%
	@mkdir -p $(R_BUILD)/stencila/inst/unitTests
	cp $< $@

# Copy over DESCRIPTION
R_PACKAGE_DESC := $(R_BUILD)/stencila/DESCRIPTION
$(R_PACKAGE_DESC): r/DESCRIPTION
	cp $< $@

# Finalise the package directory
R_PACKAGE_DATE := $(shell date --utc +%Y-%m-%dT%H:%M:%SZ)
$(R_BUILD)/stencila: $(R_PACKAGE_DLL) $(R_PACKAGE_CLI) $(R_PACKAGE_RS) $(R_PACKAGE_TESTS) $(R_PACKAGE_DESC)
	# Edit package version and date using sed:
	#	.* = anything, any number of times
	#	$ = end of line
	# The $ needs to be doubled for escaping make
	# ISO 8601 date/time stamp used: http://en.wikipedia.org/wiki/ISO_8601
	sed -i 's!Version: .*$$!Version: $(R_PACKAGE_VERSION)!' $(R_PACKAGE_DESC)
	sed -i 's!Date: .*$$!Date: $(R_PACKAGE_DATE)!' $(R_PACKAGE_DESC)
	# Run roxygen to generate Rd files and NAMESPACE file
	cd $(R_BUILD) ;\
		rm -f stencila/man/*.Rd ;\
		Rscript -e "library(roxygen2);roxygenize('stencila');"
	# Touch the directory to ensure it is newer than its contents
	touch $@
r-package-dir: $(R_BUILD)/stencila

# Check the package by running R CMD check
# on the package directory. Do this in the
# build directory to prevent polluting source tree
r-package-check: $(R_BUILD)/stencila
	cd $(R_BUILD) ;\
	  R CMD check stencila

# Build the package
R_PACKAGE_FILE_BUILT := stencila_$(R_PACKAGE_VERSION).$(R_PACKAGE_EXT) # What gets build by R CMD
R_PACKAGE_FILE := stencila_$(VERSION).$(R_PACKAGE_EXT) # What we want it to be called (with non-standard-for-R version string)
$(R_BUILD)/$(R_PACKAGE_FILE): $(R_BUILD)/stencila
ifeq ($(OS),linux)
	cd $(R_BUILD); R CMD build stencila
endif
ifeq ($(OS),win)
	cd $(R_BUILD); R CMD INSTALL --build stencila
endif
ifneq ($(R_BUILD)/$(R_PACKAGE_FILE_BUILT),$(R_BUILD)/$(R_PACKAGE_FILE))
	mv $(R_BUILD)/$(R_PACKAGE_FILE_BUILT) $(R_BUILD)/$(R_PACKAGE_FILE)
endif
r-package: $(R_BUILD)/$(R_PACKAGE_FILE)

# Deposit package into local repository for mirroring to http://get.stenci.la/r
# This allows installs from within R i.e:
# 	
# 	install.packages('stencila',repo='http://get.stenci.la/r')
# 	
# See http://cran.r-project.org/doc/manuals/R-admin.html#Setting-up-a-package-repository
r-repo: r-package
ifeq (dirty,$(DIRTY))
	$(error Local repo is not created for dirty versions: $(VERSION). Commit or stash and try again.)
else
	# Make R package repository sub directory
	mkdir -p $(R_BUILD)/repo/$(R_REPO_DIR)
	# Copy package there
	cp $(R_BUILD)/$(R_PACKAGE_FILE) $(R_BUILD)/repo/$(R_REPO_DIR)
	# Generate the PACKAGE file for the repo
	Rscript -e "require(tools); tools::write_PACKAGES('$(R_BUILD)/repo/$(R_REPO_DIR)',type='$(R_REPO_TYPE)')"
endif

# Deliver R package to get.stenci.la
# Requires http://aws.amazon.com/cli/ and access keys for get.stenci.la
r-deliver: $(R_BUILD)/stencila-dll.zip r-repo
ifeq (dirty,$(DIRTY))
	$(error Delivery is not done for dirty versions: $(VERSION). Commit or stash and try again.)
else
	aws s3 cp $(R_BUILD)/stencila-dll.zip s3://get.stenci.la/$(R_DLL_PATH)
	aws s3 cp --recursive $(R_BUILD)/repo/$(R_REPO_DIR) s3://get.stenci.la/r/$(R_REPO_DIR)
	$(call DELIVERY_NOTIFY,r,$(R_VERSION),$(OS)/$(ARCH),http://get.stenci.la/$(R_REPO_DIR)/$(R_PACKAGE_FILE))
endif

# Install package in a testenv directory
# This is better than installing package in the user's R library location
# Previously, the `lib.loc='.'` argument was supplied to `library` but
# that did not work with the current DLL loading mechanism. So `.libPaths('.')`
# is used instead
$(R_BUILD)/testenv/stencila: $(R_BUILD)/$(R_PACKAGE_FILE)
	cd $(R_BUILD) ;\
	  mkdir -p testenv ;\
	  R CMD INSTALL -l testenv $(R_PACKAGE_FILE)

# Test the package by running unit tests
r-tests: $(R_BUILD)/testenv/stencila $(R_BUILD)/$(R_PACKAGE_FILE)
	cd $(R_BUILD) ;cd testenv ;\
	    (Rscript -e ".libPaths(c('.',.libPaths()[1])); library(stencila); setwd('stencila/unitTests/'); source('do-svUnit.R'); quit(save='no',status=fails);") || (exit 1)  ;\
	    (Rscript -e ".libPaths(c('.',.libPaths()[1])); setwd('stencila/unitTests/'); source('testthat-spreadsheet.R'); quit(save='no',status=fails);") || (exit 1)

# Install R on the local host
# Not intended for development but rather 
# to install on the host machine after a build
r-install: $(R_BUILD)/$(R_PACKAGE_FILE)
	R CMD INSTALL $(R_BUILD)/$(R_PACKAGE_FILE)
	sudo Rscript -e 'library(stencila);stencila:::install()'

# Remove everything
r-clean:
	rm -rf $(BUILD)/r


#################################################################################################
# Stencila web browser module

web-requires:
	cd web; npm install

web-build:
	cd web; gulp build

web-watch:
	cd web; gulp watch

web-examples:
	stencila-r web/examples/a render write page:"index.html"
	stencila-r web/examples/stencil-with-pars render write page:"index.html"
	stencila-r web/examples/b update write page:"index.html"
	stencila-r web/examples/sheet-with-error update write page:"index.html"

web-devserve:
	cd web; node server.js

web-devserve-hub:
	cd web; node server.js https://stenci.la

web-devserve-hubdev:
	cd web; node server.js http://localhost:7300

web-deliver:
ifeq (dirty,$(DIRTY))
	$(error Delivery is not done for dirty versions: $(VERSION). Commit or stash and try again.)
else
	aws s3 sync web/build s3://get.stenci.la/web/
	$(call DELIVERY_NOTIFY,web,ES5)
endif

web-clean:
	rm -rf web/build

#################################################################################################

# Clean everything!
clean:
	rm -rf $(BUILD)