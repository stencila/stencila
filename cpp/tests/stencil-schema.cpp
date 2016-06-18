#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;


BOOST_AUTO_TEST_SUITE(stencil_schema)

BOOST_AUTO_TEST_CASE(set_get){
	Stencil s;

	BOOST_CHECK_EQUAL(s.schema(), "");
	BOOST_CHECK_EQUAL(s.schema("rmd").schema(), "rmd");
}

BOOST_AUTO_TEST_CASE(conform){
	Stencil s("html://Foo");

	BOOST_CHECK(not s.select("p"));
	BOOST_CHECK_EQUAL(s.html(), "Foo");

	s.conform();
	BOOST_CHECK(s.select("p"));
	BOOST_CHECK_EQUAL(s.html(), "<p>Foo</p>");
}

BOOST_AUTO_TEST_SUITE_END()
