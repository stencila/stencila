#include <boost/algorithm/string.hpp>

#include <pugixml.hpp>

#include <tidy.h>
#include <tidybuffio.h>

#include <stencila/html.hpp>
#include <stencila/string.hpp>

namespace Stencila {
namespace Html {

bool is_void_element(const std::string& name){
	for(auto elem : {
		"area","base","br","col","embed","hr","img","input",
		"keygen","link","meta","param","source","track","wbr"
	}){
		if(name==elem) return true;
	}
	return false;
}

bool is_inline_element(const std::string& name){
	for(auto elem : {
		"b", "big", "i", "small", "tt",
		"abbr", "acronym", "cite", "code", "dfn", "em", "kbd", "strong", "samp", "var",
		"a", "bdo", "br", "img", "map", "object", "q", "script", "span", "sub", "sup",
		"button", "input", "label", "select", "textarea"
	}){
		if(name==elem) return true;
	}
	return false;
}

bool is_shortable_element(const std::string& name){
	for(auto elem : {
		"title",
		"h1","h2","h3","h4","h5","h6","h7",
		"li",
		"th","td"
	}){
		if(name==elem) return true;
	}
	return false;
}

Fragment::Fragment(const std::string& html):
	Xml::Document(){
	load(html);
}

Fragment::Fragment(const Xml::Document& xml):
	Xml::Document(xml){
}

/**
 * Parse and tidy up a HTML string
 */
std::string Fragment::tidy(const std::string& html){
	// Create a tidyhtml5 document
	TidyDoc document = tidyCreate();
	// Set processing  and output options.
	// For a list of options see http://www.htacg.org/tidy-html5/quickref.html
	// For C names of options see tidy-html5-master/src/config.c
	bool ok = false;
	// 	Do not drop attributes or elements (otherwise elems that have meaning in
	// 	an unrendered Stencil get lost)
	ok = tidyOptSetBool(document,TidyDropPropAttrs,no);
	ok = tidyOptSetBool(document,TidyDropFontTags,no);
	ok = tidyOptSetBool(document,TidyDropEmptyElems,no);
	ok = tidyOptSetBool(document,TidyDropEmptyParas,no);
	// Don't wrap lines
	ok = tidyOptSetInt(document,TidyWrapLen,0);
	// Don't add newlines
	ok = tidyOptSetBool(document,TidyVertSpace,no);
	//	Turn off adding a html-tidy <meta name="generator".. tag for itself
	ok = tidyOptSetBool(document,TidyMark,no);
	// Output as well formed XML since that is where is is going to
	ok = tidyOptSetBool(document,TidyXmlOut,yes);
	// Do processing and output
	if(ok){
		std::string input = html;
		// For some reason tidy does not like a "<!DOCTYPE html>"
		// in the document so remove that first
		replace_all(input,"<!DOCTYPE html>","");
		// Unfortunately Tidy relaces all tabs with spaces, including in <pre> elements
		// There does not appear to be an option for turning this off (only for changing number of tab spaces)
		// The HTML5 spec does not say that tabs are not allowed in pre elements (http://www.w3.org/TR/html5/grouping-content.html#the-pre-element)
		// Leaving this behaviour on may cause problems with indentation of code.
		// Putting in a character proxy for tabs (e.g. "---tab---" as used below) in other elements can cause Tidy to insert extra
		// element so should be avoided.
		// So, protect tabs in <pre> elements only...
		std::size_t from = 0;
		while(true){
			// Find start and end tag
			// Start tag is found allowing for zero or more attributes
			std::size_t start = input.find("<pre",from);
			if(start==std::string::npos) break;
			start += 5;
			std::size_t end = input.find("</pre>",start);
			// Extract preformatted text, protect tabs and reinsert
			std::string pre = input.substr(start,end);
			replace_all(pre,"\t","---tab---");
			input.replace(start,end,pre);
			// Continue...
			from = end + 6;
		}

		TidyBuffer error_buffer = {0};
		// `status` will be greater than 1 if there are errors
		// in the `tidyParseString`
		int status = tidySetErrorBuffer(document, &error_buffer);
		if(status >= 0) status = tidyParseString(document,input.c_str());
		if(status >= 0) status = tidyCleanAndRepair(document);
		if(status >= 0) status = tidyRunDiagnostics(document);

		TidyBuffer output_buffer = {0};
		if(status>=0) status = tidySaveBuffer(document, &output_buffer);

		std::stringstream error_stream;
		error_stream<<error_buffer.bp;
		std::string error = error_stream.str();
		tidyBufFree(&error_buffer);
		
		std::string output;
		// Do not attempt to obtain output if there has been an 
		// error in parsing
		if(status==1){
			std::stringstream output_stream;
			output_stream<<output_buffer.bp;
			output = output_stream.str();
			tidyBufFree(&output_buffer);
		}

		tidyRelease(document);
		
		if(status>=0){
			int errors = tidyErrorCount(document);
			if(errors>0) {
				STENCILA_THROW(Exception,"Parsing error: "+error);
			}
			// Reinstate tabs
			replace_all(output,"---tab---","\t");
			return output;
		} else {
			STENCILA_THROW(Exception,"An error occurred");
		}
	}
	STENCILA_THROW(Exception,"An error occurred");
}   

Fragment& Fragment::load(const std::string& html, bool document){
	std::string tidied = tidy(html);

	// In some cases tidy is returning an empty string
	// this catches that
	if(html.length()>0 and tidied.length()==0){
		STENCILA_THROW(Exception,"No tidied HTML returned");
	}

	if(not document){
		// Just copy tidied body
		Xml::Document temp(tidied);
		clear();
		append_children(temp.find("body"));
	} else {
		// Load the entire tidied HTM document
		Xml::Document::load(tidied);
	}

	// Tidy inserts new lines at start and end of pre and script tags.
	// See https://github.com/htacg/tidy-html5/issues/158 and https://github.com/htacg/tidy-html5/issues/227
	// For us the main issue is the newlines in inline script elements for MathJax, so just deal with these
	// Note that type="math/asciimath; mode=display" element are not filtered by this select (which is the desired behaviour)
	for(auto node : filter("script[type='math/asciimath'],script[type='math/tex']")){
		auto text = node.text();
		if(text.front()=='\n') text.erase(0,1);
		if(text.back()=='\n') text.pop_back();
		node.text(text);
	}

	// Unfortunately, TidyHTML seems to unecessarily add in <li> elements when there is whitespace within a <ul> or <ol>
	// This is a temporary fix, it's here becasue it is the easiest place for it.
	for(auto node : filter("li[style='list-style: none']")) node.destroy();

	return *this;
}

namespace {
	
void dump_node(std::stringstream& stream, Html::Node node, bool pretty, const std::string& indent=""){
	if(node.is_document()){
		// Dump children without indent
		for(auto child : node.children()) dump_node(stream,child,pretty,"");
	}
	else if(node.is_doctype()){
		stream<<"<!DOCTYPE html>";
	}
	else if(node.is_element()){
		auto name = node.name();
		auto block = not is_inline_element(name);
		
		// Dump start tag with attributes
		if(pretty and block) stream<<"\n"<<indent;
		stream<<"<"<<name;
		for(auto name : node.attrs()){
			auto value = node.attr(name);
			// Escape quotes in attribute values
			boost::replace_all(value,"\"","&quot;");
			stream<<" "<<name<<"=\""<<value<<"\"";
		}
		stream<<">";

		// For void element nothing else to do so return
		if(is_void_element(name)) return;

		// Can this element be "shortend" (not presented as a block)
		bool shorten = false;
		auto children = node.children();
		if(children.size()==0) shorten = true;
		else if(is_shortable_element(name)){
			shorten = true;
			for(auto child : children){
				if(not child.is_text() or child.text().length()>100){
					shorten = false;
					break;
				}
			}
		}

		// Are internal newlines required?
		bool newlines = pretty and block and not shorten and name!="pre";
		
		// Dump child nodes
		bool previous_was_block = block;
		for(auto child : children){
			bool is_inline = child.is_text() or is_inline_element(child.name());
			if(newlines and is_inline and previous_was_block) stream<<"\n"<<indent+"\t";
			dump_node(stream,child,pretty,indent+"\t");
			previous_was_block = not is_inline;
		}

		// Closing tag
		if(newlines) stream<<"\n"<<indent;
		stream<<"</"<<name<<">";
	}
	else if(node.is_text()){
		// Escape & and < in text.
		// Note that this is will incorrectly escape already escaped values in the text e.g. if someone has used &gt;
		// That _may_ actually be the desired behaviour
		auto text = node.text();
		boost::replace_all(text,"&","&amp;");
		boost::replace_all(text,"<","&lt;");
		boost::replace_all(text,">","&gt;");
		stream<<text;
	}
	else if(node.is_cdata()){
		// Note that this currently does not include the "<![CDATA[" prefix and the "]]>" suffix
		stream<<node.text();
	}
}

}

std::string Fragment::dump(bool pretty) const {
	std::stringstream html;
	dump_node(html,*this,pretty);
	return trim(html.str());
}

Fragment& Fragment::read(const std::string& path){
	std::ifstream file(path);
	std::stringstream html;
	html<<file.rdbuf();
	load(html.str());
	return *this;
}

Fragment& Fragment::write(const std::string& path){
	std::ofstream file(path);
	file<<dump();
	return *this;
}

Document::Document(const std::string& html):
	Fragment(){
	// Even for an empty string call load since 
	// tidy() sets up the document structure
	load(html);
}

Document& Document::load(const std::string& html){
	Fragment::load(html,true);

	// Add a DOCTYPE declaration
	doctype("html");

	// Add necessary elements to head
	Node head = find("head");
	// Set charset
	// Although it is not technically required to define the character set, failing to do so can leave the page vulnerable to 
	// cross-site scripting attacks in older versions of IE. Note that even in old browsers this short version is equivalent to:
	//   <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
	//  (http://www.coreservlets.com/html5-tutorial/basic-html5-document.html)
	if(not head.find("meta","charset")) {
		head.append("meta",{{"charset","utf-8"}});
	}

	return *this;
}

Document& Document::read(const std::string& path){
	std::ifstream file(path);
	std::stringstream html;
	html<<file.rdbuf();
	load(html.str());
	return *this;
}

}	
}
