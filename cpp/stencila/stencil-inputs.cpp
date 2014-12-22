#include <stencila/stencil.hpp>

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

void Stencil::Input::render(Stencil& stencil, Node node, Context* context){
	parse(node);
    context->input(name,type,value);
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
