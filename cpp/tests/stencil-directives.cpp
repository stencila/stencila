#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_directives)

BOOST_AUTO_TEST_CASE(par){
	{
		Stencil::Parameter p("x");
		BOOST_CHECK(p.valid);
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"");
		BOOST_CHECK_EQUAL(p.value,"");
	}{
		Stencil::Parameter p("x type number");
		BOOST_CHECK(p.valid);
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"number");
		BOOST_CHECK_EQUAL(p.value,"");
	}{
		Stencil::Parameter p("x type number value 42");
		BOOST_CHECK(p.valid);
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"number");
		BOOST_CHECK_EQUAL(p.value,"42");
	}{
		Stencil::Parameter p("x value 42");
		BOOST_CHECK(p.valid);
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"");
		BOOST_CHECK_EQUAL(p.value,"42");
	}{
		Stencil::Parameter p("x value pi*7*6");
		BOOST_CHECK(p.valid);
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"");
		BOOST_CHECK_EQUAL(p.value,"pi*7*6");
	}{
		Stencil::Parameter p("x foo bar");
		BOOST_CHECK(not p.valid);
	}
}

BOOST_AUTO_TEST_SUITE_END()
