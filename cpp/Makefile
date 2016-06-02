all: library

include ../shared.make

# Collect necessary include and lib directories and library names
REQUIRES_INC_DIRS := 
REQUIRES_LIB_DIRS := 
REQUIRES_LIBS :=


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
#   --with-libraries - so that only those libraries that are needed are built
BOOST_BOOTSTRAP_FLAGS := --with-libraries=atomic,chrono,date_time,filesystem,program_options,python,regex,system,test,timer,thread
ifeq ($(OS), win)
	BOOST_BOOTSTRAP_FLAGS += gcc
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

build/requires/boost-built.flag: build/requires/boost
	cd $< ; ./bootstrap.sh gcc $(BOOST_BOOTSTRAP_FLAGS)
ifeq ($(OS), win)
	# Under MSYS, project-config.jam must be edited to fix [this error](http://stackoverflow.com/a/5244844/1583041) 
	# The spaces are important so that we don't clobber the python setup in this config file
	#sed -i "s!mingw !gcc !" $</project-config.jam
endif
	cd $< ; ./b2 $(BOOST_B2_FLAGS)
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/boost/include
REQUIRES_LIB_DIRS += -Lbuild/requires/boost/lib
REQUIRES_LIBS += boost_filesystem boost_system boost_regex boost_thread 

requires-boost: build/requires/boost-built.flag


CMARK_VERSION := 0.25.2

resources/cmark-$(CMARK_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/jgm/cmark/archive/$(CMARK_VERSION).tar.gz

build/requires/cmark: resources/cmark-$(CMARK_VERSION).tar.gz
	mkdir -p build/requires
	tar xzf $< -C build/requires
	rm -rf $@
	mv build/requires/cmark-$(CMARK_VERSION) $@
	touch $@

CMARK_CMAKE_FLAGS := 
ifneq ($(OS), win)
	CMARK_CMAKE_FLAGS += -DCMAKE_C_FLAGS=-fPIC
endif
build/requires/cmark/build/src/libcmark.a: build/requires/cmark
	mkdir -p $</build
	cd $</build ;\
		cmake .. $(CMARK_CMAKE_FLAGS) ;\
		make libcmark_static

requires-cmark: build/requires/cmark/build/src/libcmark.a

REQUIRES_INC_DIRS += -Ibuild/requires/cmark/src -Ibuild/requires/cmark/build/src
REQUIRES_LIB_DIRS += -Lbuild/requires/cmark/build/src
REQUIRES_LIBS += cmark


LIBGIT2_VERSION := 0.24.0

resources/libgit2-$(LIBGIT2_VERSION).zip:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/libgit2/libgit2/archive/v$(LIBGIT2_VERSION).zip

build/requires/libgit2: resources/libgit2-$(LIBGIT2_VERSION).zip
	mkdir -p build/requires
	unzip -qo $< -d build/requires
	rm -rf $@
	mv build/requires/libgit2-$(LIBGIT2_VERSION) $@
	touch $@

# For build options see https://libgit2.github.com/docs/guides/build-and-link/
#  	BUILD_CLAR=OFF - do not build tests
#  	BUILD_SHARED_LIBS=OFF - do not build shared library
LIBGIT2_CMAKE_FLAGS := -DBUILD_CLAR=OFF -DBUILD_SHARED_LIBS=OFF
ifneq ($(OS), win)
	LIBGIT2_CMAKE_FLAGS += -DCMAKE_C_FLAGS=-fPIC
endif
build/requires/libgit2-built.flag: build/requires/libgit2
	cd $< ;\
	  mkdir -p build ;\
	  cd build ;\
	  cmake .. $(LIBGIT2_CMAKE_FLAGS);\
	  cmake --build .
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/libgit2/include
REQUIRES_LIB_DIRS += -Lbuild/requires/libgit2/build
REQUIRES_LIBS += git2

requires-libgit2: build/requires/libgit2-built.flag



LIBZIP_VERSION := 1.1.2

resources/libzip-$(LIBZIP_VERSION).tar.gz:
	mkdir -p resources
	wget -q -O $@ http://www.nih.at/libzip/libzip-$(LIBZIP_VERSION).tar.gz

build/requires/libzip: resources/libzip-$(LIBZIP_VERSION).tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/libzip-$(LIBZIP_VERSION) $@
	touch $@

build/requires/libzip/lib/.libs/libzip.a: build/requires/libzip
	cd $<  && ./configure --disable-shared --enable-static --with-pic && make

REQUIRES_INC_DIRS += -Ibuild/requires/libzip/lib
REQUIRES_LIB_DIRS += -Lbuild/requires/libzip/lib/.libs
REQUIRES_LIBS += zip

requires-libzip: build/requires/libzip/lib/.libs/libzip.a



CPP_NETLIB_VERSION := 0.11.2

resources/cpp-netlib-$(CPP_NETLIB_VERSION)-final.tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/cpp-netlib/cpp-netlib/archive/cpp-netlib-$(CPP_NETLIB_VERSION)-final.tar.gz
	
build/requires/cpp-netlib: resources/cpp-netlib-$(CPP_NETLIB_VERSION)-final.tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/cpp-netlib-cpp-netlib-$(CPP_NETLIB_VERSION)-final build/requires/cpp-netlib
	touch $@

# cpp-netlib needs to be compiled with OPENSSL_NO_SSL2 defined because SSL2 is insecure and depreciated and on
# some systems (e.g. Ubuntu) OpenSSL is compiled with no support for it
CPP_NETLIB_CMAKE_FLAGS := -DCMAKE_BUILD_TYPE=Debug  -DCMAKE_C_COMPILER=gcc -DCMAKE_CXX_COMPILER=g++ -DCMAKE_CXX_FLAGS="-DOPENSSL_NO_SSL2 -O2 -fPIC"
# Under MSYS some additional CMake flags need to be specified
# The "-I/usr/include" in -DCMAKE_CXX_FLAGS seems uncessary but it's not
ifeq ($(OS), win)
CPP_NETLIB_CMAKE_FLAGS += -DOPENSSL_ROOT_DIR=/usr/ssl -DOPENSSL_INCLUDE_DIR=/usr/include/ -DOPENSSL_LIBRARIES=/usr/lib/ -DCMAKE_CXX_FLAGS="-DOPENSSL_NO_SSL2 -O2 -fPIC -I/usr/include"
endif
build/requires/cpp-netlib/libs/network/src/libcppnetlib-client-connections.a: build/requires/cpp-netlib
	cd build/requires/cpp-netlib; \
		export BOOST_ROOT=../boost ; \
		cmake $(CPP_NETLIB_CMAKE_FLAGS); \
		make cppnetlib-client-connections cppnetlib-server-parsers cppnetlib-uri

REQUIRES_INC_DIRS += -Ibuild/requires/cpp-netlib/
REQUIRES_LIB_DIRS += -Lbuild/requires/cpp-netlib/libs/network/src
REQUIRES_LIBS += cppnetlib-client-connections cppnetlib-uri

requires-cpp-netlib: build/requires/cpp-netlib/libs/network/src/libcppnetlib-client-connections.a


PUGIXML_VERSION := 1.7

resources/pugixml-$(PUGIXML_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/zeux/pugixml/archive/v$(PUGIXML_VERSION).tar.gz

build/requires/pugixml: resources/pugixml-$(PUGIXML_VERSION).tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/pugixml-$(PUGIXML_VERSION) build/requires/pugixml
	touch $@

PUGIXML_CXX_FLAGS := -O2
ifneq ($(OS), win)
	PUGIXML_CXX_FLAGS += -fPIC
endif
build/requires/pugixml/src/libpugixml.a: build/requires/pugixml
	cd $</src ;\
	  $(CXX) $(PUGIXML_CXX_FLAGS) -c pugixml.cpp ;\
	  $(AR) rcs libpugixml.a pugixml.o

REQUIRES_INC_DIRS += -Ibuild/requires/pugixml/src
REQUIRES_LIB_DIRS += -Lbuild/requires/pugixml/src
REQUIRES_LIBS += pugixml

requires-pugixml: build/requires/pugixml/src/libpugixml.a


JSONCPP_VERSION := 1.7.2

resources/jsoncpp-$(JSONCPP_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/open-source-parsers/jsoncpp/archive/$(JSONCPP_VERSION).tar.gz

build/requires/jsoncpp/dist: resources/jsoncpp-$(JSONCPP_VERSION).tar.gz
	mkdir -p build/requires
	tar xzf $< -C build/requires
	cd build/requires/ ;\
		rm -rf jsoncpp ;\
		mv -f jsoncpp-$(JSONCPP_VERSION) jsoncpp ;\
		cd jsoncpp ;\
			python amalgamate.py ;
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/jsoncpp/dist

requires-jsoncpp: build/requires/jsoncpp/dist


TIDYHTML5_VERSION := 5.1.25

resources/tidy-html5-$(TIDYHTML5_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/htacg/tidy-html5/archive/$(TIDYHTML5_VERSION).tar.gz

build/requires/tidy-html5: resources/tidy-html5-$(TIDYHTML5_VERSION).tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/tidy-html5-$(TIDYHTML5_VERSION) build/requires/tidy-html5
	touch $@

TIDYHTML5_CMAKE_FLAGS :=
ifneq ($(OS), win)
TIDYHTML5_CMAKE_FLAGS += -DCMAKE_C_FLAGS="-O2 -fPIC"
else
TIDYHTML5_CMAKE_FLAGS += -DCMAKE_C_FLAGS="-O2"
endif
build/requires/tidy-html5-built.flag: build/requires/tidy-html5
	cd build/requires/tidy-html5/build/cmake ;\
	  cmake $(TIDYHTML5_CMAKE_FLAGS) ../..
ifeq ($(OS), win)
	cd build/requires/tidy-html5/build/cmake ;\
		cmake --build . --config Release --target tidy-static
	# Under MSYS2 there are lots of multiple definition errors for localize symbols in the library
	objcopy --localize-symbols=requires/tidy-html5-localize-symbols.txt build/requires/tidy-html5/build/cmake/libtidys.a
else
	cd build/requires/tidy-html5/build/cmake ;\
		make
endif
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/tidy-html5/include
REQUIRES_LIB_DIRS += -Lbuild/requires/tidy-html5/build/cmake
REQUIRES_LIBS += tidys

requires-tidy-html5: build/requires/tidy-html5-built.flag


WEBSOCKETPP_VERSION := 0.7.0

resources/websocketpp-$(WEBSOCKETPP_VERSION).zip:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/zaphoyd/websocketpp/archive/$(WEBSOCKETPP_VERSION).zip

build/requires/websocketpp-built.flag: resources/websocketpp-$(WEBSOCKETPP_VERSION).zip
	rm -rf build/requires/websocketpp
	unzip -qo $< -d build/requires
	cd build/requires ;\
	  mv websocketpp-$(WEBSOCKETPP_VERSION) websocketpp ;\
	  touch websocketpp
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/websocketpp

requires-websocketpp: build/requires/websocketpp-built.flag

ifeq ($(OS), win)
requires-binary:
	pacman -S --noconfirm \
		   mingw-w64-x86_64-boost=1.60.0 \
		   mingw-w64-x86_64-libgit2=0.24.0 \
		   mingw-w64-x86_64-libzip=1.1.2  \
		   mingw-w64-x86_64-jsoncpp=1.7.2

REQUIRES_INC_DIRS += -I/mingw64/include -I/mingw64/lib/libzip/include/
REQUIRES_LIB_DIRS += -L/mingw64/lib

build/requires: requires-binary \
	requires-cmark \
	requires-pugixml \
	requires-tidy-html5 \
	requires-websocketpp

else
build/requires: \
	requires-boost \
	requires-cmark \
	requires-cpp-netlib \
	requires-libgit2 \
	requires-libzip \
	requires-pugixml \
	requires-jsoncpp \
	requires-tidy-html5 \
	requires-websocketpp
endif

requires: build/requires

# List of other libraries required. These are not included `libstencila.a`
OTHER_LIBS := z crypto ssl
ifeq ($(OS), linux)
	OTHER_LIBS += rt pthread curl
endif
ifeq ($(OS), osx)
	OTHER_LIBS += curl
endif
ifeq ($(OS), win)
	OTHER_LIBS += ws2_32 mswsock ssh2
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
FLAGS := --std=c++11 -O2 -Wall \
			   -Wno-unknown-pragmas -Wno-missing-braces -Wno-unused-local-typedefs \
			   -Wno-unknown-warning-option
ifneq ($(OS), win)
	FLAGS += -fPIC
endif

# Compile version.o
VERSION_TXT := build/library/generated/version.txt
VERSION_CPP := build/library/generated/version.cpp
VERSION_O := build/library/objects/version.o
$(VERSION_O):
ifneq ($(shell if [ -e "$(VERSION_TXT)" ]; then cat "$(VERSION_TXT)"; else echo ""; fi),$(VERSION))
	@mkdir -p build/library/objects
	@mkdir -p build/library/generated/
	echo "$(VERSION)" > $(VERSION_TXT)
	echo "#include <stencila/version.hpp>" > $(VERSION_CPP)
	echo "const std::string Stencila::version = \"$(VERSION)\";" >> $(VERSION_CPP)
	echo "const std::string Stencila::commit = \"$(COMMIT)\";" >> $(VERSION_CPP)
	$(CXX) $(FLAGS) -I. $(REQUIRES_INC_DIRS) -o$@ -c $(VERSION_CPP)
endif
.PHONY: $(VERSION_O)

# Compile C++ source files
build/library/objects/stencila-%.o: stencila/%.cpp
	@mkdir -p build/library/objects
	$(CXX) $(FLAGS) -I. $(REQUIRES_INC_DIRS) -o$@ -c $<

# Generate syntax parser using Lemon
build/library/generated/syntax-%-parser.cpp: stencila/syntax-%.y
	@mkdir -p $(dir $@)
	lemon $<
	mv stencila/syntax-$*.h $(dir $@)
	mv stencila/syntax-$*.c $@
	rm stencila/syntax-$*.out

# Compile syntax parser
build/library/objects/stencila-syntax-%-parser.o: build/library/generated/syntax-%-parser.cpp
	@mkdir -p $(dir $@)
	$(CXX) $(FLAGS) -Wno-unused-variable -I. $(REQUIRES_INC_DIRS) -Ibuild/library/generated -o$@ -c $<	

# Generate syntax lexer using Flex
build/library/generated/syntax-%-lexer.cpp: stencila/syntax-%.l
	@mkdir -p $(dir $@)
	flex --outfile $@ --header-file=$(dir $@)syntax-$*-lexer.hpp $<

# Compile syntax lexer
build/library/objects/stencila-syntax-%-lexer.o: build/library/generated/syntax-%-lexer.cpp build/library/generated/syntax-%-parser.cpp
	@mkdir -p $(dir $@)
	$(CXX) $(FLAGS) -Wno-deprecated-register -Wno-unused-function -I. -Ibuild/library/generated -o$@ -c $<	

# Generate the `parse()` method by using the template .cxx file
build/library/generated/syntax-%-parse.cpp: stencila/syntax-parser-parse.cxx
	@mkdir -p $(dir $@)
	sed -e 's!{lang}!$*!' -e 's!{lang-title}!\u$*!' stencila/syntax-parser-parse.cxx > $@

# Compile lexer and parser source
build/library/objects/stencila-syntax-%-parse.o: build/library/generated/syntax-%-parse.cpp \
														build/library/objects/stencila-syntax-%-lexer.o \
	 		  											build/library/objects/stencila-syntax-%-parser.o
	$(CXX) $(FLAGS) -I. $(REQUIRES_INC_DIRS) -Ibuild/library/generated -o$@ -c $<

# List of parsing related object files used in library and tests
PARSER_YS := $(notdir $(wildcard stencila/syntax-*.y))
PARSER_OS := $(patsubst syntax-%.y, build/library/objects/stencila-syntax-%-lexer.o,$(PARSER_YS)) \
				$(patsubst syntax-%.y, build/library/objects/stencila-syntax-%-parser.o,$(PARSER_YS)) \
				$(patsubst syntax-%.y, build/library/objects/stencila-syntax-%-parse.o,$(PARSER_YS))

# List of all Stencila library object files
LIBRARY_OS := $(PARSER_OS) \
					   $(patsubst %.cpp,build/library/objects/stencila-%.o,$(notdir $(wildcard stencila/*.cpp))) \
					   $(VERSION_O)
library-objects: $(LIBRARY_OS)

# Extract object files from requirement libraries
# Care may be required to ensure no name clashes in object files
define LIBRARY_EXTRACT
	mkdir -p build/library/objects/$2
	cd build/library/objects/$2 ;\
		ar x $(realpath build/requires)/$1 ;\
		for filename in *.o*; do mv $$filename ../$2-$$filename; done ;
	rm -rf build/library/objects/$2
endef

build/library/objects/requires-objects.flag: build/requires
	$(call LIBRARY_EXTRACT,boost/lib/libboost_system.a,boost-system)
	$(call LIBRARY_EXTRACT,boost/lib/libboost_filesystem.a,boost-filesystem)
	$(call LIBRARY_EXTRACT,boost/lib/libboost_regex.a,boost-regex)
	$(call LIBRARY_EXTRACT,boost/lib/libboost_thread.a,boost-thread)
	$(call LIBRARY_EXTRACT,cmark/build/src/libcmark.a,cmark)
	$(call LIBRARY_EXTRACT,cpp-netlib/libs/network/src/libcppnetlib-client-connections.a,cppnetlib-client-connections)
	$(call LIBRARY_EXTRACT,cpp-netlib/libs/network/src/libcppnetlib-uri.a,cppnetlib-uri)
	$(call LIBRARY_EXTRACT,libgit2/build/libgit2.a,git2)
	$(call LIBRARY_EXTRACT,libzip/lib/.libs/libzip.a,zip)
	$(call LIBRARY_EXTRACT,pugixml/src/libpugixml.a,pugixml)
	$(call LIBRARY_EXTRACT,tidy-html5/build/cmake/libtidys.a,tidy)
	touch $@
requires-objects: build/library/objects/requires-objects.flag

# Archive all object files (Stencila .cpp files and those extracted from requirements libraries)
# into a single static library.
# To output lists of contents for checking:
# 		ar t libstencila.a > files.txt 
#		nm -gC libstencila.a > symbols.txt  # C demangles but is an invalid option on OS X
build/library/libstencila.a: library-objects requires-objects
	cd build/library  && $(AR) rc libstencila.a `find . -name "*.o"`
library-staticlib: build/library/libstencila.a


library: library-staticlib

library-clean:
	rm -rf build/library

#################################################################################################
# Stencila C++ package
PACKAGE := stencila-$(OS)-$(ARCH)-$(VERSION).tar.gz
PACKAGE_BUILD := build/package/$(PACKAGE)

# Copy over Stencila header files
STENCILA_HPPS := $(wildcard stencila/*.hpp)
PACKAGE_HPPS := $(patsubst %.hpp,build/package/stencila/stencila/%.hpp,$(notdir $(STENCILA_HPPS)))
build/package/stencila/stencila/%.hpp: stencila/%.hpp
	@mkdir -p build/package/stencila/stencila
	cp $< $@

# Zip it up
$(PACKAGE_BUILD): $(PACKAGE_HPPS) build/library/libstencila.a
	cp build/library/libstencila.a build/package/stencila
	cd build/package ; tar czf stencila-$(OS)-$(ARCH)-$(VERSION).tar.gz stencila
package: $(PACKAGE_BUILD)

# Deliver C++ package to get.stenci.la
publish: $(PACKAGE_BUILD)
ifeq (dirty,$(DIRTY))
	$(error Publish is not done for dirty versions: $(VERSION). Commit or stash and try again.)
else
	aws s3 cp $(PACKAGE_BUILD) s3://get.stenci.la/cpp/$(PACKAGE) --cache-control max-age=31536000
endif


#################################################################################################
# Stencila C++ tests

# Compile options for tests include:
# 		-g (debug symbols)
# 		-O0 (no optimizations, so coverage is valid)
# 		--coverage (for coverage instrumentation)
TEST_COMPILE := $(CXX) $(FLAGS) -I. $(REQUIRES_INC_DIRS)
TEST_LIB_DIRS := $(REQUIRES_LIB_DIRS)
TEST_LIBS := $(REQUIRES_LIBS) $(OTHER_LIBS) boost_unit_test_framework boost_timer boost_chrono
ifeq ($(OS), linux)
TEST_COMPILE += -g -O0 --coverage
TEST_LIBS += gcov
endif
TEST_LIBS := $(patsubst %, -l%,$(TEST_LIBS))

# Compile a test file into an object file
# $(realpath $<) is used for consistency of paths in coverage reports
TEST_OS := $(patsubst %.cpp,build/tests/%.o,$(notdir $(wildcard tests/*.cpp)))
build/tests/%.o: tests/%.cpp
	@mkdir -p build/tests
	$(TEST_COMPILE) -o$@ -c $(realpath $<)

# Compile a stencila source file into an object file
# This needs to be done (instead of linking to libstencila.a) so that coverage statistics
# can be generated for these files
# $(realpath $<) is used for consistency of paths in coverage reports
TEST_STENCILA_OS := $(PARSER_OS) \
						$(patsubst %.cpp,build/tests/stencila/%.o,$(notdir $(wildcard stencila/*.cpp))) \
						$(VERSION_O)
build/tests/stencila/%.o: stencila/%.cpp
	@mkdir -p build/tests/stencila
	$(TEST_COMPILE) -o$@ -c $(realpath $<)

# Input files (typically text files) used for tests
TEST_INPUTS := build/tests/stencil-cila-html.txt \
				   build/tests/stencil-cila-render.cila \
				   build/tests/html-doc-1.html
build/tests/%: tests/%
	cp -f $< $@

# Compile a single test file into an executable
build/tests/%.exe: build/tests/%.o build/tests/tests.o $(TEST_STENCILA_OS)
	$(TEST_COMPILE) -o$@ $^ $(TEST_LIB_DIRS) $(TEST_LIBS)

# Compile all test files into an executable
build/tests/tests.exe: $(TEST_OS) $(TEST_STENCILA_OS)
	$(TEST_COMPILE) -o$@ $^ $(TEST_LIB_DIRS) $(TEST_LIBS)

# Make test executable precious so they are kept despite
# being intermediaries for test runs
.PRECIOUS: build/tests/%.exe

# Run a test
# Limit memory to prevent bugs like infinite recursion from filling up the
# machine's memory. This needs to be quite high for some tests. 2Gb = 2,097,152 kb
build/tests/%: build/tests/%.exe $(TEST_INPUTS)
	cd build/tests/ ;\
		ulimit -v 2097152; (./$(notdir $<)) || (exit 1)

# Run a single test suite by specifying in command line e.g.
# 	make test-stencil-cila
# Creates a symlink so the debugger picks this test as the one
# to debug
test-%: build/tests/%.exe $(TEST_INPUTS)
	cd build/tests/ ;\
		ln -sfT $*.exe test-to-debug ;\
		ulimit -v 2097152 ;\
		(./$*.exe) || (exit 1)

# Run quick tests only
tests-quick: build/tests/tests.exe $(TEST_INPUTS)
	cd build/tests/ ;\
		ulimit -v 2097152; (./tests.exe --run_test=*_quick/*) || (exit 1)

# Run all tests
test: tests-quick

# Run all tests and report results and coverage to XML files
# Requires python, xsltproc and [gcovr](http://gcovr.com/guide.html):
#   sudo apt-get install xsltproc
#   sudo pip install gcovr
# Use of 
#   gcovr --root . --filter='.*/stencila/.*'
# below seems to be necessary when there are different source and build directories to
# only produce coverage reports for files in 'stencila' 

# Run all tests and generate coverage stats
tests-coverage: build/tests/tests.exe
	cd build/tests ;\
	  # Run all tests \
	  ./tests.exe;\
	  # Produce coverage stats using gcovr helper for gcov \
	  gcovr --root . --filter='.*/stencila/.*'

# Run all tests and report results to Junit compatible XML files and coverage X
# to Cobertura comparible XML files
build/tests/boost-test-to-junit.xsl: tests/boost-test-to-junit.xsl
	cp $< $@
tests-xml: build/tests/tests.exe build/tests/boost-test-to-junit.xsl
	cd build/tests ;\
	  # Run all tests with reporting to XML file \
	  ./tests.exe --report_format=xml --report_level=detailed --log_format=xml --log_level=test_suite > boost-test-out.xml 2>&1 ;\
	  # Because redirecting stdout and stderr to one file need to wrap in an outer tag \
	  python -c "print '<xml>',file('boost-test-out.xml').read(),'</xml>'" > boost-test.xml ;\
	  # Convert to Junit XML format \
	  xsltproc --output junit.xml boost-test-to-junit.xsl boost-test.xml ;\
	  # Produce coverage report \
	  gcovr --root $(ROOT) --filter='.*/stencila/.*' --xml --output=coverage.xml

# Run all tests and create coverage to HTML files
# Useful for examining coverage during local development 
tests-html: build/tests/tests.exe
	cd build/tests ;\
	  # Run all tests \
	  ./tests.exe;\
	  # Produce coverage report \
	  gcovr --root $(ROOT) --filter='.*/stencila/.*' --html --html-details --output=coverage.html

tests-clean:
	rm -rf build/tests


#################################################################################################
# C++ documentation

build/docs/Doxyfile: docs/Doxyfile
	@mkdir -p build/docs
	cp $< $@

build/docs/%.css: docs/%.css
	@mkdir -p build/docs
	cp $< $@

build/docs/%.html: docs/%.html
	@mkdir -p build/docs
	cp $< $@

docs: build/docs/Doxyfile build/docs/doxy.css \
	      build/docs/doxy-header.html build/docs/doxy-footer.html
	cd build/docs ;\
	  sed -i 's!PROJECT_NUMBER = .*$$!PROJECT_NUMBER = $(VERSION)!' Doxyfile ;\
	  sed -i 's!INPUT = .*$$!INPUT = stencila/ README.md!' Doxyfile ;\
	  sed -i 's!USE_MDFILE_AS_MAINPAGE = .*$$!USE_MDFILE_AS_MAINPAGE = README.md!' Doxyfile ;\
	  doxygen Doxyfile

# Requires a branch called "gh-pages":
#	git checkout --orphan gh-pages
#	git rm -rf .
# and the "ghp-import" script
# 	sudo pip install ghp-import
docs-publish: docs
	mkdir -p build/pages/cpp
	cp -fr build/docs/html/. build/pages/cpp
	ghp-import -m "Updated pages" -p build/pages

#################################################################################################

# Remove everything except C++ requirements
scrub:
	rm -rf build/library build/tests build/docs

# Clen up build
clean:
	rm -rf build