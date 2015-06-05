#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>
#include <boost/regex.hpp>

#include <stencila/component.hpp>

BOOST_AUTO_TEST_SUITE(component_quick)

using namespace Stencila;

boost::regex temp_path_pattern("/tmp/stencila/[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}$");

/**
 * @class Component
 *
 * By default a components path is empty
 */
BOOST_AUTO_TEST_CASE(path_initialised_empty){
	Component c;
	BOOST_CHECK_EQUAL(c.path(),"");
}

/**
 * Get path and ensure there is
 */
BOOST_AUTO_TEST_CASE(path_get_ensure){
	Component c;
	BOOST_CHECK(c.path(true)!="");
}

/**
 * @class Component
 *
 * Setting an empty path creates a temporary path
 */
BOOST_AUTO_TEST_CASE(path_set_empty){
	Component c;
	c.path("");
	BOOST_CHECK(boost::regex_match(c.path(),temp_path_pattern));
	BOOST_CHECK(boost::filesystem::exists(c.path()));
	c.destroy();
}

/**
 * @class Component
 *
 * Setting an empty path twice does not change the path
 */
BOOST_AUTO_TEST_CASE(path_set_empty_twice){
	Component c;
	std::string first = c.path("").path();
	std::string second = c.path("").path();
	BOOST_CHECK_EQUAL(first,second);
	c.destroy();
}


/**
 * @class Component
 *
 * Changing the path moves the component directory to the new path
 */
/*
BOOST_AUTO_TEST_CASE(path_change){
	Component c;
	
	std::string first = c.path(true);
	std::string second = Stencila::Host::temp_dirname();
	c.path(second);
	BOOST_CHECK(first!=second);
	BOOST_CHECK(not boost::filesystem::exists(first));
	BOOST_CHECK(boost::filesystem::exists(second));

	c.destroy();
	BOOST_CHECK(not boost::filesystem::exists(second));
}
*/

/**
 * @class Component
 *
 * When `write` is called with an empty path then a unique path
 * is created in the user's Stencila library
 */
BOOST_AUTO_TEST_CASE(write_path_empty){
	Component c;
	c.write();
	BOOST_CHECK(boost::regex_match(c.path(),temp_path_pattern));
	c.destroy();
}

/**
 * @class Component
 *
 * Destroying a component with an empty path works
 */
BOOST_AUTO_TEST_CASE(destroy_empty){
	Component c;
	c.destroy();
	BOOST_CHECK_EQUAL(c.path(),"");
}

/**
 * @class Component
 *
 * Destroying a component with a non-empty path removes it's directory
 */
BOOST_AUTO_TEST_CASE(destroy_transient){
	Component c;
	c.create("foo.txt");
	std::string path = c.path();
	c.destroy();
	BOOST_CHECK(not boost::filesystem::exists(path));
}

BOOST_AUTO_TEST_SUITE_END()
