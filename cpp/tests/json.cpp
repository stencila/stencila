#include <boost/test/unit_test.hpp>

#include <stencila/json.hpp>

BOOST_AUTO_TEST_SUITE(json)

using namespace Stencila::Json;

BOOST_AUTO_TEST_CASE(get_object){
    Document doc(R"({
        "a": false,
        "b": 42,
        "c": 3.14,
        "d": [1,2,3],
        "e": {
            "e1": 42
        }
    })");
    
    Value& a = doc["a"];
    BOOST_CHECK(is<bool>(a));
    BOOST_CHECK(not is<double>(a));
    BOOST_CHECK(not has(a,"a1"));
    BOOST_CHECK_EQUAL(size(a),0u);

    Value& e = doc["e"];
    BOOST_CHECK(is<Object>(e));
    BOOST_CHECK(not is<double>(e));
    BOOST_CHECK(has(e,"e1"));
    BOOST_CHECK_EQUAL(size(a),0u);

    BOOST_CHECK_EQUAL(as<int>(doc["e"]["e1"]),42);
}

BOOST_AUTO_TEST_CASE(get_array){
    Document doc(R"([4,3,2,1])");
    BOOST_CHECK_EQUAL(size(doc),4u);
    BOOST_CHECK_EQUAL(as<int>(doc[0]),4);
    BOOST_CHECK_EQUAL(as<int>(doc[1]),3);
}

BOOST_AUTO_TEST_CASE(set){
    Document doc;

    doc.append("a","hello");
    BOOST_CHECK(has(doc,"a"));
    BOOST_CHECK(is<std::string>(doc["a"]));
    BOOST_CHECK_EQUAL(as<std::string>(doc["a"]),"hello");

    doc.append("b",std::vector<int>{1,2,3});
    BOOST_CHECK(has(doc,"b"));
    BOOST_CHECK(is<Array>(doc["b"]));
    BOOST_CHECK_EQUAL(size(doc["b"]),3u);
    BOOST_CHECK_EQUAL(as<int>(doc["b"][1]),2);
}

BOOST_AUTO_TEST_CASE(copy){
    Document a = R"({"foo":"bar","list":[1,2,3]})";
    Document b = a;
    BOOST_CHECK_EQUAL(as<std::string>(b["foo"]),"bar");
    BOOST_CHECK_EQUAL(size(b["list"]),3u);
    BOOST_CHECK_EQUAL(as<int>(b["list"][2]),3);
}

BOOST_AUTO_TEST_SUITE_END()
