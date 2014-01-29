#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/regex.hpp>

#include <stencila/component.hpp>

BOOST_AUTO_TEST_SUITE(component)

using namespace Stencila;

//Define a class so we can derive from Component<> for testing
class Test : public Component<Test> {
private:
    friend class Component<Test>;
    const char* type_(void) const {
        return "test";
    }
    void read_(void){

    }
    void write_(void){
        
    }
};

boost::regex transient_path_pattern("^"+Stencila::Host::user_dir()+"/~[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}$");//"

BOOST_AUTO_TEST_CASE(title){
    Test c;
    
    BOOST_CHECK_EQUAL(c.title(),"");
    
    BOOST_CHECK_EQUAL(c.title("A really useful component").title(),"A really useful component");
    
    c.title() = "A quite useful component";
    BOOST_CHECK_EQUAL(c.title(),"A quite useful component");
}

BOOST_AUTO_TEST_CASE(description){
    Test c;
    
    BOOST_CHECK_EQUAL(c.description(),"");
}

BOOST_AUTO_TEST_CASE(keywords){
    Test c;
    
    BOOST_CHECK_EQUAL(c.keywords().size(),0);

    std::vector<std::string> keywords = {"great","fantastic"};
    c.keywords().insert(c.keywords().end(),keywords.begin(),keywords.end());
    BOOST_CHECK_EQUAL(c.keywords().size(),keywords.size());
}

BOOST_AUTO_TEST_CASE(authors){
    Test c;
    
    BOOST_CHECK_EQUAL(c.authors().size(),0);

    c.authors({"Peter Pan","Marry Poppins"});
    c.authors().push_back("Joe Bloggs");
    BOOST_CHECK_EQUAL(c.authors().size(),3);
}

/**
 * @class Component
 *
 * By default a components path is empty
 */
BOOST_AUTO_TEST_CASE(path_initialised_empty){
    Test c;
    BOOST_CHECK_EQUAL(c.path(),"");
}

/**
 * @class Component
 *
 * Setting an empty path creates a transient path
 */
BOOST_AUTO_TEST_CASE(path_set_empty){
    Test c;
    c.path("");
    BOOST_CHECK(boost::regex_match(c.path(),transient_path_pattern));
    BOOST_CHECK(boost::filesystem::exists(c.path()));
    c.destroy();
}

/**
 * @class Component
 *
 * Setting an empty path twice does not change the path
 */
BOOST_AUTO_TEST_CASE(path_set_empty_twice){
    Test c;
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
    Test c;
    
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
    Test c;
    std::string first = c.read().path();
    BOOST_CHECK(boost::regex_match(c.path(),transient_path_pattern));
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
    Test c;
    c.write();
    BOOST_CHECK(boost::regex_match(c.path(),transient_path_pattern));
    c.destroy();
}

/**
 * @class Component
 *
 * Destroying a component with an empty path works
 */
BOOST_AUTO_TEST_CASE(destroy_empty){
    Test c;
    c.destroy();
    BOOST_CHECK_EQUAL(c.path(),"");
}

/**
 * @class Component
 *
 * Destroying a component with a non-empty path removes it's directory
 */
BOOST_AUTO_TEST_CASE(destroy_transient){
    Test c;
    c.create("foo.txt");
    std::string path = c.path();
    c.destroy();
    BOOST_CHECK(not boost::filesystem::exists(path));
}

BOOST_AUTO_TEST_CASE(commit){
    Test c;

    BOOST_CHECK_EQUAL(c.log().size(),0);

    c.commit();
    auto log = c.log();
    BOOST_CHECK_EQUAL(log.size(),1);
    BOOST_CHECK_EQUAL(log[0].message,"Updated");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.git"));
    c.destroy();
}

BOOST_AUTO_TEST_CASE(version){
    Test c;
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
    Test c;
    
    c.create("version-0.0.1.txt","0.0.1");
    c.commit();
    c.version("0.0.1");

    c.delete_("version-0.0.1.txt");
    c.create("version-0.0.2.txt","0.0.2");
    c.commit();
    c.version("0.0.2");
    
    c.provide("0.0.1");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.0.0.1/version-0.0.1.txt"));
    BOOST_CHECK(not boost::filesystem::exists(c.path()+"/.0.0.1/version-0.0.2.txt"));
    BOOST_CHECK(not boost::filesystem::exists(c.path()+"/.0.0.1/.git"));

    c.provide("0.0.2");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.0.0.2/version-0.0.2.txt"));
    BOOST_CHECK(not boost::filesystem::exists(c.path()+"/.0.0.2/version-0.0.1.txt"));
    BOOST_CHECK(not boost::filesystem::exists(c.path()+"/.0.0.2/.git"));

    c.destroy();
}

BOOST_AUTO_TEST_CASE(obtain){
    Test c;
    
    c.commit();
    c.version("0.0.1");
    c.version("0.0.2");
    
    Test& c1 = Stencila::obtain<Test>(c.address(),"0.0.1");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.0.0.1"));

    Test& c2 = Stencila::obtain<Test>(c.address(),"0.0.2");
    BOOST_CHECK(boost::filesystem::exists(c.path()+"/.0.0.2"));

    BOOST_CHECK_EQUAL(c1.address(),c2.address());

    c.destroy();
}

BOOST_AUTO_TEST_SUITE_END()
 