//Compile all tests into a single executable

#define BOOST_TEST_MODULE tests
#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

// Global fixture for one off setup and teardown at start and end of testing
struct GlobalFixture {
    GlobalFixture(void){
        
    }
};
BOOST_GLOBAL_FIXTURE(GlobalFixture);

#define STENCILA_PYTHON_CONTEXTS 1
#define STENCILA_R_CONTEXTS 1

#include "component.cpp"
#include "stencil.cpp"
#include "network.cpp"

// Tests in alphabetical order
#include "array-dynamic-tests.cpp"
#include "array-static-tests.cpp"
#include "component-tests.cpp"
#include "dimension-tests.cpp"
#include "git-tests.cpp"
#include "html-tests.cpp"
#include "json-tests.cpp"
#include "map-context-tests.cpp"
#include "python-context-tests.cpp"
#include "query-tests.cpp"
#include "r-context-tests.cpp"
#include "stencil-tests.cpp"
#include "traits-tests.cpp"
#include "xml-tests.cpp"
