#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::xml(void) const {
	return dump();
}


Stencil& Stencil::xml(const std::string& xml){
	// The input XML may be a fragment (e.g. just text, not as a child of a node)
	// So that this is parsed properly wrap it and then extract.
	Xml::Document doc("<stencil>"+xml+"</stencil>");
	clear();
	for(auto child : doc.select("./stencil","xpath").children()) append(child);
	return *this;
}

} //namespace Stencila
