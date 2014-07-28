#include <boost/test/unit_test.hpp>

#include <stencila/reflector.hpp>

BOOST_AUTO_TEST_SUITE(reflector)

using namespace Stencila;

struct A : public Reflector<A> {

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

BOOST_AUTO_TEST_CASE(has){
	A a;
	BOOST_CHECK(a.has("a"));
}

BOOST_AUTO_TEST_CASE(header_row){
	A a;
	BOOST_CHECK_EQUAL(a.header_row(","),"a,b,c");
}

BOOST_AUTO_TEST_CASE(to_row){
	A a;
	BOOST_CHECK_EQUAL(a.to_row(","),"1,b,42");
}

BOOST_AUTO_TEST_CASE(from_row){
	A a;
	a.from_row("0,z,64",",");
	BOOST_CHECK_EQUAL(a.a,false);
	BOOST_CHECK_EQUAL(a.b,'z');
	BOOST_CHECK_EQUAL(a.c,64);
}

BOOST_AUTO_TEST_SUITE_END()
