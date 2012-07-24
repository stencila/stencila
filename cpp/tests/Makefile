include ../Makefile.inc

all: tests.out

# Compile options include g (debug symbols), --coverage (coverage statistics) and -O0 (no optimizations, so coverage is valid) 
CPP_TEST_COMPILE := g++ -std=c++0x -Wall -g --coverage -O0

# Compile and run all test suites together
tests.exe: tests.cpp dataset.cpp datatable.cpp dataquery.cpp
	$(CPP_TEST_COMPILE) -o tests.exe tests.cpp $(CPP_LIBS) -lboost_unit_test_framework -lgcov

# Compile and run an individual test suit
#   Assumed that only dependency is the corresponding header file in the parent directory.
#   Compiled with the STENCILA_TEST_SINGLE macro defined
#   More recent versions of gcov have "--relative-only" which "Only output information about source files with a relative pathname (after source prefix elision). 
#      Absolute paths are usually system header files and coverage of any inline functions therein is normally uninteresting."
%.exe : %.cpp ../%.hpp
	$(CPP_TEST_COMPILE) -DSTENCILA_TEST_SINGLE -o $@ $< $(CPP_LIBS) -lboost_unit_test_framework -lgcov
	
# Run the test
%.out : %.exe
	./$<
	
clean:
	rm -f *.o *.exe *.out *.gcno *.gcda *.gcov *.cov