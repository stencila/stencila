#include <boost/test/unit_test.hpp>

#include <stencila/structure.hpp>

BOOST_AUTO_TEST_SUITE(structure_quick)

using namespace Stencila;

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

struct B : public Structure<B> {

	A a;
	int b = 314;

	template<class Mirror>
	void reflect(Mirror& mirror){
		mirror
			.data(a,"a")
			.data(b,"b")
		;
	}
};

BOOST_AUTO_TEST_CASE(is_structure){
	A a;
	BOOST_CHECK(IsStructure<A>::value);
}

BOOST_AUTO_TEST_CASE(has){
	A a;
	BOOST_CHECK(a.has("a"));
	BOOST_CHECK(a.has("b"));
	BOOST_CHECK(a.has("c"));
}

BOOST_AUTO_TEST_CASE(labels){
	A a;
	BOOST_CHECK_EQUAL(a.labels()[1],"b");
}

BOOST_AUTO_TEST_CASE(json_1){
	{
		A a;

		std::string json = 
R"({
    "a": "true",
    "b": "g",
    "c": "24"
}
)";

		a.json(json);
		BOOST_CHECK_EQUAL(a.a,true);
		BOOST_CHECK_EQUAL(a.b,'g');
		BOOST_CHECK_EQUAL(a.c,24);

		BOOST_CHECK_EQUAL(a.json(),json);
	}{
		B b;

		std::string json_in =
R"({
    "a": {
        "a": false,
        "b": "p",
        "c": 39
    },
    "b": 227
}
)";
		b.json(json_in);
		BOOST_CHECK_EQUAL(b.a.a,false);
		BOOST_CHECK_EQUAL(b.a.b,'p');
		BOOST_CHECK_EQUAL(b.a.c,39);
		BOOST_CHECK_EQUAL(b.b,227);

		std::string json_out =
R"({
    "a": {
        "a": "false",
        "b": "p",
        "c": "39"
    },
    "b": "227"
}
)";
		BOOST_CHECK_EQUAL(b.json(),json_out);
	}
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
