#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/array.hpp>
#include <stencila/query.hpp>

BOOST_AUTO_TEST_SUITE(array)

using namespace Stencila;

STENCILA_DIM(One,one,one,1);
STENCILA_DIM(Two,two,two,2);
STENCILA_DIM(Three,three,three,3);
STENCILA_DIM(Four,four,four,4);
STENCILA_DIM(Five,five,five,5);
STENCILA_DIM(Six,Sixe,six,6);
STENCILA_DIM(Seven,seven,seven,7);


BOOST_AUTO_TEST_CASE(constructors){
    Array<> a;
    BOOST_CHECK_EQUAL(a.size(),0);

    Array<> b(42,3.14);
    BOOST_CHECK_EQUAL(b.size(),42);
    BOOST_CHECK_EQUAL(b[0],3.14);
    BOOST_CHECK_EQUAL(b[41],3.14);

    Array<> c({1,2,3});
    BOOST_CHECK_EQUAL(c[0],1);
    BOOST_CHECK_EQUAL(c[1],2);
    BOOST_CHECK_EQUAL(c[2],3);

    std::vector<double> std_vector({1,2,3});
    Array<> d(std_vector);
    BOOST_CHECK_EQUAL(d[0],std_vector[0]);
    BOOST_CHECK_EQUAL(d[1],std_vector[1]);
    BOOST_CHECK_EQUAL(d[2],std_vector[2]);

    std::array<double,3> std_array = {1,2,3};
    Array<> e(std_array);
    BOOST_CHECK_EQUAL(e[0],std_array[0]);
    BOOST_CHECK_EQUAL(e[1],std_array[1]);
    BOOST_CHECK_EQUAL(e[2],std_array[2]);

    Array<> f({one,two,three});
    BOOST_CHECK_EQUAL(f.size(),1*2*3);
}

BOOST_AUTO_TEST_CASE(size){
    Array<> a;
    BOOST_CHECK_EQUAL(a.size(),0);
    BOOST_CHECK_EQUAL(a.size(10).size(),10);
}

BOOST_AUTO_TEST_CASE(query){
    Array<> a(42);

    // Static queries
    BOOST_CHECK_EQUAL(count(a),a.size());

    //Dynamic queries
    BOOST_CHECK_EQUAL(a(new Count)[0],count(a));
    BOOST_CHECK_EQUAL(a(new Sum)[0],sum(a));
}

BOOST_AUTO_TEST_SUITE_END()
