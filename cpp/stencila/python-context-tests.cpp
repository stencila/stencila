#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/python-context.hpp>

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
}

BOOST_AUTO_TEST_SUITE_END()
