#include <boost/test/unit_test.hpp>

#include <stencila/sheet.hpp>

BOOST_AUTO_TEST_SUITE(sheet_quick)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(identify){
	BOOST_CHECK_EQUAL(Sheet::identify(0,0),"A1");
	BOOST_CHECK_EQUAL(Sheet::identify(1,0),"A2");

	BOOST_CHECK_EQUAL(Sheet::identify(1,1),"B2");
	BOOST_CHECK_EQUAL(Sheet::identify(2,2),"C3");

	BOOST_CHECK_EQUAL(Sheet::identify(0,25),"Z1");
	BOOST_CHECK_EQUAL(Sheet::identify(0,26),"AA1");
	BOOST_CHECK_EQUAL(Sheet::identify(0,27),"AB1");
	BOOST_CHECK_EQUAL(Sheet::identify(0,28),"AC1");

	BOOST_CHECK_EQUAL(Sheet::identify(0,52),"BA1");
}

BOOST_AUTO_TEST_CASE(parse){
	auto p0 = Sheet::parse("");
	BOOST_CHECK_EQUAL(p0[0],"");
	BOOST_CHECK_EQUAL(p0[1],"");
	BOOST_CHECK_EQUAL(p0[2],"");

	// Tabs are replaced with spaces
	BOOST_CHECK_EQUAL(Sheet::parse("\tfoo\t\tbar\t")[0]," foo  bar ");

	// Spaces are significant before, after and within a constant
	BOOST_CHECK_EQUAL(Sheet::parse("42")[0],"42");
	BOOST_CHECK_EQUAL(Sheet::parse(" 42")[0]," 42");
	BOOST_CHECK_EQUAL(Sheet::parse(" foo bar ")[0]," foo bar ");

	// Expressions
	for(auto content : {"= 6*7"," =6*7"," = 6*7  "}){
		auto p = Sheet::parse(content);
		BOOST_CHECK_EQUAL(p[0],"");
		BOOST_CHECK_EQUAL(p[1],"6*7");
		BOOST_CHECK_EQUAL(p[2],"");
	}

	// Expression with alias
	for(auto content : {"answer = 6*7"," answer =6*7"," answer= 6*7 "}){
		auto p = Sheet::parse(content);
		BOOST_CHECK_EQUAL(p[0],"");
		BOOST_CHECK_EQUAL(p[1],"6*7");
		BOOST_CHECK_EQUAL(p[2],"answer");
	}
}

BOOST_AUTO_TEST_SUITE_END()
