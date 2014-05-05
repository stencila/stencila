#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/python-context>

BOOST_AUTO_TEST_SUITE(python_context)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(various){
    PythonContext c;

    BOOST_CHECK(c.accept("py"));

    c.execute("a = 42");
    BOOST_CHECK_EQUAL(c.write("a"),"42");

    c.assign("foo","\"bar\"");
    BOOST_CHECK_EQUAL(c.write("foo"),"bar");

    BOOST_CHECK(c.test("foo==\"bar\""));

    c.enter();
        c.assign("so","2");
    c.exit();
    //BOOST_CHECK_THROWS(c.write("so"));
    
    c.execute("__callback__()");

    c.call({"execute",{"answer = 42"}});
    BOOST_CHECK_EQUAL(c.call({"write",{"answer"}}),"42");
}

BOOST_AUTO_TEST_SUITE_END()
