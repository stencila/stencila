//! A C++ file to compile all tests into a single executable
#define BOOST_TEST_MODULE tests
#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

//! @brief Global fixture for one off setup and teardown at start and end of testing
struct GlobalFixture {
    GlobalFixture(void){
        
    }
};
BOOST_GLOBAL_FIXTURE(GlobalFixture);

#include "component.cpp"

#include "stencil.cpp"

#include "contexts/map.cpp"

#include "dimension.cpp"
#include "array.cpp"
#include "grid.cpp"

#include "utilities/git.cpp"
#include "utilities/html.cpp"
#include "utilities/xml.cpp"
