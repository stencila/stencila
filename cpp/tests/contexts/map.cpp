#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/contexts/map.hpp>

BOOST_AUTO_TEST_SUITE(xml)

using namespace Stencila;
using namespace Stencila::Contexts;

BOOST_AUTO_TEST_CASE(assign){
    Map map;

    map.assign("foo","bar");
    BOOST_CHECK_EQUAL(map.text("foo"),"bar");

    map.assign("foo","barred");
    BOOST_CHECK_EQUAL(map.text("foo"),"barred");
}

BOOST_AUTO_TEST_CASE(test){
    Map map;

    map.assign("ok","1");
    BOOST_CHECK(map.test("ok"));
    
    map.assign("ok","");
    BOOST_CHECK(not map.test("ok"));
}

BOOST_AUTO_TEST_CASE(subject_match){
    Map map;

    map.assign("a","A");
    map.subject("a");
        BOOST_CHECK(map.match("A"));
        BOOST_CHECK(not map.match("B"));

        map.enter("a");
            map.assign("a1","1");

            map.subject("a1");
                BOOST_CHECK(map.match("1"));
                BOOST_CHECK(not map.match("2"));
            map.unsubject();

        map.exit();   

    map.unsubject();
}

BOOST_AUTO_TEST_CASE(loop){
    Map map;

    // Set up some variable for looping over
    map.assign("planets","");
    map.enter("planets");
        map.assign("1","Argabuthon");
        map.assign("2","Bartledan");
        map.assign("3","Bethselamin");
        map.assign("4","Earth");
        map.assign("5","Gagrakacka");
    map.exit();

    map.assign("syllables","");
    map.enter("syllables");
        map.assign("1","tzjin");
        map.assign("2","anthony");
        map.assign("3","ks");
    map.exit();

    map.enter("planets");
    BOOST_CHECK_EQUAL(map.text("4"),"Earth");
    map.exit();

    // Outer loop
    map.begin("planet","planets");
        BOOST_CHECK_EQUAL(map.text("planet"),"Argabuthon");
        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.text("planet"),"Bartledan");
        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.text("planet"),"Bethselamin");

        //Inner loop
        map.begin("syllable","syllables");
            BOOST_CHECK_EQUAL(map.text("syllable"),"tzjin");
            BOOST_CHECK(map.next());
            BOOST_CHECK_EQUAL(map.text("syllable"),"anthony");
            BOOST_CHECK(map.next());
            BOOST_CHECK_EQUAL(map.text("syllable"),"ks");
            BOOST_CHECK(not map.next());
        map.end();
        BOOST_CHECK_THROW(map.test("syllable"),Exception);

        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.text("planet"),"Earth");
        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.text("planet"),"Gagrakacka");
        BOOST_CHECK(not map.next());
    map.end();
    BOOST_CHECK_THROW(map.test("planet"),Exception);
}

BOOST_AUTO_TEST_SUITE_END()
