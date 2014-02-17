#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/array.hpp>
#include <stencila/query.hpp>

BOOST_AUTO_TEST_SUITE(array)

using namespace Stencila;

STENCILA_DIM(One,one,one,1);
STENCILA_DIM(Two,two,twp,2);
STENCILA_DIM(Three,three,three,3);
STENCILA_DIM(Four,four,four,4);
STENCILA_DIM(Five,five,five,5);
STENCILA_DIM(Six,Sixe,six,6);
STENCILA_DIM(Seven,seven,seven,7);

BOOST_AUTO_TEST_CASE(dimension_macro_statics){
    BOOST_CHECK_EQUAL(Four::size(),4);
    BOOST_CHECK_EQUAL(four.size(),4);

    BOOST_CHECK_EQUAL(Four::label(),"four");
    BOOST_CHECK_EQUAL(four.label(),"four");
}

BOOST_AUTO_TEST_CASE(dimension_iterate){
    unsigned int levels[5] = {0,1,2,3,4};
    unsigned int index;

    index = 0;
    for(Level<Five> level=five.begin(); level!=five.end(); level++){
        BOOST_CHECK_EQUAL(level,levels[index++]);
    }

    index = 0;
    for(auto level: five){
        BOOST_CHECK_EQUAL(level,levels[index++]);
    }
}

BOOST_AUTO_TEST_CASE(static_array_constructors){
    typedef Array<double,Three> A;

    A a;

    A b(3.14);
    BOOST_CHECK_EQUAL(b[0],3.14);
    BOOST_CHECK_EQUAL(b[1],3.14);
    BOOST_CHECK_EQUAL(b[2],3.14);

    A c({6,7,9});
    BOOST_CHECK_EQUAL(c[0],6);
    BOOST_CHECK_EQUAL(c[1],7);
    BOOST_CHECK_EQUAL(c[2],9);

    std::vector<double> std_vector({1,2,3});
    A d(std_vector);
    BOOST_CHECK_EQUAL(d[0],std_vector[0]);
    BOOST_CHECK_EQUAL(d[1],std_vector[1]);
    BOOST_CHECK_EQUAL(d[2],std_vector[2]);

    std::array<double,3> std_array = {1,2,3};
    A e(std_array);
    BOOST_CHECK_EQUAL(e[0],std_array[0]);
    BOOST_CHECK_EQUAL(e[1],std_array[1]);
    BOOST_CHECK_EQUAL(e[2],std_array[2]);
}

BOOST_AUTO_TEST_CASE(dynmaic_array_constructors){
    Array<> a;
    BOOST_CHECK_EQUAL(a.size(),0);

    Array<> b(42);
    BOOST_CHECK_EQUAL(b.size(),42);

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
}

BOOST_AUTO_TEST_CASE(static_array_sizes){
    Array<double,Three> a;
    BOOST_CHECK_EQUAL(a.size(),three.size());

    Array<double,Four,Five,Seven> b;
    BOOST_CHECK_EQUAL(b.size(),four.size()*five.size()*seven.size());
}

BOOST_AUTO_TEST_CASE(dynamic_array_sizes){
    Array<> a;
    BOOST_CHECK_EQUAL(a.size(),0);
    BOOST_CHECK_EQUAL(a.size(10).size(),10);
}

BOOST_AUTO_TEST_CASE(static_array_dimensioned){
    Array<double,Four,Five,Seven> a;

    BOOST_CHECK(a.dimensioned(four));
    BOOST_CHECK(a.dimensioned(seven));
    BOOST_CHECK(not a.dimensioned(two));
}

BOOST_AUTO_TEST_CASE(static_array_subscript){
    Array<double,One> a = {1};
    BOOST_CHECK_EQUAL(a(Level<One>(0)),1);

    Array<double,One,Two> b = {11,12};
    BOOST_CHECK_EQUAL(b(Level<One>(0),Level<Two>(0)),11);
    BOOST_CHECK_EQUAL(b(Level<One>(0),Level<Two>(1)),12);
    
    Array<double,Two,Three> c = {11,12,13,21,22,23};
    BOOST_CHECK_EQUAL(c(Level<Two>(0),Level<Three>(1)),12);
    BOOST_CHECK_EQUAL(c(Level<Two>(1),Level<Three>(0)),21);
    BOOST_CHECK_EQUAL(c(Level<Two>(1),Level<Three>(1)),22);
    BOOST_CHECK_EQUAL(c(Level<Two>(1),Level<Three>(2)),23);

    // The following should not compile because they involve the
    // wrong number of levels, or levels in the wrong order:
    //   a(Level<One>(0),Level<Two>(0));
    //   b(Level<One>(0));
    //   c(Level<Three>(0),Level<Two>(0));
    //(that's a feature, not a bug!)
}

BOOST_AUTO_TEST_CASE(static_array_query){
    Array<int,Two,Five,Seven> a = 3;
    BOOST_CHECK_EQUAL(count(a),a.size());
    BOOST_CHECK_EQUAL(sum(a),a.size()*3);

    Array<char,Four> b = {'f','o','r','d'};
    std::string word;
    each(b,[&word](char item){
        word += item;
    });
    BOOST_CHECK_EQUAL(word,"ford");
}

BOOST_AUTO_TEST_CASE(dynamic_array_query){
    Array<> a(42);
    BOOST_CHECK_EQUAL(count(a),a.size());
}

BOOST_AUTO_TEST_SUITE_END()
 