#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/component.hpp>

BOOST_AUTO_TEST_SUITE(component)

using namespace Stencila;

//Define a class so we can derive from Component<> for testing
class Test : public Component<Test> {};

BOOST_AUTO_TEST_CASE(size){
    BOOST_CHECK_EQUAL(sizeof(Test),sizeof(void*));
}

BOOST_AUTO_TEST_CASE(title){
    Test c1;
    
    BOOST_CHECK_EQUAL(c1.title(),"");
    
    BOOST_CHECK_EQUAL(c1.title("A really useful component").title(),"A really useful component");
    
    c1.title() = "A quite useful component";
    BOOST_CHECK_EQUAL(c1.title(),"A quite useful component");
}

BOOST_AUTO_TEST_CASE(description){
    Test c1;
    
    BOOST_CHECK_EQUAL(c1.description(),"");
}

BOOST_AUTO_TEST_CASE(keywords){
    Test c1;
    
    BOOST_CHECK_EQUAL(c1.keywords().size(),0);

    std::vector<std::string> keywords = {"great","fantastic"};
    c1.keywords().insert(c1.keywords().end(),keywords.begin(),keywords.end());
    BOOST_CHECK_EQUAL(c1.keywords().size(),keywords.size());
}

BOOST_AUTO_TEST_CASE(authors){
    Test c1;
    
    BOOST_CHECK_EQUAL(c1.authors().size(),0);

    c1.authors({"Peter Pan","Marry Poppins"});
    c1.authors().push_back("Joe Bloggs");
    BOOST_CHECK_EQUAL(c1.authors().size(),3);
}

BOOST_AUTO_TEST_SUITE_END()
 