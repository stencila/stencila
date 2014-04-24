all: version.hpp requires tests

.PHONY: version.hpp requires

.PRECIOUS: tests/%.exe

# Include for various STENCILA_XXX and STENCILA_CPP_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/variables.make

# Define a list of CPP files by recursively searching source directory
STENCILA_CPPS := $(shell find stencila -name "*.cpp")
# Strip off directory names and replace .cpp with .o
STENCILA_OS := $(patsubst %.cpp,builds/%.o,$(notdir $(STENCILA_CPPS)))

# Compile options include g (debug symbols), --coverage (coverage statistics) and -O0 (no optimizations, so coverage is valid) 
STENCILA_CPP_TEST_COMPILE := $(CXX) $(STENCILA_CPP_FLAGS) -g --coverage -O0 $(STENCILA_CPP_INCLUDE_DIRS)

# Set VPATH for make to search for prerequisites
# http://www.gnu.org/software/make/manual/make.html#General-Search
VPATH = stencila

show:
	@echo $(STENCILA_CPPS)
	@echo $(STENCILA_OS)

builds/%.o : stencila/%.hpp stencila/%.cpp
	$(CXX) $(STENCILA_CPP_FLAGS) $(STENCILA_CPP_INCLUDE_DIRS) -c -o$@ $<

# Make the version file
version.hpp:
	cd $(STENCILA_CPP_HOME)/stencila/; \
	cp version.hpp.template version.hpp; \
	sed -i 's!version = .*;$$!version = "$(STENCILA_VERSION)";!' version.hpp

# Make all the requirements
# Note this is a PHONY
requires:
	$(MAKE) -C requires all

# Make the builds directory
tests:
	mkdir tests

tests/%.o: stencila/%.cpp tests
	cd tests && $(CXX) $(STENCILA_CPP_FLAGS) $(STENCILA_CPP_INCLUDE_DIRS) -DSTENCILA_TEST_SINGLE -c -o ../$@ ../$<

# Compile and run an individual test suit
#   Compiled with the STENCILA_TEST_SINGLE macro defined
#   More recent versions of gcov have "--relative-only" which "Only output information about source files with a relative pathname (after source prefix elision). 
#      Absolute paths are usually system header files and coverage of any inline functions therein is normally uninteresting."
tests/%.exe: stencila/%.cpp tests
	cd tests && $(STENCILA_CPP_TEST_COMPILE) -DSTENCILA_TEST_SINGLE -o ../$@ ../$< $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS) -lboost_unit_test_framework -lgcov

# Run the test putting results in a file
tests/%.run : tests/%.exe
	./$<

clean:
	rm -rf tests








# Compile and run all test suites together
tests.exe: *.cpp requires
	$(STENCILA_CPP_TEST_COMPILE) -o tests.exe tests.cpp $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS) -lboost_unit_test_framework -lgcov


library: builds/libstencila.a

builds/libstencila.a: $(STENCILA_OS)
	$(AR) rcs $@ $(STENCILA_OS)
