#include <boost/test/unit_test.hpp>

#include <stencila/sheet.hpp>

BOOST_AUTO_TEST_SUITE(sheet_quick)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(meta_attributes){
	Sheet s1;
	BOOST_CHECK_EQUAL(s1.title(),"");
	BOOST_CHECK_EQUAL(s1.description(),"");
	BOOST_CHECK_EQUAL(s1.authors().size(),0);
	BOOST_CHECK_EQUAL(s1.keywords().size(),0);

	Sheet s2;
	s2.load("title = A title\tdescription = A description\tauthors = Peter Pan, @captainhook\tkeywords = data, is, gold");

	BOOST_CHECK_EQUAL(s2.title(),"A title");
	BOOST_CHECK_EQUAL(s2.description(),"A description");
	
	auto a = s2.authors();
	BOOST_CHECK_EQUAL(a.size(),2);
	BOOST_CHECK_EQUAL(a[0],"Peter Pan");
	BOOST_CHECK_EQUAL(a[1],"@captainhook");

	auto k = s2.keywords();
	BOOST_CHECK_EQUAL(k.size(),3);
	BOOST_CHECK_EQUAL(k[0],"data");
	BOOST_CHECK_EQUAL(k[1],"is");
	BOOST_CHECK_EQUAL(k[2],"gold");
}

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
