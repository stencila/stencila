#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <array>
#include <map>
#include <set>
#include <vector>

#include <stencila/traits.hpp>

BOOST_AUTO_TEST_SUITE(traits)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(traits){
    typedef std::vector<int> vec;
	BOOST_CHECK_EQUAL(IsContainer<vec>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<vec>::value,false);
    BOOST_CHECK_EQUAL(IsPaired<vec>::value,false);

    typedef std::array<double,10> arr;
	BOOST_CHECK_EQUAL(IsContainer<arr>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<arr>::value,false);
    BOOST_CHECK_EQUAL(IsPaired<arr>::value,false);
    
    typedef std::set<int> set;
	BOOST_CHECK_EQUAL(IsContainer<set>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<set>::value,true);
    BOOST_CHECK_EQUAL(IsPaired<set>::value,false);
    
    typedef std::map<int,int> map;
	BOOST_CHECK_EQUAL(IsContainer<map>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<map>::value,true);
    BOOST_CHECK_EQUAL(IsPaired<map>::value,true);
}

BOOST_AUTO_TEST_SUITE_END()
