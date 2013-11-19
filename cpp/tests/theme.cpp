#include <iostream>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/theme.hpp>

BOOST_AUTO_TEST_SUITE(theme)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(construct){
    Theme theme;
    BOOST_CHECK_EQUAL(theme.id().length(),(unsigned int)37);
    BOOST_CHECK_EQUAL(theme.obtain<Theme>(theme.id()),&theme);
}

BOOST_AUTO_TEST_CASE(unique_ids){
    Theme theme1;
    Theme theme2;
    BOOST_CHECK(theme1.id()!=theme2.id());
}

BOOST_AUTO_TEST_SUITE_END()
