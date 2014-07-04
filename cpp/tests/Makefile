all: tests.run

# Include for various STENCILA_XXX and STENCILA_CPP_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/../variables.make

# Define a header macro to make it easier to read Makefile output
# Previous version set colours using tput that failed when 
# using with SublimeText
define header
	@echo -------------------------------------------------------
	@echo $1
	@echo -------------------------------------------------------
endef

# Compile options include g (debug symbols), --coverage (coverage statistics) and -O0 (no optimizations, so coverage is valid)
# 	More recent versions of gcov have "--relative-only" which "Only output information about source files with a relative pathname (after source prefix elision). 
#   Absolute paths are usually system header files and coverage of any inline functions therein is normally uninteresting."
STENCILA_CPP_COMPILE_TEST := $(CXX) $(STENCILA_CPP_FLAGS) -g --coverage -O0 $(STENCILA_CPP_INCS)
STENCILA_CPP_LIBS_TEST := $(STENCILA_CPP_LIBS) -lboost_unit_test_framework -lgcov

# Define a list of Stencila C++ library files by searching source directory
STENCILA_CPPS := $(shell find ../stencila -name "*.cpp")
# Strip off directory names and replace .cpp with .o in the build subdirectory
STENCILA_OS := $(patsubst %.cpp,build/stencila/%.o,$(notdir $(STENCILA_CPPS)))

# Define a list of test files in this directory
TEST_CPPS := $(shell find -name "*.cpp")
# Strip off directory names and replace .cpp with .o (in the build subdirectory)
TEST_OS := $(patsubst %.cpp,build/%.o,$(notdir $(TEST_CPPS)))

# Compile a stencila object file
# These are compiled into build/stencila to prevent name
# clashes with test object files
build/stencila/%.o : ../stencila/%.cpp
	$(call header, $@ : $^)
	mkdir -p build/stencila
	$(STENCILA_CPP_COMPILE_TEST) -o $@ -c $<

# Compile a test object file
# This is a separate step from compiling the executable to ensure that
# the gcov file "gcda" is created within the build directory
build/%.o : %.cpp
	$(call header, $@ : $^)
	mkdir -p build
	$(STENCILA_CPP_COMPILE_TEST) -c -o $@ $<

# Compile PythonContext test
build/python-context.o : python-context.cpp
	$(STENCILA_CPP_COMPILE_TEST) $(STENCILA_CPP_FLAGS_PY) $(STENCILA_CPP_INCS_PY) -c -o $@ $^

build/python-context.exe : build/python-context.o build/tests.o $(STENCILA_OS)
	$(STENCILA_CPP_COMPILE_TEST) $(STENCILA_CPP_FLAGS_PY) -o $@ $^ $(STENCILA_CPP_LIDS) $(STENCILA_CPP_LIDS_PY) $(STENCILA_CPP_LIBS_TEST) $(STENCILA_CPP_LIBS_PY)

# Compile RContext test
build/r-context.o : r-context.cpp
	$(STENCILA_CPP_COMPILE_TEST) $(STENCILA_CPP_FLAGS_R) $(STENCILA_CPP_INCS_R) -c -o $@ $^

build/r-context.exe : build/r-context.o build/tests.o $(STENCILA_OS)
	$(STENCILA_CPP_COMPILE_TEST) $(STENCILA_CPP_FLAGS_R) -o $@ $^ $(STENCILA_CPP_LIDS) $(STENCILA_CPP_LIDS_R) $(STENCILA_CPP_LIBS_TEST) $(STENCILA_CPP_LIBS_R)


# Compile a single test file into an executable
build/%.exe : build/tests.o build/%.o $(STENCILA_OS)
	$(call header, $@ : $^)
	$(STENCILA_CPP_COMPILE_TEST) -o $@ $^ $(STENCILA_CPP_LIDS) $(STENCILA_CPP_LIBS_TEST)

# Compile all test files into an executable
build/tests.exe : $(TEST_OS) $(STENCILA_OS)
	$(call header, $@ : $^)
	$(STENCILA_CPP_COMPILE_TEST) -o $@ $^ $(STENCILA_CPP_LIDS) $(STENCILA_CPP_LIBS_TEST)

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