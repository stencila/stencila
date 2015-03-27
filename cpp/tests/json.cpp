#include <boost/test/unit_test.hpp>

#include <stencila/json.hpp>

BOOST_AUTO_TEST_SUITE(json_quick)

using namespace Stencila::Json;

BOOST_AUTO_TEST_CASE(construct){
	Document a(R"([{"a":1},4])");
	// Construct a document from a node in another document
	Document b(a[0].dump());
	BOOST_CHECK_EQUAL(a.dump(),"[{\"a\":1},4]");
	BOOST_CHECK_EQUAL(b.dump(),"{\"a\":1}");
}

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
	
	Node a = doc["a"];
	BOOST_CHECK(a.is<bool>());
	BOOST_CHECK(not a.is<double>());
	BOOST_CHECK(not a.has("a1"));
	BOOST_CHECK_EQUAL(a.size(),0u);

	Node e = doc["e"];
	BOOST_CHECK(e.is<Object>());
	BOOST_CHECK(not e.is<double>());
	BOOST_CHECK(e.has("e1"));
	BOOST_CHECK_EQUAL(e.size(),1u);

	BOOST_CHECK_EQUAL(doc["e"]["e1"].as<int>(),42);
}

BOOST_AUTO_TEST_CASE(get_array){
	Document doc(R"([4,3,2,1])");
	BOOST_CHECK_EQUAL(doc.size(),4u);
	BOOST_CHECK_EQUAL(doc[0].as<int>(),4);
	BOOST_CHECK_EQUAL(doc[1].as<int>(),3);
}

BOOST_AUTO_TEST_CASE(append){
	Document doc;

	doc.append("a","hello");
	BOOST_CHECK(doc.has("a"));
	BOOST_CHECK(doc["a"].is<std::string>());
	BOOST_CHECK_EQUAL(doc["a"].as<std::string>(),"hello");

	doc.append("b",std::vector<int>{1,2,3});
	BOOST_CHECK(doc.has("b"));
	BOOST_CHECK(doc["b"].is<Array>());
	BOOST_CHECK_EQUAL(doc["b"].size(),3u);
	BOOST_CHECK(doc["b"][1].is<int>());
	BOOST_CHECK_EQUAL(doc["b"][1].as<int>(),2);

	std::map<std::string,std::string> map;
	map["a"] = "a";
	map["b"] = "b";
	map["c"] = "c";

	doc.append("c",map);
	BOOST_CHECK(doc.has("c"));
	BOOST_CHECK(doc["c"].is<Object>());
	BOOST_CHECK_EQUAL(doc["c"].size(),3u);
	BOOST_CHECK(doc["c"]["a"].is<std::string>());
	BOOST_CHECK_EQUAL(doc["c"]["a"].as<std::string>(),"a");
}

BOOST_AUTO_TEST_CASE(copy){
	Document a = R"({"foo":"bar","list":[1,2,3]})";
	Document b = a;
	BOOST_CHECK_EQUAL(b["foo"].as<std::string>(),"bar");
	BOOST_CHECK_EQUAL(b["list"].size(),3u);
	BOOST_CHECK_EQUAL(b["list"][2].as<int>(),3);
}

BOOST_AUTO_TEST_CASE(load_dump){
	std::string json = R"({"foo":"bar","list":[1,2,3]})";
	Document doc;
	doc.load(json);
	BOOST_CHECK_EQUAL(doc.dump(),json);
}

BOOST_AUTO_TEST_SUITE_END()
