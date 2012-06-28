all: tests.out

# Compile options include -O0 (no optimizations) and --coverage so that coverage statistics are valid
compile := g++ -std=c++0x -Wall -O0 --coverage
libs := -lsqlite3 -lboost_system -lboost_filesystem -lboost_unit_test_framework -lgcov

# Compile and run all test suites together
tests.exe: tests.cpp dataset.cpp datatable.cpp dataquery.cpp
	$(compile) -o tests.exe tests.cpp $(libs)

# Compile and run an individual test suit
#   Assumed that only dependency is the corresponding header file in the parent directory.
#   Compiled with the STENCILA_TEST_SINGLE macro defined
#   More recent versions of gcov have "--relative-only" which "Only output information about source files with a relative pathname (after source prefix elision). 
#      Absolute paths are usually system header files and coverage of any inline functions therein is normally uninteresting."
%.exe : %.cpp ../%.hpp
	$(compile) -DSTENCILA_TEST_SINGLE -o $@ $< $(libs)
	
# Run the test
%.out : %.exe
	./$<
	
clean:
	rm -f *.o *.exe *.out *.gcno *.gcda *.gcov *.cov