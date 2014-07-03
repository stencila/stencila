#include <boost/test/unit_test.hpp>

#include <stencila/r-context>   

BOOST_AUTO_TEST_SUITE(r_context)
 
using namespace Stencila;
 
BOOST_AUTO_TEST_CASE(various){
    RContext c; 
     
    BOOST_CHECK(c.accept("r"));  
 
    c.execute("a = 42");
    BOOST_CHECK_EQUAL(c.write("a"),"42"); 

    c.assign("foo","\"bar\"");
    BOOST_CHECK_EQUAL(c.write("foo"),"bar");

    BOOST_CHECK(c.test("foo==\"bar\""));

    c.enter();
        c.assign("so","2");
        BOOST_CHECK_EQUAL(c.write("so"),"2");
    c.exit();
    BOOST_CHECK_THROW(c.write("so"),RException);
    
    c.call({"execute",{"answer = 42"}});
    BOOST_CHECK_EQUAL(c.call({"write",{"answer"}}),"42");
} 

BOOST_AUTO_TEST_CASE(begin_next){
    RContext c;
    c.execute("bits = c('a','b','c')"); 
    BOOST_CHECK(c.begin("bit","bits"));

    BOOST_CHECK_EQUAL(c.write("bit"),"a");
    BOOST_CHECK(c.next());
    BOOST_CHECK_EQUAL(c.write("bit"),"b");
    BOOST_CHECK(c.next());
    BOOST_CHECK_EQUAL(c.write("bit"),"c");
}

BOOST_AUTO_TEST_CASE(begin_next_dataframe){
    RContext c;
    c.execute("bits = data.frame(letter=c('a','b','c'),number=1:3)"); 
    BOOST_CHECK(c.begin("bit","bits"));

    BOOST_CHECK_EQUAL(c.write("bit$letter"),"a");
    BOOST_CHECK(c.next());
    BOOST_CHECK_EQUAL(c.write("bit$number"),"2");
    BOOST_CHECK(c.next());
    BOOST_CHECK_EQUAL(c.write("bit$letter"),"c");
}

BOOST_AUTO_TEST_CASE(image){
    RContext c;

    BOOST_CHECK_EQUAL(c.execute("plot(1,1)","png"),"1.png");
}

BOOST_AUTO_TEST_CASE(error){
    RContext c;

    BOOST_CHECK_THROW(c.execute("nonexistant<1"),RException);

    try{
        c.execute("nonexistant*5");
    } catch(const RException& exc){
        BOOST_CHECK_EQUAL(exc.what(),"Error: object 'nonexistant' not found\n");
    }
} 

BOOST_AUTO_TEST_SUITE_END() 
