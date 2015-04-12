#include <stencila/stencil.hpp>

namespace Stencila {

Stencil::Stencil(void){
}

Stencil::Stencil(const std::string& from){
	initialise(from);
}

Stencil::~Stencil(void){
	if(context_) delete context_;
}

}
