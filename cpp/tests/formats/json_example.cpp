#include <iostream>

#include <stencila/formats/json.hpp>

int main(void){
	using namespace Stencila::Formats::Json;
	
	// Create a document (note backslahes to escape quotes in string)...
	Document doc("{\"name\":\"pi\"}");
	
	// Add a member ...
	doc.add("value",3.14159);
	// check the member got added...
	doc.has("value");
	// check the member is a double...
	doc.is<double>(doc["value"]);
	// get the member as a double..
	double pi = doc.as<double>(doc["value"]);
	
	// Add an array member...
	doc.add("see",std::vector<std::string>{
		"http://en.wikipedia.org/wiki/Pi",
		"http://www.wolframalpha.com/input/?i=pi"
	});
	
	// Print it out, as one long string...
	std::cout<<doc.print();
	// ,or with indentation...
	std::cout<<doc.pretty();
}