#include <boost/test/unit_test.hpp>

#include <stencila/http-client.hpp>
using namespace Stencila::Http;

BOOST_AUTO_TEST_SUITE(http_client_slow)

BOOST_AUTO_TEST_CASE(getting){
	auto r1 = get("https://httpbin.org/get");
	BOOST_CHECK_EQUAL(r1.status(),200);

	auto r2 = get(
		"https://httpbin.org/get",
		{},
		{{"Header","Value"}}
	);
	BOOST_CHECK_EQUAL(r2.body(),"");
}

BOOST_AUTO_TEST_CASE(posting){
	auto response = post("https://httpbin.org/post");
	BOOST_CHECK_EQUAL(response.status(),200);
}

BOOST_AUTO_TEST_SUITE_END()
 