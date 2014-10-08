#include <boost/algorithm/string/replace.hpp>

#include <pugixml.hpp>

#include <tidy-html5/tidy.h>
#include <tidy-html5/buffio.h>

#include <stencila/html.hpp>

namespace Stencila {
namespace Html {

Document::Document(const std::string& html):
	Xml::Document(){
	// Even for an initally empty document call load
	// with an empty string so that `tidy()` can create
	// the elements necessary in a HTML5 document (e.g. <body>)
	load(html);
}

/**
 * Parse and tidy up a HTML string
 */
std::string Document::tidy(const std::string& html){
	// Create a tidyhtml5 document
	TidyDoc document = tidyCreate();
	// Set processing  and output options.
	// For a list of options see http://w3c.github.io/tidy-html5/quickref.html
	// For C names of options see tidy-html5-master/src/config.c
	bool ok = false;
	// 	Do not drop attributes or elements (otherwise elems that have meaning in
	// 	an unrendered Stencil get lost)
	ok = tidyOptSetBool(document,TidyDropPropAttrs,no);
	ok = tidyOptSetBool(document,TidyDropFontTags,no);
	ok = tidyOptSetBool(document,TidyDropEmptyElems,no);
	ok = tidyOptSetBool(document,TidyDropEmptyParas,no);
	//	Output as XHTML5
	ok = tidyOptSetBool(document,TidyXhtmlOut,yes);
	ok = tidyOptSetValue(document,TidyDoctype,"html5");
	//	Turn off adding a html-tidy <meta name="generator".. tag for itself
	ok = tidyOptSetBool(document,TidyMark,no);
	// Do processing and output
	if(ok){
		// Unfortunately Tidy relaces all tabs with spaces, including in <pre> elements
		// There does not appear to be an option for turning this off (only for changing number of tab spaces)
		// The HTML5 spec does not say that tabs are not allowed in pre elements (http://www.w3.org/TR/html5/grouping-content.html#the-pre-element)
		// Leaving this behaviour on may cause problems with indentation of code.
		// Putting in a character proxy for tabs (e.g. "---tab---" as used below) in other elements can cause Tidy to insert extra
		// element so should be avoided.
		// So, protect tabs in <pre> elements only...
		std::string input = html;
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
			boost::replace_all(pre,"\t","---tab---");
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
			boost::replace_all(output,"---tab---","\t");
			return output;
		} else {
			STENCILA_THROW(Exception,"An error occurred");
		}
	}
	STENCILA_THROW(Exception,"An error occurred");
}   

Document& Document::load(const std::string& html){
	// For some reason tidy does not like a "<!DOCTYPE html>"
	// in the document so remove that first
	std::string html_to_tidy = boost::replace_all_copy(html,"<!DOCTYPE html>","");
	
	std::string tidied = tidy(html_to_tidy);
	// In some cases tidy is returning an empty string
	// this catches that
	if(html_to_tidy.length()>0 and tidied.length()==0){
		STENCILA_THROW(Exception,"No tidied HTML returned");
	}

	// Load the tidied HTML into the document
	Xml::Document::load(tidied);
	
	// tidy-html5 does not add a DOCTYPE declaration even when `TidyXhtmlOut` is `yes` and
	// `TidyDoctype` is `"html5"`. So add one here..
	doctype("html");

	Node head = find("head");

	// Set charset
	// Although it is not technically required to define the character set, failing to do so can leave the page vulnerable to 
	// cross-site scripting attacks in older versions of IE. Note that even in old browsers this short version is equivalent to:
	//   <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
	//  (http://www.coreservlets.com/html5-tutorial/basic-html5-document.html)
	if(not head.find("meta","charset")) {
		head.append("meta",{{"charset","UTF-8"}});
	}

	// Validate this document
	validate();

	return *this;
}

Document& Document::read(const std::string& filename){
	std::ifstream file(filename);
	std::stringstream html;
	html<<file.rdbuf();
	load(html.str());
	return *this;
}

/**
 * A Xml::Document traverser which ensures that the content of the 
 * document conforms to HTML5
 */
struct Validator : pugi::xml_tree_walker {
	virtual bool for_each(pugi::xml_node& node) {
		if(node.type()==pugi::node_element){
			std::string name  = node.name();
			// Check to see if this is a "void element"
			bool voide = false;
			for(auto tag : {"area","base","br","col","embed","hr","img","input",
							"keygen","link","meta","param","source","track","wbr"}){
				if(name==tag){
					voide = true;
					break;
				}
			}
			if(voide){
				// "In the HTML syntax, void elements are elements that always are empty and never have an end tag"
				// Remove all child elements. 
				while(node.first_child()) node.remove_child(node.first_child());
			}
			else {
				// Ensure that other nodes have a least one child so that self-closing tags are not used for them
				if(!node.first_child()) node.append_child(pugi::node_pcdata);
			}
		}
		// Continue traversal
		return true;
	}
};

Document& Document::validate(void){
	Validator validator;
	pimpl_->traverse(validator);
	return *this;
}

}	
}
