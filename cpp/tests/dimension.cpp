#include <boost/test/unit_test.hpp>

#include <stencila/dimension.hpp>

BOOST_AUTO_TEST_SUITE(dimension_quick)

using namespace Stencila;

STENCILA_DIM(One,one,one,1);
STENCILA_DIM(Two,two,two,2);
STENCILA_DIM(Three,three,three,3);
STENCILA_DIM(Four,four,four,4);
STENCILA_DIM(Five,five,five,5);
STENCILA_DIM(Six,Sixe,six,6);
STENCILA_DIM(Seven,seven,seven,7); 

BOOST_AUTO_TEST_CASE(dimension_macro){
	BOOST_CHECK_EQUAL(Four::size(),4u);
	BOOST_CHECK_EQUAL(four.size(),4u);

	BOOST_CHECK_EQUAL(Four::name(),"four");
	BOOST_CHECK_EQUAL(four.name(),"four");
}

BOOST_AUTO_TEST_CASE(dimension_base){
	Dimension<> dim = three;

	BOOST_CHECK_EQUAL(dim.size(),3u);
	BOOST_CHECK_EQUAL(dim.name(),"three");
}

BOOST_AUTO_TEST_CASE(dimension_iterate){
	unsigned int levels[5] = {0,1,2,3,4};
	unsigned int index;

	index = 0;
	for(Level<Five> level=five.begin(); level!=five.end(); level++){
		BOOST_CHECK_EQUAL(level.index(),levels[index++]);
	}

	index = 0;
	for(auto level: five){
		BOOST_CHECK_EQUAL(level.index(),levels[index++]);
	}
}

BOOST_AUTO_TEST_SUITE_END()
