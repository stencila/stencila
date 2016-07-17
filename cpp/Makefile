include ../shared.make

all: setup build cover

.PHONY: setup build test cover clean

setup:
	$(MAKE) -f requires/boost.make boost
	$(MAKE) -f requires/cmark.make cmark
	$(MAKE) -f requires/jsoncpp.make jsoncpp
	$(MAKE) -f requires/libgit2.make libgit2
	$(MAKE) -f requires/libzip.make libzip
#	$(MAKE) -f requires/cpr.make cpr
	$(MAKE) -f requires/pugixml.make pugixml
	$(MAKE) -f requires/tidy-html5.make tidy-html5
	$(MAKE) -f requires/websocketpp.make websocketpp

build:
	cd build && cmake .. && cmake --build .

test:
ifeq ($(OS), linux)
	cd build/tests && ulimit -v 2097152 && (./tests --run_test="*_quick/*") || (exit 1)
endif

cover: test

clean:
	rm -rf build






# List of other libraries required. These are not included `libstencila.a`
OTHER_LIBS := z curl ssl crypto
ifeq ($(OS), linux)
	OTHER_LIBS += rt pthread
endif
ifeq ($(OS), osx)
	OTHER_LIBS += iconv
endif
ifeq ($(OS), win)
	OTHER_LIBS += ws2_32 mswsock ssh2
endif

FLAGS := --std=c++11 -O2 -Wall \
			   -Wno-unknown-pragmas -Wno-missing-braces -Wno-unused-local-typedefs \
			   -Wno-unknown-warning-option
ifneq ($(OS), win)
	FLAGS += -fPIC
endif

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

TEST_LDFLAGS :=
ifeq ($(OS), osx)
TEST_LDFLAGS += -Wl,-framework -Wl,CoreFoundation -Wl,-framework -Wl,Security
endif

# Compile a test file into an object file
# $(realpath $<) is used for consistency of paths in coverage reports
TEST_OS := $(patsubst %.cpp,build/tests/%.o,$(notdir $(wildcard tests/*.cpp)))
build/tests/%.o: tests/%.cpp
	@mkdir -p build/tests
	$(CXX) $(FLAGS) -I. $(REQUIRES_INC_DIRS) -o$@ -c $(realpath $<)

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
	$(CXX) $(FLAGS) -I. $(REQUIRES_INC_DIRS) -o$@ $^ $(TEST_LIB_DIRS) $(TEST_LIBS) $(TEST_LDFLAGS)

# Compile all test files into an executable
build/tests/tests.exe: $(TEST_OS) $(TEST_STENCILA_OS)
	$(CXX) $(FLAGS) -I. $(REQUIRES_INC_DIRS) -o$@ $^ $(TEST_LIB_DIRS) $(TEST_LIBS) $(TEST_LDFLAGS)

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
