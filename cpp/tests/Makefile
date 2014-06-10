all: tests.run

# Include for various STENCILA_XXX and STENCILA_CPP_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/../variables.make

# Define a list of CPP files by recursively searching source directory
STENCILA_CPPS := $(shell find ../stencila -name "*.cpp")
# Strip off directory names and replace .cpp with .o
# The object files are put in the build subdirectory
STENCILA_OS := $(patsubst %.cpp,build/%.o,$(notdir $(STENCILA_CPPS)))

# Compile options include g (debug symbols), --coverage (coverage statistics) and -O0 (no optimizations, so coverage is valid)
#   Compiled with the STENCILA_TEST_SINGLE macro defined. This gets undeffed in tests.cpp
# 	More recent versions of gcov have "--relative-only" which "Only output information about source files with a relative pathname (after source prefix elision). 
#   Absolute paths are usually system header files and coverage of any inline functions therein is normally uninteresting."
STENCILA_CPP_TEST_COMPILE := $(CXX) $(STENCILA_CPP_FLAGS) -DSTENCILA_TEST_SINGLE -g --coverage -O0 $(STENCILA_CPP_INCLUDE_DIRS)

# Create build directory for stencila object files
build:
	mkdir -p build

# Compile a stencila object file
build/%.o : ../stencila/%.cpp
	@echo $@ : $^
	$(STENCILA_CPP_TEST_COMPILE) $(STENCILA_CPP_INCLUDE_DIRS) -o $@ -c $<

# Compile a test executable
%.exe : %.cpp $(STENCILA_OS)
	@echo $@ : $^
	$(STENCILA_CPP_TEST_COMPILE) -o $@ $^ $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS) -lboost_unit_test_framework -lgcov

# Run a test executable
%.run : %.exe
	./$<

# If a test fails and the executable does not return 0
# then it is removed. We don't want that so mark them as
# precious
.PRECIOUS: %.exe build/%.o

clean:
	rm -rf build
	rm -rf *.o *.exe *.out *.gcda *.gcno