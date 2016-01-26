#include <stencila/version.hpp>
#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

std::string Stencil::html(bool document, bool pretty) const {
	if (document) {
		// Create a valid HTML document with title and
		// content in body (but without other embellishments produced by page())
		Html::Document doc;
		doc.select("head title").text(title());
		doc.select("body").append(*this);
		return doc.dump(pretty);
	} else {
		// Return content only
		// Place into a Html::Fragment
		Html::Fragment frag = *this;
		if(pretty){
			// Clean ids added by frontend
			auto elems = frag.filter("[id]");
			for(auto elem : elems){
				if(elem.attr("id").find("_")!=std::string::npos){
					elem.erase("id");
				}
			}
		}
		auto html = frag.dump(pretty);
		return trim(html);
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
