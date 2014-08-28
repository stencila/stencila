#include <boost/test/unit_test.hpp>
#include <boost/regex.hpp>

#include <stencila/component.hpp>

BOOST_AUTO_TEST_SUITE(component)

using namespace Stencila;

boost::regex temp_path_pattern(".*/temp/[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}$");

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
BOOST_AUTO_TEST_CASE(path_change){
    Component c;
    
    std::string first = c.read().path();
    std::string second = c.path(Stencila::Host::user_dir()+"/~temp").path();
    BOOST_CHECK(first!=second);
    BOOST_CHECK(not boost::filesystem::exists(first));
    BOOST_CHECK(boost::filesystem::exists(second));
    BOOST_CHECK_EQUAL(c.address(),"~temp");

    c.destroy();
    BOOST_CHECK(not boost::filesystem::exists(second));
}

/**
 * @class Component
 *
 * When `read` is called with an empty path then a unique path
 * is created in the user's Stencila library
 */
BOOST_AUTO_TEST_CASE(read_path_empty){
    Component c;
    std::string first = c.read().path();
    BOOST_CHECK(boost::regex_match(c.path(),temp_path_pattern));
    std::string second = c.read().path();
    BOOST_CHECK_EQUAL(first,second);
    c.destroy();
}

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

BOOST_AUTO_TEST_CASE(commit){
    Component c;

    BOOST_CHECK_EQUAL(c.commits().size(),0);

    c.commit();
    auto commits = c.commits();
    BOOST_CHECK_EQUAL(commits.size(),1);
    BOOST_CHECK_EQUAL(commits[0].message,"Updated");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.git"));
    c.destroy();
}

BOOST_AUTO_TEST_CASE(version){
    Component c;
    BOOST_CHECK_EQUAL(c.version(),"");
    c.commit();
    
    BOOST_CHECK_EQUAL(c.version("0.0.1").version(),"0.0.1");
    BOOST_CHECK_THROW(c.version("0.0.0"),Exception);

    BOOST_CHECK_EQUAL(c.version("0.1.0").version(),"0.1.0");
    BOOST_CHECK_THROW(c.version("0.0.1"),Exception);

    BOOST_CHECK_EQUAL(c.version("1.0.0").version(),"1.0.0");
    BOOST_CHECK_THROW(c.version("0.1.0"),Exception);

    c.destroy();
}

BOOST_AUTO_TEST_CASE(provide){
    using boost::filesystem::exists;

    Component c;
    
    c.create("version-0.0.1.txt","0.0.1");
        BOOST_CHECK(exists(c.path()+"/version-0.0.1.txt"));
    c.commit();
    c.version("0.0.1");

    c.delete_("version-0.0.1.txt");
        BOOST_CHECK(not exists(c.path()+"/version-0.0.1.txt"));
    c.create("version-0.0.2.txt","0.0.2");
        BOOST_CHECK(exists(c.path()+"/version-0.0.2.txt"));
    c.commit();
    c.version("0.0.2");

    c.provide("0.0.1");
        BOOST_CHECK(exists(c.path()+"/.0.0.1/version-0.0.1.txt"));
        BOOST_CHECK(not exists(c.path()+"/.0.0.1/version-0.0.2.txt"));
        BOOST_CHECK(not exists(c.path()+"/.0.0.1/.git"));

    c.provide("0.0.2");
        BOOST_CHECK(exists(c.path()+"/.0.0.2/version-0.0.2.txt"));
        BOOST_CHECK(not exists(c.path()+"/.0.0.2/version-0.0.1.txt"));
        BOOST_CHECK(not exists(c.path()+"/.0.0.2/.git"));

    c.destroy();
}

BOOST_AUTO_TEST_CASE(get){
    Component c;
    
    c.commit();
    c.version("0.0.1");
    c.version("0.0.2");
    c.declare();

    BOOST_CHECK_EQUAL(c.versions().size(),2);
    BOOST_CHECK_EQUAL(c.versions()[0],"0.0.1");
    BOOST_CHECK_EQUAL(c.versions()[1],"0.0.2");

    Component& c0 = Component::get<>(c.address());
    BOOST_CHECK(boost::filesystem::exists(c.path()));
    
    Component& c1 = Component::get<>(c.address(),"0.0.1");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.0.0.1"));

    Component& c2 = Component::get<>(c.address(),"0.0.2");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.0.0.2"));

    BOOST_CHECK_EQUAL(c0.address(),c1.address());
    BOOST_CHECK_EQUAL(c1.address(),c2.address());

    c.destroy();
}

BOOST_AUTO_TEST_SUITE_END()
 