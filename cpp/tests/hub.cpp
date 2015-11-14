#include <boost/test/unit_test.hpp>

#include <stencila/hub.hpp>
#include <stencila/host.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(hub_slow)

BOOST_AUTO_TEST_CASE(signin){
	// For this test you need to set the environment variables
	// STENCILA_USERNAME and STENCILA_HUB_TOKEN (which should match of course!)
	std::string username = Host::env_var("STENCILA_USERNAME");

	hub.signin();
	BOOST_CHECK_EQUAL(hub.username(),username);

	auto doc = hub.get("user/current");
	BOOST_CHECK_EQUAL(doc["username"].as<std::string>(),username);

	hub.signout();
	BOOST_CHECK_EQUAL(hub.username(),"");
}

BOOST_AUTO_TEST_SUITE_END()
 