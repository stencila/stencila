#include <stencila/stencil.hpp>

#include <iostream>

namespace Stencila {

Stencil::Input::Input(void){
}

Stencil::Input::Input(Node node){
	parse(node);
}

void Stencil::Input::parse(Node node){
	name = node.attr("name");
	type = node.attr("type");
	value = node.attr("value");
}

void Stencil::Input::render(Stencil& stencil, Node node, std::shared_ptr<Context> context){
	parse(node);

	// Update and check hash
	auto hash = stencil.hash(node);
	if(hash==node.attr("data-hash")) return;
	else {
		node.attr("data-hash",hash);
		context->input(name,type,value);
	}
}		

Stencil& Stencil::inputs(const std::map<std::string,std::string>& inputs) {
	for(auto input : inputs){
		auto name = input.first;
		Node elem = select("input[name="+name+"]");
		if(elem){
			auto value = input.second;
			elem.attr("value",value);
		}
	}
	return *this;
}

}
