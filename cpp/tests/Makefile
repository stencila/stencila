all: tests.run

# Include for various STENCILA_XXX and STENCILA_CPP_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/../variables.make

# Define a header macro to 
# make it easier to read Makefile output
define header
	@tput setaf 6
	@echo $1
	@tput sgr0
endef

# Compile options include g (debug symbols), --coverage (coverage statistics) and -O0 (no optimizations, so coverage is valid)
#   Compiled with the STENCILA_TEST_SINGLE macro defined. This gets undeffed in tests.cpp
# 	More recent versions of gcov have "--relative-only" which "Only output information about source files with a relative pathname (after source prefix elision). 
#   Absolute paths are usually system header files and coverage of any inline functions therein is normally uninteresting."
STENCILA_CPP_TEST_COMPILE := $(CXX) $(STENCILA_CPP_FLAGS) -DSTENCILA_TEST_SINGLE -g --coverage -O0 $(STENCILA_CPP_INCLUDE_DIRS)

# Define a list of CPP files by recursively searching source directory
STENCILA_CPPS := $(shell find ../stencila -name "*.cpp")
# Strip off directory names and replace .cpp with .o
# The object files are put in the build subdirectory
STENCILA_OS := $(patsubst %.cpp,build/stencila/%.o,$(notdir $(STENCILA_CPPS)))

# Compile a stencila object file
# These are compiled into build/stencila to prevent names
# clashes with test object files
build/stencila/%.o : ../stencila/%.cpp
	$(call header, $@ : $^)
	mkdir -p build/stencila
	$(STENCILA_CPP_TEST_COMPILE) $(STENCILA_CPP_INCLUDE_DIRS) -o $@ -c $<

# Compile a test object file
# This is a separate step from compiling the executable to ensure that
# the gcov file "gcda" is created within the build directory
build/%.o : %.cpp
	$(call header, $@ : $^)
	mkdir -p build
	$(STENCILA_CPP_TEST_COMPILE) $(STENCILA_CPP_INCLUDE_DIRS) -o $@ -c $<

# Compile a test executable
build/%.exe : build/%.o $(STENCILA_OS)
	$(call header, $@ : $^)
	$(STENCILA_CPP_TEST_COMPILE) -o $@ $^ $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS) -lboost_unit_test_framework -lgcov

# Run a test executable
# Changes into build directory and runs tests from there
%.run : build/%.exe
	$(call header, $@ : $^)
	cd build && ../$<

# If a test fails and the executable does not return 0
# then dependencies are removed. We don't want that so mark these
# as precious
.PRECIOUS: build/%.exe build/%.o build/stencila/%.o

# Remove the build subdirectory
clean:
	rm -rf build