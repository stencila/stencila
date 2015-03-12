#include <boost/test/unit_test.hpp>

#include <stencila/hub.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(hub_slow)

BOOST_AUTO_TEST_CASE(signin){	
	hub.signin("hub","insecure");
	BOOST_CHECK_EQUAL(hub.username(),"hub");

	auto doc = hub.get("user/current");
	BOOST_CHECK_EQUAL(doc["username"].as<std::string>(),"hub");

	hub.signout();
	BOOST_CHECK_EQUAL(hub.username(),"");
}

BOOST_AUTO_TEST_SUITE_END()
 