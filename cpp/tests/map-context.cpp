#include <boost/test/unit_test.hpp>

#include <stencila/map-context.hpp>

BOOST_AUTO_TEST_SUITE(map_content)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(assign){
    MapContext map;

    map.assign("foo","bar");
    BOOST_CHECK_EQUAL(map.write("foo"),"bar");

    map.assign("foo","barred");
    BOOST_CHECK_EQUAL(map.write("foo"),"barred");
}

BOOST_AUTO_TEST_CASE(test){
    MapContext map;

    map.assign("ok","1");
    BOOST_CHECK(map.test("ok"));
    
    map.assign("ok","");
    BOOST_CHECK(not map.test("ok"));
}

BOOST_AUTO_TEST_CASE(subject_match){
    MapContext map;

    map.assign("a","A");
    map.assign("b","B");

    map.mark("a");
        BOOST_CHECK(map.match("A"));
        BOOST_CHECK(not map.match("B"));

        map.mark("b");
            BOOST_CHECK(not map.match("A"));
            BOOST_CHECK(map.match("B"));
        map.unmark();   

    map.unmark();
}

BOOST_AUTO_TEST_CASE(loop){
    MapContext map;

    // Set up some variable for looping over
    map.assign("planets","Argabuthon Bartledan Bethselamin Earth Gagrakacka");
    map.assign("syllables","tzjin anthony ks");

    // Outer loop
    map.begin("planet","planets");
        BOOST_CHECK_EQUAL(map.write("planet"),"Argabuthon");
        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.write("planet"),"Bartledan");
        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.write("planet"),"Bethselamin");

        //Inner loop
        map.begin("syllable","syllables");
            BOOST_CHECK_EQUAL(map.write("syllable"),"tzjin");
            BOOST_CHECK(map.next());
            BOOST_CHECK_EQUAL(map.write("syllable"),"anthony");
            BOOST_CHECK(map.next());
            BOOST_CHECK_EQUAL(map.write("syllable"),"ks");
            BOOST_CHECK(not map.next());
        map.end();
        BOOST_CHECK_THROW(map.test("syllable"),Exception);

        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.write("planet"),"Earth");
        BOOST_CHECK(map.next());
        BOOST_CHECK_EQUAL(map.write("planet"),"Gagrakacka");
        BOOST_CHECK(not map.next());
    map.end();
    BOOST_CHECK_THROW(map.test("planet"),Exception);
}

BOOST_AUTO_TEST_SUITE_END()
