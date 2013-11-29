#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/component.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(component)

BOOST_AUTO_TEST_CASE(id){
    Component<> c1;
    BOOST_CHECK_EQUAL(c1.id().length(),22);

    Component<>* c2 = Component<>::obtain<Component<>>(c1.id());
    BOOST_CHECK(c2!=0);
    BOOST_CHECK_EQUAL(c1.id(),c2->id());
}

BOOST_AUTO_TEST_SUITE_END()
 