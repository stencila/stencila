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
# Get count of commits since last tag
COMMIT_COUNT := $(shell git rev-list  `git rev-list --tags --no-walk --max-count=1`..HEAD --count)
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
	@echo COMMIT_COUNT: $(COMMIT_COUNT)
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
ifeq ($(OS), win)
	# b2 must be called with "system" layout of library names and header locations (otherwise it defaults to 'versioned' on Windows)
	# b2 must be called with "release" build otherwise defaults to debug AND release, which with "system" causes an 
	#   error (http://boost.2283326.n4.nabble.com/atomic-building-with-layout-system-mingw-bug-7482-td4640920.html)
	BOOST_B2_FLAGS += --layout=system release toolset=gcc
else
	# cxxflags=-fPIC - so that the statically compiled library has position independent code for use in shared libraries
	BOOST_B2_FLAGS += cxxflags=-fPIC
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


CMARK_VERSION := 0.25.2

$(RESOURCES)/cmark-$(CMARK_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/jgm/cmark/archive/$(CMARK_VERSION).tar.gz

$(BUILD)/cpp/requires/cmark: $(RESOURCES)/cmark-$(CMARK_VERSION).tar.gz
	mkdir -p $(BUILD)/cpp/requires
	tar xzf $< -C $(BUILD)/cpp/requires
	rm -rf $@
	mv $(BUILD)/cpp/requires/cmark-$(CMARK_VERSION) $@
	touch $@

$(BUILD)/cpp/requires/cmark/build/src/libcmark.a: $(BUILD)/cpp/requires/cmark
	mkdir -p $</build
	cd $</build ;\
		cmake .. -DCMAKE_C_FLAGS=-fPIC ;\
		make ;\
		rm src/libcmark.so* # Prevent linking to shared library

cpp-requires-cmark: $(BUILD)/cpp/requires/cmark/build/src/libcmark.a

CPP_REQUIRES_INC_DIRS += -I$(BUILD)/cpp/requires/cmark/src -I$(BUILD)/cpp/requires/cmark/build/src
CPP_REQUIRES_LIB_DIRS += -L$(BUILD)/cpp/requires/cmark/build/src
CPP_REQUIRES_LIBS += cmark


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
ifeq ($(OS), win)
	LIBGIT2_CMAKE_FLAGS += -G "MSYS Makefiles"
else
	LIBGIT2_CMAKE_FLAGS += -DCMAKE_C_FLAGS=-fPIC
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
ifneq ($(OS), win)
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
ifeq ($(OS), win)
	cd $(BUILD)/cpp/requires/tidy-html5/build/cmake ;\
		cmake --build . --config Release
	# Under MSYS2 there are lots of multiple definition errors for localize symbols in the library
	objcopy --localize-symbols=cpp/requires/tidy-html5-localize-symbols.txt $(BUILD)/cpp/requires/tidy-html5/build/cmake/libtidys.a
else
	cd $(BUILD)/cpp/requires/tidy-html5/build/cmake ;\
		make
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
	cpp-requires-cmark \
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
ifeq ($(OS), osx)
	CPP_OTHER_LIBS += curl
endif
ifeq ($(OS), win)
	CPP_OTHER_LIBS += ws2_32 mswsock ssh2
endif

#################################################################################################
# Stencila C++ library

# C++ compiler options when compiling Stencila source into libstencila.a or 
# language packages
# 
# -Wno-unknown-pragmas : for clang, prevents lots of warings
# -Wno-missing-braces : for clang, unecessary, see http://stackoverflow.com/a/13905432/4625911
# -Wno-unused-local-typedefs : because boost defines quite a lot of local typedefs
# -Wno-unknown-warning-option : because clang doesn't know -Wno-unused-local-typedefs
CPP_FLAGS := --std=c++11 -O2 -Wall \
			   -Wno-unknown-pragmas -Wno-missing-braces -Wno-unused-local-typedefs \
			   -Wno-unknown-warning-option
ifneq ($(OS), win)
	CPP_FLAGS += -fPIC
endif

# Compile version.o
CPP_VERSION_TXT := $(BUILD)/cpp/library/generated/version.txt
CPP_VERSION_CPP := $(BUILD)/cpp/library/generated/version.cpp
CPP_VERSION_O := $(BUILD)/cpp/library/objects/version.o
$(CPP_VERSION_O):
ifneq ($(shell if [ -e "$(CPP_VERSION_TXT)" ]; then cat "$(CPP_VERSION_TXT)"; else echo ""; fi),$(VERSION))
	@mkdir -p $(dir $@)
	echo "$(VERSION)" > $(CPP_VERSION_TXT)
	echo "#include <stencila/version.hpp>" > $(CPP_VERSION_CPP)
	echo "const std::string Stencila::version = \"$(VERSION)\";" >> $(CPP_VERSION_CPP)
	echo "const std::string Stencila::commit = \"$(COMMIT)\";" >> $(CPP_VERSION_CPP)
	$(CXX) $(CPP_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -o$@ -c $(CPP_VERSION_CPP)
endif
.PHONY: $(CPP_VERSION_O)

# Compile C++ source files
$(BUILD)/cpp/library/objects/stencila-%.o: cpp/stencila/%.cpp
	@mkdir -p $(BUILD)/cpp/library/objects
	$(CXX) $(CPP_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -o$@ -c $<

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
	$(CXX) $(CPP_FLAGS) -Wno-unused-variable -Icpp $(CPP_REQUIRES_INC_DIRS) -I$(BUILD)/cpp/library/generated -o$@ -c $<	

# Generate syntax lexer using Flex
$(BUILD)/cpp/library/generated/syntax-%-lexer.cpp: cpp/stencila/syntax-%.l
	@mkdir -p $(dir $@)
	flex --outfile $@ --header-file=$(dir $@)syntax-$*-lexer.hpp $<

# Compile syntax lexer
$(BUILD)/cpp/library/objects/stencila-syntax-%-lexer.o: $(BUILD)/cpp/library/generated/syntax-%-lexer.cpp $(BUILD)/cpp/library/generated/syntax-%-parser.cpp
	@mkdir -p $(dir $@)
	$(CXX) $(CPP_FLAGS) -Wno-deprecated-register -Wno-unused-function -Icpp -I$(BUILD)/cpp/library/generated -o$@ -c $<	

# Generate the `parse()` method by using the template .cxx file
$(BUILD)/cpp/library/generated/syntax-%-parse.cpp: cpp/stencila/syntax-parser-parse.cxx
	@mkdir -p $(dir $@)
	sed -e 's!{lang}!$*!' -e 's!{lang-title}!\u$*!' cpp/stencila/syntax-parser-parse.cxx > $@

# Compile lexer and parser source
$(BUILD)/cpp/library/objects/stencila-syntax-%-parse.o: $(BUILD)/cpp/library/generated/syntax-%-parse.cpp \
														$(BUILD)/cpp/library/objects/stencila-syntax-%-lexer.o \
	 		  											$(BUILD)/cpp/library/objects/stencila-syntax-%-parser.o
	$(CXX) $(CPP_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS) -I$(BUILD)/cpp/library/generated -o$@ -c $<

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
	$(call CPP_LIBRARY_EXTRACT,cmark/build/src/libcmark.a,cmark)
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
# To output lists of contents for checking:
# 		ar t libstencila.a > files.txt 
#		nm -gC libstencila.a > symbols.txt  # C demangles but is an invalid option on OS X
$(BUILD)/cpp/library/libstencila.a: cpp-library-objects cpp-requires-objects
	cd $(BUILD)/cpp/library  && $(AR) rc libstencila.a `find . -name "*.o"`
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
CPP_TEST_COMPILE := $(CXX) $(CPP_FLAGS) -Icpp $(CPP_REQUIRES_INC_DIRS)
CPP_TEST_LIB_DIRS := $(CPP_REQUIRES_LIB_DIRS)
CPP_TEST_LIBS := $(CPP_REQUIRES_LIBS) $(CPP_OTHER_LIBS) boost_unit_test_framework boost_timer boost_chrono
ifeq ($(OS), linux)
CPP_TEST_COMPILE += -g -O0 --coverage
CPP_TEST_LIBS += gcov
endif
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
$(BUILD)/docker/ubuntu-14.04-py-2.7/image.txt: docker/ubuntu-14.04-py-2.7/Dockerfile docker/stencila-session.py
	@mkdir -p $(dir $@)
	cp docker/ubuntu-14.04-py-2.7/Dockerfile $(dir $@)
	cp docker/stencila-session.py $(dir $@)
	cp $(shell ls -rt py/dist/*.whl | tail -n 1) $(dir $@)/stencila.whl
	docker build --tag stencila/ubuntu-14.04-py-2.7:$(VERSION) $(dir $@)
	docker tag --force stencila/ubuntu-14.04-py-2.7:$(VERSION) stencila/ubuntu-14.04-py-2.7:latest
	echo "stencila/ubuntu-14.04-py-2.7:$(VERSION)" > $@

docker-py-build: $(BUILD)/docker/ubuntu-14.04-py-2.7/image.txt

docker-py-deliver: docker-py-build
	docker push stencila/ubuntu-14.04-py-2.7:$(VERSION)
	docker push stencila/ubuntu-14.04-py-2.7:latest
	$(call DELIVERY_NOTIFY,docker,ubuntu-14.04-py-2.7)

#################################################################################################

# Clean everything!
clean:
	rm -rf $(BUILD)