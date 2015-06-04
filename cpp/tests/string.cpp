#include <boost/test/unit_test.hpp>

#include <stencila/string.hpp>

BOOST_AUTO_TEST_SUITE(string_quick)

BOOST_AUTO_TEST_CASE(stringify){
	using Stencila::string;
	BOOST_CHECK_EQUAL(string(true),"1");
	BOOST_CHECK_EQUAL(string('b'),"b");
	BOOST_CHECK_EQUAL(string(3.14),"3.14");
}

BOOST_AUTO_TEST_CASE(unstringify){
	using Stencila::unstring;
	BOOST_CHECK_EQUAL(unstring<bool>("1"),1);
	BOOST_CHECK_EQUAL(unstring<int>("42"),42);
	BOOST_CHECK_EQUAL(unstring<double>("3.14"),3.14);
	BOOST_CHECK_EQUAL(unstring<std::string>("foo"),"foo");
}

BOOST_AUTO_TEST_CASE(trim){
	using Stencila::trim;
	std::string string;
	BOOST_CHECK_EQUAL(trim(string="abc"),"abc");
	BOOST_CHECK_EQUAL(trim(string=" abc"),"abc");
	BOOST_CHECK_EQUAL(trim(string="abc  "),"abc");
	BOOST_CHECK_EQUAL(trim(string=" a b c "),"a b c");
}

BOOST_AUTO_TEST_CASE(lower){
	using Stencila::lower;
	BOOST_CHECK_EQUAL(lower("ABC"),"abc");
	BOOST_CHECK_EQUAL(lower("abc"),"abc");
}

BOOST_AUTO_TEST_CASE(upper){
	using Stencila::upper;
	BOOST_CHECK_EQUAL(upper("abc"),"ABC");
	BOOST_CHECK_EQUAL(upper("ABC"),"ABC");
}

BOOST_AUTO_TEST_CASE(replace_all){
	using Stencila::replace_all;
	std::string string = "abc";
	BOOST_CHECK_EQUAL(replace_all(string,"b","a"),"aac");
	BOOST_CHECK_EQUAL(replace_all(string,"a","d"),"ddc");
	BOOST_CHECK_EQUAL(string,"ddc");
}

BOOST_AUTO_TEST_CASE(split){
	using Stencila::split;
	auto bits = split("a,b,c",",");
	BOOST_CHECK_EQUAL(bits.size(),3);
	BOOST_CHECK_EQUAL(bits[0],"a");
	BOOST_CHECK_EQUAL(bits[1],"b");
	BOOST_CHECK_EQUAL(bits[2],"c");
}

BOOST_AUTO_TEST_CASE(join){
	using Stencila::join;
	BOOST_CHECK_EQUAL(join({"a","b","c"},","),"a,b,c");
	BOOST_CHECK_EQUAL(join({"a","b"},"|"),"a|b");
}

BOOST_AUTO_TEST_CASE(slugify){
	using Stencila::slugify;
	BOOST_CHECK_EQUAL(slugify("a b c"),"a-b-c");
	BOOST_CHECK_EQUAL(slugify(" a b c  "),"a-b-c");
	BOOST_CHECK_EQUAL(slugify("a^b%c&d*e"),"a-b-c-d-e");
	BOOST_CHECK_EQUAL(slugify("The quick brown..."),"the-quick-brown---");
	BOOST_CHECK_EQUAL(slugify("The quick brown fox",9),"the-quick");
}

BOOST_AUTO_TEST_SUITE_END()
