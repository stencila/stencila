#include <boost/test/unit_test.hpp>

#include <stencila/query.hpp>

BOOST_AUTO_TEST_SUITE(query_quick)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(count_){
	std::vector<int> items(100);
	BOOST_CHECK_EQUAL(count(items),100u);
}

BOOST_AUTO_TEST_CASE(sum_){
	std::vector<int> items = {1,2,3};
	BOOST_CHECK_EQUAL(sum(items),6);

	Sum s1,s2,s3;
	s3.join(s1.apply(items));
	s3.join(s2.apply(items));
	BOOST_CHECK_EQUAL(s1,6);
	BOOST_CHECK_EQUAL(s2,6);
	BOOST_CHECK_EQUAL(s3,12);
}

BOOST_AUTO_TEST_SUITE_END()
 