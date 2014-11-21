#include <stencila/stencil.hpp>

namespace Stencila {

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
