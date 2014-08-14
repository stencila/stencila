all: cpp-tests py-tests r-tests

# Get root directory for Stencila project
ROOT := $(realpath .)

# Get the operating system  e.g. linux
OS := $(shell ./config.py os)
# Get the machine architecture e.g i386, x86_64
ARCH := $(shell ./config.py arch)
# Get Stencila version
VERSION :=  $(shell ./config.py version)

# Build directory uses a heirarchy based on the 
# operating system and machine architecture.
BUILD := build/$(OS)/$(ARCH)/$(VERSION)

# Resources directory for downloads of dependencies
# that are independent of build
RESOURCES := build/resources

#################################################################################################
# Symbolic links to builds
# 
# Useful for automatically collecting the latest build products

.PHONY: build/current
build/current:
	@ln -sfT $(OS)/$(ARCH)/$(VERSION) build/current
build-current: build/current

#################################################################################################
# C++ requirements

BOOST_VERSION := 1_55_0

$(RESOURCES)/boost_$(BOOST_VERSION).tar.bz2:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ http://prdownloads.sourceforge.net/boost/boost_$(BOOST_VERSION).tar.bz2

$(BUILD)/cpp/requires/boost: $(RESOURCES)/boost_$(BOOST_VERSION).tar.bz2
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $@
	tar --bzip2 -xf $<
	mv boost_$(BOOST_VERSION) $@
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
#
# Under MSYS some differences are required
#	- project-config.jam must be edited to fix the [error](http://stackoverflow.com/a/5244844/1583041) produced by the above command

# Boost is configured with:
#   --with-libraries - so that only those libraries that are needed are built
BOOST_BOOTSTRAP_FLAGS := --with-libraries=filesystem,python,regex,system,test
ifeq ($(OS), msys)
	# bootstrap.sh must be called with mingw specified as toolset otherwise errors occur
	BOOST_BOOTSTRAP_FLAGS += --with-toolset=mingw
endif

# Boost is built with:
#   --prefix=.  - so that boost installs into its own directory
#   cxxflags=-fPIC - so that the statically compiled library has position independent code for use in shared libraries
#   link=static - so that get statically compiled instead of dynamically compiled libraries
BOOST_B2_FLAGS := --prefix=. cxxflags=-fPIC link=static install
ifeq ($(OS), msys)
	# b2 must be called with "system" layout of library names and header locations (otherwise it defaults to 'versioned' on Windows)
	# b2 must be called with "release" build otherwise defaults to debug AND release, which with "system" causes an 
	#   error (http://boost.2283326.n4.nabble.com/atomic-building-with-layout-system-mingw-bug-7482-td4640920.html)
	BOOST_B2_FLAGS += --layout=system release toolset=gcc
endif

$(BUILD)/cpp/requires/boost-built.flag: $(BUILD)/cpp/requires/boost
	cd $< ;\
	  ./bootstrap.sh $(BOOST_BOOTSTRAP_FLAGS) ;\
	  ./b2 $(BOOST_B2_FLAGS)
	touch $@

$(BUILD)/cpp/requires/boost-linked.flag: $(BUILD)/cpp/requires/boost-built.flag
	mkdir -p $(BUILD)/cpp/requires/include $(BUILD)/cpp/requires/lib
	cd $(BUILD)/cpp/requires ;\
	  ln -sfT ../boost/include/boost include/boost ;\
	  for file in $$(ls boost/lib/*.a); do ln -sf ../$$file lib; done
	touch $@

cpp-requires-boost: $(BUILD)/cpp/requires/boost-linked.flag


LIBGIT2_VERSION := 0.20.0

$(RESOURCES)/libgit2-$(LIBGIT2_VERSION).zip:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/libgit2/libgit2/archive/v$(LIBGIT2_VERSION).zip

$(BUILD)/cpp/requires/libgit2: $(RESOURCES)/libgit2-$(LIBGIT2_VERSION).zip
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $@
	unzip -qo $<
	mv libgit2-$(LIBGIT2_VERSION) $@
	touch $@

$(BUILD)/cpp/requires/libgit2-built.flag: $(BUILD)/cpp/requires/libgit2
	cd $< ;\
	  mkdir -p build ;\
	  cd build ;\
	  cmake .. -DCMAKE_C_FLAGS=-fPIC -DBUILD_SHARED_LIBS=OFF ;\
	  cmake --build .
	touch $@

$(BUILD)/cpp/requires/libgit2-linked.flag: $(BUILD)/cpp/requires/libgit2-built.flag
	mkdir -p $(BUILD)/cpp/requires/include $(BUILD)/cpp/requires/lib
	cd $(BUILD)/cpp/requires ;\
	  ln -sfT ../libgit2/include/git2.h include/git2.h ;\
	  ln -sfT ../libgit2/include/git2 include/git2 ;\
	  ln -sfT ../libgit2/build/libgit2.a lib/libgit2.a
	touch $@

cpp-requires-libgit2: $(BUILD)/cpp/requires/libgit2-linked.flag


PUGIXML_VERSION := 1.2

$(RESOURCES)/pugixml-$(PUGIXML_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ http://pugixml.googlecode.com/files/pugixml-$(PUGIXML_VERSION).tar.gz

$(BUILD)/cpp/requires/pugixml: $(RESOURCES)/pugixml-$(PUGIXML_VERSION).tar.gz
	mkdir -p $@
	cp $< $@
	cd $@ && tar xzf pugixml-$(PUGIXML_VERSION).tar.gz

$(BUILD)/cpp/requires/pugixml-built.flag: $(BUILD)/cpp/requires/pugixml
	cd $</src ;\
	  $(CXX) -O2 -fPIC -c pugixml.cpp ;\
	  $(AR) rcs libpugixml.a pugixml.o
	touch $@

$(BUILD)/cpp/requires/pugixml-linked.flag: $(BUILD)/cpp/requires/pugixml-built.flag
	mkdir -p $(BUILD)/cpp/requires/include $(BUILD)/cpp/requires/lib
	cd $(BUILD)/cpp/requires ;\
	  ln -sfT ../pugixml/src/pugixml.hpp include/pugixml.hpp ;\
	  ln -sfT ../pugixml/src/pugiconfig.hpp include/pugiconfig.hpp ;\
	  ln -sfT ../pugixml/src/libpugixml.a lib/libpugixml.a
	touch $@

cpp-requires-pugixml: $(BUILD)/cpp/requires/pugixml-linked.flag


RAPIDJSON_VERSION := 0.11

# There are several forks of rapidjson on Github
# At the time of writing the ones that appeared to be most worthwhile watching were:
# 
# 	- https://github.com/pah/rapidjson
# 	- https://github.com/miloyip/rapidjson/issues/1
# 
$(RESOURCES)/rapidjson-$(RAPIDJSON_VERSION).zip:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ http://rapidjson.googlecode.com/files/rapidjson-$(RAPIDJSON_VERSION).zip

$(BUILD)/cpp/requires/rapidjson: $(RESOURCES)/rapidjson-$(RAPIDJSON_VERSION).zip
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $@
	unzip -qo $< -d $(BUILD)/cpp/requires
	touch $@

# Apply patch from https://github.com/scanlime/rapidjson/commit/0c69df5ac098640018d9232ae71ed1036c692187
# that allows for copying of Documents [rapidjson by default prevents copying 
# of documents](http://stackoverflow.com/questions/22707814/perform-a-copy-of-document-object-of-rapidjson)
$(BUILD)/cpp/requires/rapidjson/include/rapidjson/document.h: cpp/requires/rapidjson-scanlime-0c69df5ac0.patch $(BUILD)/cpp/requires/rapidjson
	cat $< | patch -d $(BUILD)/cpp/requires/rapidjson/include/rapidjson

$(BUILD)/cpp/requires/rapidjson-linked.flag: $(BUILD)/cpp/requires/rapidjson $(BUILD)/cpp/requires/rapidjson/include/rapidjson/document.h
	mkdir -p $(BUILD)/cpp/requires/include
	cd $(BUILD)/cpp/requires ;\
	  ln -sfT ../rapidjson/include/rapidjson include/rapidjson
	touch $@

cpp-requires-rapidjson: $(BUILD)/cpp/requires/rapidjson-linked.flag


$(RESOURCES)/tidy-html5-master.zip:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/w3c/tidy-html5/archive/master.zip

$(BUILD)/cpp/requires/tidy-html5-unpacked.flag: $(RESOURCES)/tidy-html5-master.zip
	mkdir -p $(BUILD)/cpp/requires
	rm -rf $(BUILD)/cpp/requires/tidy-html5
	unzip -qo $< -d $(BUILD)/cpp/requires
	mv $(BUILD)/cpp/requires/tidy-html5-master $(BUILD)/cpp/requires/tidy-html5
	touch $(BUILD)/cpp/requires/tidy-html5-unpacked.flag

# These patches depend upon `tidy-html5-unpacked.flag` rather than simply the `tidy-html5` since that
# directory's time changes with the patches and so they keep getting applied

# Apply patch to Makefile to add -O3 -fPIC options
$(BUILD)/cpp/requires/tidy-html5/build/gmake/Makefile: cpp/requires/tidy-html5-build-gmake-Makefile.patch $(BUILD)/cpp/requires/tidy-html5-unpacked.flag
	patch $@ $<

# Apply patch from pull request #98 to add <main> tag (this is applied using `patch` rather than `git` so that `git` is not required)
# This patch affects include/tidyenum.h, src/attrdict.h, src/attrdict.c, src/tags.c
$(BUILD)/cpp/requires/tidy-html5/include/tidyenum.h: cpp/requires/tidy-html5-pull-98.patch $(BUILD)/cpp/requires/tidy-html5-unpacked.flag
	cat $< | patch -p1 -d $(BUILD)/cpp/requires/tidy-html5

# Apply patch to prevent linker error associated with "GetFileSizeEx" on MSYS
$(BUILD)/cpp/requires/tidy-html5/src/mappedio.c: cpp/requires/tidy-html5-src-mappedio.c.patch $(BUILD)/cpp/requires/tidy-html5-unpacked.flag
	patch $@ $<

# Note that we only "make ../../lib/libtidy.a" and not "make all" because the latter is not required
$(BUILD)/cpp/requires/tidy-html5-built.flag: \
		$(BUILD)/cpp/requires/tidy-html5/build/gmake/Makefile \
		$(BUILD)/cpp/requires/tidy-html5/include/tidyenum.h \
		$(BUILD)/cpp/requires/tidy-html5/src/mappedio.c
	cd $(BUILD)/cpp/requires/tidy-html5/build/gmake ;\
	  make ../../lib/libtidy.a
	touch $@

$(BUILD)/cpp/requires/tidy-html5-linked.flag: $(BUILD)/cpp/requires/tidy-html5-built.flag
	mkdir -p $(BUILD)/cpp/requires/include $(BUILD)/cpp/requires/lib
	cd $(BUILD)/cpp/requires ;\
	  ln -sfT ../tidy-html5/include include/tidy-html5 ;\
	  ln -sfT ../tidy-html5/lib/libtidy.a lib/libtidy-html5.a
	touch $@

cpp-requires-tidy-html5: $(BUILD)/cpp/requires/tidy-html5-linked.flag


WEBSOCKETPP_VERSION := 0.3.0-alpha4

$(RESOURCES)/websocketpp-$(WEBSOCKETPP_VERSION).zip:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O $@ https://github.com/zaphoyd/websocketpp/archive/$(WEBSOCKETPP_VERSION).zip

$(BUILD)/cpp/requires/websocketpp-linked.flag: $(RESOURCES)/websocketpp-$(WEBSOCKETPP_VERSION).zip
	mkdir -p $(BUILD)/cpp/requires/include
	rm -rf $(BUILD)/cpp/requires/websocketpp
	unzip -qo $< -d $(BUILD)/cpp/requires
	cd $(BUILD)/cpp/requires ;\
	  mv websocketpp-$(WEBSOCKETPP_VERSION) websocketpp ;\
	  touch websocketpp ;\
	  ln -sfT ../websocketpp/websocketpp include/websocketpp
	touch $@

cpp-requires-websocketpp: $(BUILD)/cpp/requires/websocketpp-linked.flag

# List of libraries to be used below
CPP_REQUIRE_LIBS += boost_filesystem boost_system boost_regex
CPP_REQUIRE_LIBS += git2 crypto ssl rt z # libgit2 requires libcrypto, libssl, librt, libz
CPP_REQUIRE_LIBS += pugixml
CPP_REQUIRE_LIBS += tidy-html5

$(BUILD)/cpp/requires: cpp-requires-boost cpp-requires-libgit2 cpp-requires-pugixml \
   cpp-requires-rapidjson cpp-requires-tidy-html5 cpp-requires-websocketpp

cpp-requires: $(BUILD)/cpp/requires

#################################################################################################
# Stencila C++ library

CPP_STENCILA_HPPS := $(wildcard cpp/stencila/*.hpp)
CPP_LIBRARY_HPPS := $(patsubst %.hpp,$(BUILD)/cpp/library/stencila/%.hpp,$(notdir $(CPP_STENCILA_HPPS)))
$(BUILD)/cpp/library/stencila/%.hpp: cpp/stencila/%.hpp
	@mkdir -p $(BUILD)/cpp/library/stencila
	cp $< $@
cpp-library-stencila: $(CPP_LIBRARY_HPPS)


CPP_LIBRARY_CPPS := $(wildcard cpp/stencila/*.cpp)
CPP_LIBRARY_OBJECTS := $(patsubst %.cpp,$(BUILD)/cpp/library/objects/%.o,$(notdir $(CPP_LIBRARY_CPPS)))
$(BUILD)/cpp/library/objects/%.o: cpp/stencila/%.cpp $(BUILD)/cpp/requires
	@mkdir -p $(BUILD)/cpp/library/objects
	$(CXX) --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2 -fPIC -Icpp -I$(BUILD)/cpp/requires/include -o$@ -c $<

# Archive all object files and requirements libraries into a single static lib
# Output list of contents to `contents.txt` for checking
$(BUILD)/cpp/library/libstencila.a: $(CPP_LIBRARY_OBJECTS) $(BUILD)/cpp/requires
	$(AR) rc $@ $(CPP_LIBRARY_OBJECTS) 
	$(AR) t $@ > $(BUILD)/cpp/library/contents.txt
cpp-libary-staticlib: $(BUILD)/cpp/library/libstencila.a

cpp-library: cpp-library-stencila cpp-libary-staticlib


#################################################################################################
# Stencila C++ tests

# Compile options for tests include:
# 		-g (debug symbols),
# 		-fprofile-arcs -ftest-coverage (coverage statistics)
# 		-O0 (no optimizations, so coverage is valid)
CPP_TEST_COMPILE := $(CXX) --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function \
                       -g -fprofile-arcs -ftest-coverage -fPIC -O0 -Icpp -I$(BUILD)/cpp/requires/include

CPP_TEST_LIBDIRS := $(BUILD)/cpp/requires/lib
CPP_TEST_LIBDIRS := $(patsubst %, -L%,$(CPP_TEST_LIBDIRS))

CPP_TEST_LIBS := $(CPP_REQUIRE_LIBS) boost_unit_test_framework gcov
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
CPP_TEST_STENCILA_OS := $(patsubst %.cpp,$(BUILD)/cpp/tests/stencila/%.o,$(notdir $(wildcard cpp/stencila/*.cpp)))
$(BUILD)/cpp/tests/stencila/%.o: cpp/stencila/%.cpp
	@mkdir -p $(BUILD)/cpp/tests/stencila
	$(CPP_TEST_COMPILE) -o$@ -c $(realpath $<)

# Compile a single test file into an executable
$(BUILD)/cpp/tests/%.exe: $(BUILD)/cpp/tests/%.o $(BUILD)/cpp/tests/tests.o $(CPP_TEST_STENCILA_OS) $(BUILD)/cpp/requires
	$(CPP_TEST_COMPILE) -o$@ $< $(BUILD)/cpp/tests/tests.o $(CPP_TEST_STENCILA_OS) $(CPP_TEST_LIBDIRS) $(CPP_TEST_LIBS)

# Compile all test files into an executable
$(BUILD)/cpp/tests/tests.exe: $(CPP_TEST_OS) $(CPP_TEST_STENCILA_OS) $(BUILD)/cpp/requires
	$(CPP_TEST_COMPILE) -o$@ $(CPP_TEST_OS) $(CPP_TEST_STENCILA_OS) $(CPP_TEST_LIBDIRS) $(CPP_TEST_LIBS)

# Run a test
# Limit memory to prevent bugs like infinite recursion from filling up the
# machine's memeory
$(BUILD)/cpp/tests/%.run: $(BUILD)/cpp/tests/%.exe
	ulimit -v 100000; $< 2>&1 | tee $(BUILD)/cpp/tests/%.out

# Run a single test suite by specifying in command line e.g.
# 	make cpp-test CPP_TEST=stencil-cila
ifndef CPP_TEST
  CPP_TEST := tests
endif
cpp-test: $(BUILD)/cpp/tests/$(CPP_TEST).run

# Run all tests
cpp-tests: $(BUILD)/cpp/tests/tests.run

# Run all tests and report results and coverage to XML files
# Useful for integration with CI systems like Jenkins
# Requires python, xsltproc and [gcovr](http://gcovr.com/guide.html):
#   sudo apt-get install xsltproc
#   sudo pip install gcovr
# Use of 
#   gcovr --root $(ROOT) --filter='.*/cpp/stencila/.*'
# below seems to be necessary when there are different source and build directories to
# only produce coverage reports for files in 'cpp/stencila' 
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

# Run all tests and report results and coverage to HTML files
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
	  sed -i 's!INPUT = .*$$!INPUT = $(ROOT)/cpp/stencila/!' Doxyfile ;\
	  doxygen Doxyfile

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
PY_SETUP_LIB_DIRS := ../../cpp/library ../../cpp/requires/lib
PY_SETUP_LIBS := $(PY_BOOST_PYTHON_LIB) python$(PY_VERSION) stencila $(CPP_REQUIRE_LIBS)

$(PY_BUILD)/stencila/%.py: py/stencila/%.py
	@mkdir -p $(PY_BUILD)/stencila
	cp $< $@

$(PY_BUILD)/objects/%.o: py/stencila/%.cpp $(BUILD)/cpp/requires
	@mkdir -p $(PY_BUILD)/objects
	$(CXX) $(PY_CXX_FLAGS) -Icpp -I$(BUILD)/cpp/requires/include -I$(PY_INCLUDE_DIR) -o$@ -c $<

# Copy setup.py to build directory and run it from there
# Create and touch a `dummy.cpp` for setup.py to build
# Record name of the wheel to file for reading by other build tasks
$(PY_BUILD)/setup-latest.txt: py/setup.py $(PY_PACKAGE_PYS) $(PY_PACKAGE_OBJECTS)
	cp py/setup.py $(PY_BUILD)
	cd $(PY_BUILD)/ ;\
		export \
			VERSION=$(VERSION) \
			EXTRA_OBJECTS='$(PY_SETUP_EXTRA_OBJECTS)' \
			LIBRARY_DIRS='$(PY_SETUP_LIB_DIRS)' \
			LIBRARIES='$(PY_SETUP_LIBS)' ;\
		touch dummy.cpp ;\
		$(PY_EXE) setup.py bdist_wheel ;\
		echo `ls -rt dist/*.whl | tail -n1` > setup-latest.txt

py-package: $(PY_BUILD)/setup-latest.txt

# Create a virtual environment to be used for testing with the Python version
# Using a virtual environment allows the Stencila wheel to be installed locally,
# i.e. without root privalages, and also does not affect the host machines Python setup 
$(PY_BUILD)/testenv/bin/activate:
	cd $(PY_BUILD) ;\
		virtualenv --python=python$(PY_VERSION) --no-site-packages testenv

$(PY_BUILD)/test-install.flag: $(PY_BUILD)/testenv/bin/activate $(PY_BUILD)/setup-latest.txt
	cd $(PY_BUILD) ;\
		. testenv/bin/activate ;\
		pip install --upgrade --force-reinstall `cat setup-latest.txt`
	touch $@

$(PY_BUILD)/tests.out: py/tests/tests.py $(PY_BUILD)/test-install.flag
	cp py/tests/tests.py $(PY_BUILD)/testenv
	cd $(PY_BUILD)/testenv ;\
		. bin/activate ;\
		python tests.py 2>&1 | tee ../tests.out

py-tests: $(PY_BUILD)/tests.out

py-clean:
	rm -rf $(PY_BUILD)

#################################################################################################
# R requirements

RCPP_VERSION = 0.11.2

$(RESOURCES)/Rcpp_$(RCPP_VERSION).tar.gz:
	@mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ http://cran.r-project.org/src/contrib/Rcpp_$(RCPP_VERSION).tar.gz
	
$(BUILD)/r/requires/Rcpp: $(RESOURCES)/Rcpp_$(RCPP_VERSION).tar.gz
	@mkdir -p $@
	R CMD INSTALL -l $(BUILD)/r/requires $<
r-requires-rcpp: $(BUILD)/r/requires/Rcpp


RINSIDE_VERSION := 0.2.11

$(RESOURCES)/RInside_$(RINSIDE_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ http://cran.r-project.org/src/contrib/RInside_$(RINSIDE_VERSION).tar.gz
	
$(BUILD)/r/requires/RInside: $(RESOURCES)/RInside_$(RINSIDE_VERSION).tar.gz
	@mkdir -p $@
	R CMD INSTALL -l $(BUILD)/r/requires $<
r-requires-rinside: $(BUILD)/r/requires/RInside


$(BUILD)/r/requires: $(BUILD)/r/requires/Rcpp $(BUILD)/r/requires/RInside
r-requires: $(BUILD)/r/requires

#################################################################################################
# Stencila R package

# If R_VERSION is not defined then get it
ifndef R_VERSION
  # Version number excludes any patch number
  R_VERSION := $(shell Rscript -e "cat(R.version\$$major,strsplit(R.version\$$minor,'\\\\.')[[1]][1],sep='.')" )
endif

# Define R platform
# Note in the below the double $ is to escape make's treatment of $
# and the \$ is to escape the shell's treatment of $
R_PLATFORM := $(shell Rscript -e "cat(R.version\$$platform)" )

# The R version can not include the '-dev' tag
R_PACKAGE_VERSION := $(subst -dev,,$(VERSION))

# Define other platform specific variables...
ifeq ($(OS),linux)
R_PACKAGE_EXT := tar.gz
R_DYNLIB_EXT := so
endif
ifeq ($(OS),msys)
R_PACKAGE_EXT := zip
R_DYNLIB_EXT := dll
endif
# Define where the shared library gets put
R_DYNLIB_NAME := stencila_$(R_PACKAGE_VERSION)
R_DYNLIB := $(R_DYNLIB_NAME).$(R_DYNLIB_EXT)

R_BUILD := $(BUILD)/r/$(R_VERSION)

# Print R related Makefile variables; useful for debugging
r-vars:
	@echo R_VERSION : $(R_VERSION)
	@echo R_PLATFORM : $(R_PLATFORM)
	@echo R_PACKAGE_VERSION : $(R_PACKAGE_VERSION)
	@echo R_DYNLIB : $(R_DYNLIB)

# Compile each cpp file
R_PACKAGE_OBJECTS := $(patsubst %.cpp,$(R_BUILD)/objects/%.o,$(notdir $(wildcard r/stencila/*.cpp)))
R_CXX_FLAGS := --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2 -fPIC
R_INCLUDE_DIR := /usr/share/R/include
R_INCLUDES := -Icpp -I$(BUILD)/cpp/requires/include \
              -I$(R_INCLUDE_DIR) \
              -I$(BUILD)/r/requires/Rcpp/include
$(R_BUILD)/objects/%.o: r/stencila/%.cpp $(BUILD)/cpp/requires $(BUILD)/r/requires
	@mkdir -p $(R_BUILD)/objects
	$(CXX) $(R_CXX_FLAGS) $(R_INCLUDES) -o$@ -c $<
	
# Create shared library
R_DYNLIB_LIB_DIRS := $(BUILD)/cpp/library $(BUILD)/cpp/requires/lib
R_DYNLIB_LIBS := stencila $(CPP_REQUIRE_LIBS) 
$(R_BUILD)/$(R_DYNLIB): $(R_PACKAGE_OBJECTS) $(BUILD)/cpp/library/libstencila.a
	$(CXX) -shared -o$@ $^ $(patsubst %, -L%,$(R_DYNLIB_LIB_DIRS)) $(patsubst %, -l%,$(R_DYNLIB_LIBS))

# Place zippled up shared library in package
R_PACKAGE_LIBZIP := $(R_BUILD)/stencila/inst/lib/$(R_PLATFORM)/$(R_VERSION)/$(R_DYNLIB).zip
$(R_PACKAGE_LIBZIP): $(R_BUILD)/$(R_DYNLIB)
	@mkdir -p $(R_BUILD)/stencila/inst/lib/$(R_PLATFORM)/$(R_VERSION)
	rm -f $@
	zip -j $@ $<

# Copy over `install.libs.R
R_PACKAGE_INSTALLSCRIPT := $(R_BUILD)/stencila/src/install.libs.R
$(R_PACKAGE_INSTALLSCRIPT): r/install.libs.R
	@mkdir -p $(R_BUILD)/stencila/src/
	cp $< $@

# Create a dummy C source code file in `src`
# If there is no source files in `src` then `src\nstall.libs.R` is not run. 
R_PACKAGE_DUMMYC := $(R_BUILD)/stencila/src/dummy.c
$(R_PACKAGE_DUMMYC):
	@mkdir -p $(R_BUILD)/stencila/src/
	touch $@

# Copy over each R file
R_PACKAGE_RS := $(patsubst %, $(R_BUILD)/stencila/R/%, $(notdir $(wildcard r/stencila/*.R)))
$(R_BUILD)/stencila/R/%.R: r/stencila/%.R
	@mkdir -p $(R_BUILD)/stencila/R
	cp $< $@

# Copy over each unit test file
R_PACKAGE_TESTS := $(patsubst %, $(R_BUILD)/stencila/inst/unitTests/%, $(notdir $(wildcard r/tests/*.R)))
$(R_BUILD)/stencila/inst/unitTests/%.R: r/tests/%.R
	@mkdir -p $(R_BUILD)/stencila/inst/unitTests
	cp $< $@

# Copy over DESCRIPTION
R_PACKAGE_DESC := $(R_BUILD)/stencila/DESCRIPTION
$(R_PACKAGE_DESC): r/DESCRIPTION
	cp $< $@

# Finalise the package directory
R_PACKAGE_DATE := $(shell date --utc +%Y-%m-%dT%H:%M:%SZ)
$(R_BUILD)/stencila: $(R_PACKAGE_LIBZIP) $(R_PACKAGE_INSTALLSCRIPT) $(R_PACKAGE_DUMMYC) $(R_PACKAGE_RS) $(R_PACKAGE_TESTS) $(R_PACKAGE_DESC)
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
	# Add `useDynLib` to the NAMESPACE file (after roxygensiation) so that
	# the dynamic library is loaded
	echo "useDynLib($(R_DYNLIB_NAME))" >> $(R_BUILD)/stencila/NAMESPACE
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
R_PACKAGE_FILE := stencila_$(R_PACKAGE_VERSION).$(R_PACKAGE_EXT)
$(R_BUILD)/$(R_PACKAGE_FILE): $(R_BUILD)/stencila
ifeq ($(OS),linux)
	cd $(R_BUILD); R CMD build stencila
endif
ifeq ($(OS),msys)
	cd $(R_BUILD); R CMD INSTALL --build stencila
endif
r-package: $(R_BUILD)/$(R_PACKAGE_FILE)

# Test the package by running unit tests
# Install package in a testenv directory and run unit tests from there
# This is better than installing package in the user's R library location
r-tests: $(R_BUILD)/$(R_PACKAGE_FILE)
	cd $(R_BUILD) ;\
	  mkdir -p testenv ;\
	  R CMD INSTALL -l testenv $(R_PACKAGE_FILE) ;\
	  cd testenv ;\
	    Rscript -e "library(stencila,lib.loc='.'); setwd('stencila/unitTests/'); source('do-svUnit.R')" 2>&1 | tee ../tests.out

# Install R on the local host
# Not intended for development but rather 
# to install on the host machine after a build
r-install: $(R_BUILD)/$(R_PACKAGE_FILE)
	R CMD INSTALL $(R_BUILD)/$(R_PACKAGE_FILE)

#################################################################################################

# Clean everything!
clean:
	rm -rf $(BUILD)