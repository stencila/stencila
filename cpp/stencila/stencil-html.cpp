#include <stencila/version.hpp>
#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

std::string Stencil::html(bool document,bool indent) const {
	if(not document){
		// Return content only
		return Xml::Document::dump(indent);
	} else {
		// Return a complete HTML document
		return page();
	}
}

Stencil& Stencil::html(const std::string& html){
	// Clear content before appending new content from Html::Document
	clear();
	Html::Document doc(html);
	auto body = doc.find("body");
	if(auto elem = body.find("main","id","content")){
		append_children(elem);
	}
	else append_children(doc.find("body"));
	return *this;
}

} //namespace Stencila
