/*
Copyright (c) 2012, Stencila Ltd
Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#include <iostream>

#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/formats/json.hpp>

BOOST_AUTO_TEST_SUITE(formats_json)

BOOST_AUTO_TEST_CASE(general){
	using namespace Stencila::Formats::Json;
	
	Document doc("{"
	"	\"answer\" : 42,		"
	"	\"pi\" : 3.14,			"
	"	\"name\" : \"frank\", 		"
	"	\"int_array\" : [0,1,2,3,4] ,		"
	"	\"a\" : {		"
	"		\"a\" : \"a_a\"		"
	"	}	"
	"}");
	
	Value& a = doc["a"];
	
	doc.is<Object>();
	doc.is<Array>(doc["int_array"]);
	doc.is<double>(doc["pi"]);
	
	doc.has("answer");
	doc.has(a,"a");
	
	doc.as<int>(doc["answer"]);
	doc.as<double>(doc["pi"]);
	doc.as<std::string>(doc["name"]);
	doc.as<std::vector<int>>(doc["int_array"]);
	
	doc.add("email","me@example.com");
	
	doc.as<std::string>(a["a"]);
	doc.add(a,"b","a_b");
	doc.has(a,"b");
}

BOOST_AUTO_TEST_SUITE_END()