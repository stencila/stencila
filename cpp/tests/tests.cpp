//! Used when all test suites are compiled into a single executable
#define BOOST_TEST_MODULE tests
#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

//! @brief Global fixture for one off setup and teardown at start and end of testing
struct GlobalFixture {
    GlobalFixture(void){
        boost::filesystem::create_directories("outputs");
    }
};
BOOST_GLOBAL_FIXTURE(GlobalFixture);

#include "traits.cpp"
#include "print.cpp"
#include "reflect.cpp"

#include "json.cpp"
#include "xml.cpp"
#include "compress.cpp"

#include "component.cpp"

#include "arrays.cpp"

#include "tables-tableset.cpp"
#include "tables-table.cpp"
#include "tables-query.cpp"

#include "stencil.cpp"
#include "theme.cpp"
