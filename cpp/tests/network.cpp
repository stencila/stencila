#include <boost/test/unit_test.hpp>

#include <stencila/network.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(server_slow)

BOOST_AUTO_TEST_CASE(basic){
	Server server;
	// Currently not running this
	// because it blocks!
	//server.start();
	//server.stop();
}

BOOST_AUTO_TEST_SUITE_END()
 