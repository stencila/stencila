#include <iostream>

#include "../json.hpp"

int main(void){
	using namespace Stencila::Json;
	
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
	
	std::cout<<doc.print();
	std::cout<<doc.pretty();
	
}