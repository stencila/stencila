#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/network.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(server)

BOOST_AUTO_TEST_CASE(basic){
	Server server;
	server.start();
	server.stop();
}

BOOST_AUTO_TEST_SUITE_END()
 