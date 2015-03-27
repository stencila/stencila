#include <stencila/stencil.hpp>
#include <stencila/stencil-cila.hpp>

namespace Stencila {

Stencil& Stencil::cila(const std::string& string){
	CilaParser().parse(*this,string);
	return *this;
}

std::string Stencil::cila(void) const {
	return CilaGenerator().generate(*this);
}

}
