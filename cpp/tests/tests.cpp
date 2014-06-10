//Compile all tests into a single executable
#undef STENCILA_TEST_SINGLE

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

// Tests in alphabetical order
#include "array-dynamic.cpp"
#include "array-static.cpp"
#include "component.cpp"
#include "dimension.cpp"
#include "git.cpp"
#include "html.cpp"
#include "json.cpp"
#include "map-context.cpp"
#include "python-context.cpp"
#include "query.cpp"
#include "r-context.cpp"
#include "stencil.cpp"
#include "traits.cpp"
#include "xml.cpp"
