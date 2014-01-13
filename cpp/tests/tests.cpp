//! A C++ file to compile all tests into a single executable
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

// A dummy test suite simply to make tests run in absense of any other tests
BOOST_AUTO_TEST_SUITE(dummy)

BOOST_AUTO_TEST_CASE(dummy){
}

BOOST_AUTO_TEST_SUITE_END()