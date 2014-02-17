#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/array.hpp>

BOOST_AUTO_TEST_SUITE(array)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(dimension_iterate){

    STENCILA_DIM(Region,regions,region,4);

    unsigned int levels[4] = {0,1,2,3};
    unsigned int level;

    level = 0;
    for(Level<Region> region=regions.begin(); region!=regions.end(); region++){
        BOOST_CHECK_EQUAL(region,levels[level++]);
    }

    level = 0;
    for(auto region: regions){
        BOOST_CHECK_EQUAL(region,levels[level++]);
    }

}

BOOST_AUTO_TEST_SUITE_END()
 