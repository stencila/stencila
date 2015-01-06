#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::xml(void) const {
	return dump();
}


Stencil& Stencil::xml(const std::string& xml){
	load(xml);
	return *this;
}

} //namespace Stencila
