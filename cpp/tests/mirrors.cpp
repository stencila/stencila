#include <boost/test/unit_test.hpp>

#include <stencila/mirrors.hpp>

BOOST_AUTO_TEST_SUITE(mirrors)

using namespace Stencila;

struct A {

	bool a = true;
	char b  = 'b';
	int c = 42;

	template<class Mirror>
	void reflect(Mirror& mirror){
		mirror
			.data(a,"a")
			.data(b,"b")
			.data(c,"c")
		;
	}
};

struct B : A {

	float d = 3.14;
	double e = 3.142;
	std::string f = "f";

	template<class Mirror>
	void reflect(Mirror& mirror){
		A::reflect(mirror);
		mirror
			.data(d,"d")
			.data(e,"e")
			.data(f,"f")
		;
	}
};

BOOST_AUTO_TEST_CASE(has){
	A a;
	BOOST_CHECK(Has(a,"a"));
	BOOST_CHECK(not Has(a,"z"));
}

BOOST_AUTO_TEST_CASE(row_header){
	A a;
	B b;
	BOOST_CHECK_EQUAL(RowHeader(a),"a\tb\tc");
	BOOST_CHECK_EQUAL(RowHeader(b,","),"a,b,c,d,e,f");
}

BOOST_AUTO_TEST_CASE(row_generator){
	A a;
	BOOST_CHECK_EQUAL(RowGenerator(a),"1\tb\t42");
}

BOOST_AUTO_TEST_CASE(row_parser){
	A a;
	RowParser(a,"0\tz\t64");
	BOOST_CHECK_EQUAL(a.a,false);
	BOOST_CHECK_EQUAL(a.b,'z');
	BOOST_CHECK_EQUAL(a.c,64);
}

BOOST_AUTO_TEST_SUITE_END()
