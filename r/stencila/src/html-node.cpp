#include <stencila/html.hpp>
using namespace Stencila;

#include "stencila.hpp"

STENCILA_R_FUNC HtmlNode_attr(SEXP self, SEXP name){
	STENCILA_R_BEGIN
		return wrap(from<Html::Node>(self).attr(
			as<std::string>(name)
		));
	STENCILA_R_END
}

STENCILA_R_FUNC HtmlNode_text(SEXP self){
	STENCILA_R_BEGIN
		return wrap(from<Html::Node>(self).text());
	STENCILA_R_END
}

STENCILA_R_FUNC HtmlNode_select(SEXP self,SEXP selector){
	STENCILA_R_BEGIN
		Html::Node* node = new Html::Node;
		*node = from<Html::Node>(self).select(
			as<std::string>(selector)
		);
		return to<Html::Node>(node,"HtmlNode");
	STENCILA_R_END
}
