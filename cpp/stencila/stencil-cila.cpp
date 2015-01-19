#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/xpressive/xpressive_static.hpp>
#include <boost/xpressive/regex_compiler.hpp>

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
