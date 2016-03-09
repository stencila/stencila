#include <boost/test/unit_test.hpp>

#include <stencila/component.hpp>
#include <stencila/host.hpp>

BOOST_AUTO_TEST_SUITE(component_snapshots)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(store_restore){
	Host::env_var("STENCILA_ORIGIN", "http://localhost:7300");
	Host::env_var("STENCILA_TOKEN", "01awXxfNYHqcRDRJS42sG3dhZvfqsZLYr1nd5tX7T7Nm5DXFpnAz6BrqvHRZ17vd0Rr4ihw9p0iG/xk03RCA==");

	Component c;

	c.address("admin/fkjaZEikvN62");
	BOOST_CHECK_EQUAL(c.address(),"admin/fkjaZEikvN62");

	c.write_to("foo.txt", "bar");
	BOOST_CHECK_EQUAL(c.read_from("foo.txt"), "bar");

	c.store();

	c.write_to("foo.txt", "baa baa black sheep");
	BOOST_CHECK_EQUAL(c.read_from("foo.txt"), "baa baa black sheep");

	c.restore();
	BOOST_CHECK_EQUAL(c.read_from("foo.txt"), "bar");
}

BOOST_AUTO_TEST_SUITE_END()
