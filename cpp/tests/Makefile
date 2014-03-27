all: tests.test

include $(dir $(lastword $(MAKEFILE_LIST)))/../variables.make

requires:
	$(MAKE) -C ../lib

# Compile options include g (debug symbols), --coverage (coverage statistics) and -O0 (no optimizations, so coverage is valid) 
STENCILA_CPP_TEST_COMPILE := $(CXX) $(STENCILA_CPP_FLAGS) -g --coverage -O0 $(STENCILA_CPP_INCLUDE_DIRS)

# Compile and run all test suites together
tests.exe: *.cpp requires
	$(STENCILA_CPP_TEST_COMPILE) -o tests.exe tests.cpp $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS) -lboost_unit_test_framework -lgcov

# Compile and run an individual test suit
#   Compiled with the STENCILA_TEST_SINGLE macro defined
#   More recent versions of gcov have "--relative-only" which "Only output information about source files with a relative pathname (after source prefix elision). 
#      Absolute paths are usually system header files and coverage of any inline functions therein is normally uninteresting."
%.exe: %.cpp ../stencila/%.hpp requires
	$(STENCILA_CPP_TEST_COMPILE) -DSTENCILA_TEST_SINGLE -o $@ $< $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS) -lboost_unit_test_framework -lgcov

# Run the test
%.test : %.exe
	./$<

clean:
	rm -f *.o *.exe *.test *.gcno *.gcda *.gcov *.cov
