#include <stencila/stencil.hpp>
#include <stencila/stencil-outline.hpp>

namespace Stencila {

Stencil::Stencil(void){
}

Stencil::Stencil(const std::string& from){
	initialise(from);
}

Stencil::~Stencil(void){
	if(context_) delete context_;
	if(outline_) delete outline_;
}

}
