#include <boost/test/unit_test.hpp>

#include <stencila/frame-dynamic.hpp>
#include <stencila/array.hpp>
#include <stencila/structure.hpp>

BOOST_AUTO_TEST_SUITE(frame)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(basic){
    Frame<> frame;

    BOOST_CHECK_EQUAL(frame.rows(),0u);
    BOOST_CHECK_EQUAL(frame.columns(),0u);
    BOOST_CHECK_EQUAL(frame.labels().size(),0u);

    frame.add("col1",Integer);

    BOOST_CHECK_EQUAL(frame.rows(),0u);
    BOOST_CHECK_EQUAL(frame.columns(),1u);
    BOOST_CHECK_EQUAL(frame.labels().size(),1u);
    BOOST_CHECK_EQUAL(frame.label(0),"col1");
    BOOST_CHECK_EQUAL(frame.type(0).name(),"Integer");

    frame.append();
    BOOST_CHECK_EQUAL(frame.rows(),1u);
    BOOST_CHECK_EQUAL(frame.columns(),1u);

    frame(0,0) = 42;
    BOOST_CHECK_EQUAL(frame.type(0,0).name(),"Integer");
    BOOST_CHECK_EQUAL(frame.value<int>(0,0),42);

    frame(0,0) = 3.14;
    BOOST_CHECK_EQUAL(frame.type(0,0).name(),"Real");
    BOOST_CHECK_EQUAL(frame.value<double>(0,0),3.14);
    
}

BOOST_AUTO_TEST_CASE(construct){
    Frame<> frame1;
    BOOST_CHECK_EQUAL(frame1.rows(),0u);
    BOOST_CHECK_EQUAL(frame1.columns(),0u);

    Frame<> frame2({"a","b","c"},100);
    BOOST_CHECK_EQUAL(frame2.rows(),100u);
    BOOST_CHECK_EQUAL(frame2.columns(),3u);

    Frame<> frame3(100,{"a","b","c"});
    BOOST_CHECK_EQUAL(frame3.rows(),100u);
    BOOST_CHECK_EQUAL(frame3.columns(),3u);
    BOOST_CHECK_EQUAL(frame3.label(1),"b");

    Frame<> frame4 = frame3;
    BOOST_CHECK_EQUAL(frame4.rows(),100u);
    BOOST_CHECK_EQUAL(frame4.columns(),3u);
    BOOST_CHECK_EQUAL(frame4.label(1),"b");
}

STENCILA_DIM(Two,two,two,2);

struct A : public Structure<A> {

    bool a = true;
    char b  = 'b';
    Array<int,Two> c;

    template<class Mirror>
    void reflect(Mirror& mirror){
        mirror
            .data(a,"a")
            .data(b,"b")
            .data(c,"c")
        ;
    }
};

BOOST_AUTO_TEST_CASE(from_structure){
    Frame<> frame = Frame<>::of<A>();

    BOOST_CHECK_EQUAL(frame.columns(),4u);

    auto labels = frame.labels();
    BOOST_CHECK_EQUAL(labels[0],"a");
    BOOST_CHECK_EQUAL(labels[1],"b");
    BOOST_CHECK_EQUAL(labels[2],"c(0)");
    BOOST_CHECK_EQUAL(labels[3],"c(1)");
}

BOOST_AUTO_TEST_SUITE_END()
