all: cpp-test py-test r-test

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

# Clean everything
clean:
	rm -rf $(BUILD)

#################################################################################################
# C++ requirements

cpp-requires: $(BUILD)/cpp/requires
$(BUILD)/cpp/requires: cpp-requires-boost cpp-requires-libgit2 cpp-requires-pugixml cpp-requires-rapidjson cpp-requires-tidy-html5 cpp-requires-websocketpp

#################################################################################################
# Boost C++ libraries http://www.boost.org/

cpp-requires-boost: $(BUILD)/cpp/requires/boost-linked.flag

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

#################################################################################################
# libgit2 http://libgit2.github.com/

cpp-requires-libgit2: $(BUILD)/cpp/requires/libgit2-linked.flag

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

#################################################################################################
# pugixml http://pugixml.org/

cpp-requires-pugixml: $(BUILD)/cpp/requires/pugixml-linked.flag

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

#################################################################################################
# rapidjson https://code.google.com/p/rapidjson/

cpp-requires-rapidjson: $(BUILD)/cpp/requires/rapidjson-linked.flag

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

#################################################################################################
# tidy-html5 http://w3c.github.com/tidy-html5/

cpp-requires-tidy-html5: $(BUILD)/cpp/requires/tidy-html5-linked.flag

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

#################################################################################################
# WebSocket++ https://github.com/zaphoyd/websocketpp

cpp-requires-websocketpp: $(BUILD)/cpp/requires/websocketpp-linked.flag

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

#################################################################################################
# Stencila C++ library

cpp-library: cpp-library-stencila cpp-libary-staticlib

CPP_STENCILA_HPPS := $(wildcard cpp/stencila/*.hpp)
CPP_LIBRARY_HPPS := $(patsubst %.hpp,$(BUILD)/cpp/library/stencila/%.hpp,$(notdir $(CPP_STENCILA_HPPS)))

CPP_LIBRARY_CPPS := $(wildcard cpp/stencila/*.cpp)
CPP_LIBRARY_OBJECTS := $(patsubst %.cpp,$(BUILD)/cpp/library/objects/%.o,$(notdir $(CPP_LIBRARY_CPPS)))

$(BUILD)/cpp/library/layout.flag:
	mkdir -p $(BUILD)/cpp/library/stencila $(BUILD)/cpp/library/objects
	touch $@

cpp-library-stencila: $(CPP_LIBRARY_HPPS)

$(BUILD)/cpp/library/stencila/%.hpp: cpp/stencila/%.hpp $(BUILD)/cpp/library/layout.flag
	cp $< $@

cpp-libary-staticlib: $(BUILD)/cpp/library/libstencila.a

# Archive all object files and requirements libraries into a single static lib
# Output list of contents to `contents.txt` for checking
$(BUILD)/cpp/library/libstencila.a: $(CPP_LIBRARY_OBJECTS) $(BUILD)/cpp/requires
	$(AR) rc $@ $(CPP_LIBRARY_OBJECTS) 
	$(AR) t $@ > $(BUILD)/cpp/library/contents.txt

$(BUILD)/cpp/library/objects/%.o: cpp/stencila/%.cpp $(BUILD)/cpp/library/layout.flag $(BUILD)/cpp/requires
	$(CXX) --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2 -fPIC -Icpp -I$(BUILD)/cpp/requires/include -o$@ -c $<

# List of libraries to be used below
CPP_REQUIRE_LIBS += boost_filesystem boost_system boost_regex
CPP_REQUIRE_LIBS += git2 crypto ssl rt z # libgit2 requires libcrypto, libssl, librt, libz
CPP_REQUIRE_LIBS += pugixml
CPP_REQUIRE_LIBS += tidy-html5

#################################################################################################
# Stencila C++ tests

cpp-test: cpp-library


#################################################################################################
# Stencila Python package

# If PY_VERSION is not defined then get it
ifndef $(PY_VERSION)
  PY_VERSION := $(shell ./config.py py_version)
endif

PY_BUILD := $(BUILD)/py/$(PY_VERSION)

PY_PYS := $(wildcard py/stencila/*.py)
PY_CPPS := $(wildcard py/stencila/*.cpp)

PY_PACKAGE_PYS := $(patsubst %.py,$(PY_BUILD)/stencila/%.py,$(notdir $(PY_PYS)))
PY_PACKAGE_OBJECTS := $(patsubst %.cpp,$(PY_BUILD)/objects/%.o,$(notdir $(PY_CPPS)))

PY_CXX_FLAGS := --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2 -fPIC

ifeq ($(OS), linux)
  PY_INCLUDE_DIR := /usr/include/python$(PY_VERSION)
  PY_EXE := python$(PY_VERSION)
endif

PY_BOOST_PYTHON_LIB := boost_python
#ifeq $(or $(if $(OS),3.0), $(if $(OS),3.0)
#	PY_BOOST_PYTHON_LIB += 3
#endif

PY_SETUP_EXTRA_OBJECTS := $(patsubst $(PY_BUILD)/%,%,$(PY_PACKAGE_OBJECTS))
PY_SETUP_LIB_DIRS := ../../cpp/library ../../cpp/requires/lib
PY_SETUP_LIBS := $(PY_BOOST_PYTHON_LIB) python$(PY_VERSION) stencila $(CPP_REQUIRE_LIBS)

py-package: $(PY_BUILD)/setup-latest.txt

# Setup layout of Python build directory
$(PY_BUILD)/layout.flag:
	mkdir -p $(PY_BUILD)/stencila $(PY_BUILD)/objects
	touch $@

# Copy setup.py to build directory and run it from there
# Create and touch a `dummy.cpp` for setup.py to build
# Record name of the wheel to file for reading by other build tasks
$(PY_BUILD)/setup-latest.txt: py/setup.py $(PY_BUILD)/layout.flag $(PY_PACKAGE_PYS) $(PY_PACKAGE_OBJECTS)
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

$(PY_BUILD)/stencila/%.py: py/stencila/%.py $(PY_BUILD)/layout.flag
	cp $< $@

$(PY_BUILD)/objects/%.o: py/stencila/%.cpp $(PY_BUILD)/layout.flag $(BUILD)/cpp/requires
	$(CXX) $(PY_CXX_FLAGS) -Icpp -I$(BUILD)/cpp/requires/include -I$(PY_INCLUDE_DIR) -o$@ -c $<

py-test: $(PY_BUILD)/test-output.txt

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

$(PY_BUILD)/test-output.txt: py/tests/tests.py $(PY_BUILD)/test-install.flag
	cp py/tests/tests.py $(PY_BUILD)/testenv
	cd $(PY_BUILD)/testenv ;\
		. bin/activate ;\
		python tests.py 2>&1 | tee ../test-output.txt

py-clean:
	rm -rf $(PY_BUILD)

#################################################################################################
# R requirements

#################################################################################################
# Rcpp

RCPP_VERSION = 0.11.2

$(RESOURCES)/Rcpp_$(RCPP_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ http://cran.r-project.org/src/contrib/Rcpp_$(RCPP_VERSION).tar.gz
	
$(BUILD)/r/requires/Rcpp: $(RESOURCES)/Rcpp_$(RCPP_VERSION).tar.gz
	mkdir -p $(BUILD)/r/requires
	tar -xf $<
	mv Rcpp $(BUILD)/r/requires
	touch $(BUILD)/r/requires/Rcpp

$(BUILD)/r/requires/Rcpp-built.flag: $(BUILD)/r/requires/Rcpp
ifeq ($(shell Rscript -e "cat(length(grep('Rcpp',installed.packages()))>0)"), FALSE)
	R CMD INSTALL Rcpp_$(RCPP_VERSION)
	touch $@
endif

r-requires-rcpp: $(BUILD)/r/requires/Rcpp-built.flag

#################################################################################################
# RInside

RINSIDE_VERSION := 0.2.11

$(RESOURCES)/RInside_$(RINSIDE_VERSION).tar.gz:
	mkdir -p $(RESOURCES)
	wget --no-check-certificate -O$@ http://cran.r-project.org/src/contrib/RInside_$(RINSIDE_VERSION).tar.gz
	
$(BUILD)/r/requires/RInside: $(RESOURCES)/RInside_$(RINSIDE_VERSION).tar.gz
	mkdir -p $(BUILD)/r/requires
	tar -xf $<
	mv RInside $(BUILD)/r/requires
	touch $(BUILD)/r/requires/RInside

$(BUILD)/r/requires/RInside-built.flag: $(BUILD)/r/requires/RInside
ifeq ($(shell Rscript -e "cat(length(grep('RInside',installed.packages()))>0)"), FALSE)
	R CMD INSTALL RInside_$(RINSIDE_VERSION)
	touch $@
endif

r-requires-rinside: $(BUILD)/r/requires/RInside-built.flag


#################################################################################################
# Stencila R package

# If R_VERSION is not defined then get it
ifndef $(R_VERSION)
  # Version number excludes any patch number
  R_VERSION := $(shell Rscript -e "cat(R.version\$$major,strsplit(R.version\$$minor,'\\\\.')[[1]][1],sep='.')" )
endif

# Define other platform specific variables...
ifeq ($(OS),linux)
R_PACKAGE := $(VERSION).tar.gz
R_REPO_DIR := repo/src/contrib
R_REPO_TYPE := source
R_LIB_EXT := so
endif
ifeq ($(OS),msys)
R_PACKAGE := $(VERSION).zip
R_REPO_DIR := repo/bin/windows/contrib/$(R_VERSION)
R_REPO_TYPE := win.binary
R_LIB_EXT := dll
endif
# Define where the shared library gets put
R_LIB_DIR := lib/$(OS)/$(R_VERSION)
R_LIB_NAME := $(VERSION)
R_LIB_FILE := $(R_LIB_NAME).$(R_LIB_EXT)
R_LIB_ZIPFILE := $(R_LIB_FILE).zip


R_BUILD := $(BUILD)/r/$(R_VERSION)

R_RS := $(wildcard r/stencila/R/*.R)
R_CPPS := $(wildcard r/stencila/inst/src/*.cpp)

R_CXX_FLAGS := --std=c++11 -Wall -Wno-unused-local-typedefs -Wno-unused-function -O2 -fPIC

R_INCLUDE_DIR := /usr/share/R/include

R_INCLUDES := -Icpp -I$(BUILD)/cpp/requires/include \
              -I$(R_INCLUDE_DIR) \
              -I$(BUILD)/r/requires/Rcpp/inst/include \
              -I$(BUILD)/r/requires/RInside/inst/include

R_PACKAGE_OBJECTS := $(patsubst %.cpp,$(R_BUILD)/objects/%.o,$(notdir $(R_CPPS)))

r-package: $(R_BUILD)/$(R_LIB_FILE) $(R_BUILD)/layout.flag

# Setup layout of Python build directory
$(R_BUILD)/layout.flag:
	mkdir -p $(R_BUILD)/stencila $(R_BUILD)/objects
	mkdir -p $(R_BUILD)/stencila/inst
	mkdir -p $(R_BUILD)/stencila/src
	touch $@
	
$(R_BUILD)/stencila: r/stencila
	cp -r $< $@

$(R_BUILD)/$(R_LIB_FILE): $(R_PACKAGE_OBJECTS)
	$(CXX) -shared -o$@ $(R_PACKAGE_OBJECTS) $(R_LDFLAGS) $(RCPP_LDFLAGS) $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS)

$(R_BUILD)/objects/%.o: r/stencila/inst/src/%.cpp $(R_BUILD)/layout.flag $(BUILD)/cpp/requires
	$(CXX) $(R_CXX_FLAGS) $(R_INCLUDES) -o$@ -c $<

# Create package directory structure
stencila:
	mkdir -p stencila/R stencila/src

# Copy over `install.libs.R
$(R_BUILD)/stencila/src/install.libs.R: r/install.libs.R
	cp $< $@

# Create a dummy C source code file in `src`
# If there is no source files in `src` then `src\nstall.libs.R` is not run. 
$(R_BUILD)/stencila/src/dummy.c:
	touch $@

# Copy over R files
cp $(wildcard r/stencila/R/*.R) $(R_BUILD)/stencila/R
# Copy over unitTests
cp r/tests $(R_BUILD)/stencila/inst/unitTests

r-test: r-package
