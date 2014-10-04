#include <boost/test/unit_test.hpp>

#include <stencila/reflector.hpp>
#include <stencila/mirror-inspect.hpp>
#include <stencila/mirror-rows.hpp>
#include <stencila/mirror-stencil.hpp>

BOOST_AUTO_TEST_SUITE(mirrors)

using namespace Stencila;
using namespace Stencila::Mirrors;

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

struct B : public A, public Reflector<B> {
	using Reflector<B>::has_reflect;

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

struct C : public Reflector<C>{

	A a;
	B b;

	template<class Mirror>
	void reflect(Mirror& mirror){
		mirror
			.data(a,"a")
			.data(b,"b")
		;
	}
};

BOOST_AUTO_TEST_CASE(has){
	A a;
	BOOST_CHECK(Has(a,"a"));
	BOOST_CHECK(not Has(a,"z"));
}

BOOST_AUTO_TEST_CASE(stencil_parser){
	{
		A a;
		Stencil stencil;
		stencil.html(std::string(R"(<div id="a">0</div><div id="b">j</div><div id="c">4200</div>)"));
		StencilParser(a,stencil);
		BOOST_CHECK_EQUAL(a.a,false);
		BOOST_CHECK_EQUAL(a.b,'j');
		BOOST_CHECK_EQUAL(a.c,4200);
	}
}

BOOST_AUTO_TEST_CASE(stencil_generator){
	{
		A a;
		Stencil stencil;
		StencilGenerator(a,stencil);
		BOOST_CHECK_EQUAL(stencil.dump(),R"(<div id="a">1</div><div id="b">b</div><div id="c">42</div>)");
	}
	{
		C c;
		Stencil stencil;
		StencilGenerator(c,stencil);
		BOOST_CHECK_EQUAL(stencil.dump(),R"(<div id="a"><div id="a">1</div><div id="b">b</div><div id="c">42</div></div><div id="b"><div id="a">1</div><div id="b">b</div><div id="c">42</div><div id="d">3.14</div><div id="e">3.142</div><div id="f">f</div></div>)");
	}
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
