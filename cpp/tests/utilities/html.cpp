#include <iostream>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/utilities/html.hpp>

BOOST_AUTO_TEST_SUITE(html)

using namespace Stencila::Utilities::Html;

BOOST_AUTO_TEST_CASE(load){

    BOOST_CHECK_EQUAL(Document().dump(),"<!DOCTYPE html><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title /><meta charset=\"UTF-8\" /></head><body /></html>");

    #define CHECK(in,out) BOOST_CHECK_EQUAL(Document(in).find("body").dump_children(),out);

    CHECK(
        "<h2>subheading</h3>",
        "<h2>subheading</h2>"
    )

    #undef CHECK
}

BOOST_AUTO_TEST_SUITE_END()
