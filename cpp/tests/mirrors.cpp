#include <boost/test/unit_test.hpp>

#include <stencila/structure.hpp>
#include <stencila/array.hpp>

#include <stencila/mirror-inspect.hpp>
#include <stencila/mirror-rows.hpp>
#include <stencila/mirror-stencil.hpp>

BOOST_AUTO_TEST_SUITE(mirrors)

using namespace Stencila;
using namespace Stencila::Mirrors;

struct A : public Structure<A> {

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

struct B : public A, public Structure<B> {
	using Structure<B>::structure_type;
	using Structure<B>::labels;

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

struct C : public Structure<C>{

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

STENCILA_DIM(Two,two,two,2);
STENCILA_DIM(Three,three,three,3);

struct D : public Structure<D> {

	int a = 42;
	Array<int,Two> b;

	template<class Mirror>
	void reflect(Mirror& mirror){
		mirror
			.data(a,"a")
			.data(b,"b")
		;
	}
};

BOOST_AUTO_TEST_CASE(has){
	BOOST_CHECK(Has("a").mirror<A>());
	BOOST_CHECK(not Has("z").mirror<A>());
}

BOOST_AUTO_TEST_CASE(labels){
	std::vector<std::string> a = Labels().mirror<A>();
	BOOST_CHECK_EQUAL(a.size(),3u);
	BOOST_CHECK_EQUAL(a[0],"a");
	BOOST_CHECK_EQUAL(a[1],"b");

	std::vector<std::string> c = Labels().mirror<C>();
	BOOST_CHECK_EQUAL(c.size(),9u);
	BOOST_CHECK_EQUAL(c[0],"a.a");
	BOOST_CHECK_EQUAL(c[2],"a.c");
	BOOST_CHECK_EQUAL(c[4],"b.b");

	std::vector<std::string> d = Labels().mirror<D>();
	BOOST_CHECK_EQUAL(d.size(),3u);
	BOOST_CHECK_EQUAL(d[0],"a");
	BOOST_CHECK_EQUAL(d[1],"b(0)");
	BOOST_CHECK_EQUAL(d[2],"b(1)");

	std::vector<std::string> e = Labels().mirror<Array<int,Two,Three>>();
	BOOST_CHECK_EQUAL(e.size(),6u);
	BOOST_CHECK_EQUAL(e[0],"(0,0)");
	BOOST_CHECK_EQUAL(e[1],"(0,1)");
}

BOOST_AUTO_TEST_CASE(stencil_parser){
	{
		A a;
		Stencil stencil;
		stencil.html(std::string(R"(<div id="a">0</div><div id="b">j</div><div id="c">4200</div>)"));
		StencilParser(stencil).mirror(a);
		BOOST_CHECK_EQUAL(a.a,false);
		BOOST_CHECK_EQUAL(a.b,'j');
		BOOST_CHECK_EQUAL(a.c,4200);
	}
}

BOOST_AUTO_TEST_CASE(stencil_generator){
	{
		A a;
		Stencil stencil;
		StencilGenerator(stencil).mirror(a);
		BOOST_CHECK_EQUAL(stencil.dump(),R"(<div id="a">1</div><div id="b">b</div><div id="c">42</div>)");
	}
	{
		C c;
		Stencil stencil;
		StencilGenerator(stencil).mirror(c);
		BOOST_CHECK_EQUAL(stencil.dump(),R"(<div id="a"><div id="a">1</div><div id="b">b</div><div id="c">42</div></div><div id="b"><div id="a">1</div><div id="b">b</div><div id="c">42</div><div id="d">3.14</div><div id="e">3.142</div><div id="f">f</div></div>)");
	}
}

BOOST_AUTO_TEST_CASE(row_header){
	BOOST_CHECK_EQUAL(RowHeader().mirror<A>(),"a\tb\tc");
	BOOST_CHECK_EQUAL(RowHeader(",").mirror<B>(),"a,b,c,d,e,f");
}

BOOST_AUTO_TEST_CASE(row_generator){
	A a;
	BOOST_CHECK_EQUAL(RowGenerator().mirror(a),"1\tb\t42");
}

BOOST_AUTO_TEST_CASE(row_parser){
	A a;
	RowParser("0\tz\t64").mirror(a);
	BOOST_CHECK_EQUAL(a.a,false);
	BOOST_CHECK_EQUAL(a.b,'z');
	BOOST_CHECK_EQUAL(a.c,64);
}

BOOST_AUTO_TEST_SUITE_END()
