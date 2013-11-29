#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/component.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(component)

class Test : public Component<Test> {
public:
    static std::string type(void){
        return "test";
    };
    Test(void){
    }
    Test(const Id& id){
    }
};

BOOST_AUTO_TEST_CASE(id){
    Test c1;
    BOOST_CHECK_EQUAL(c1.id().length(),22);

    Test c2;
    BOOST_CHECK(c2.id()!=c1.id());

    Test* c1p = Component<>::obtain<Test>(c1.id());
    BOOST_CHECK(c1p!=0);
    BOOST_CHECK_EQUAL(c1.id(),c1p->id());
}

BOOST_AUTO_TEST_CASE(tags){
    Test c1;
    BOOST_CHECK_EQUAL(c1.tags().size(),0);

    std::vector<std::string> tags = {"great","fantastic"};
    c1.tags().insert(c1.tags().end(),tags.begin(),tags.end());
    BOOST_CHECK_EQUAL(c1.tags().size(),tags.size());
}

BOOST_AUTO_TEST_CASE(authors){
    Test c1;
    BOOST_CHECK_EQUAL(c1.tags().size(),0);

    c1.tags({"Peter Pan","Marry Poppins"});
    c1.tags().push_back("Joe Bloggs");
    BOOST_CHECK_EQUAL(c1.tags().size(),3);
}

BOOST_AUTO_TEST_SUITE_END()
 