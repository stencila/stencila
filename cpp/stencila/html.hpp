#pragma once

#include <tidy-html5/tidy.h>
#include <tidy-html5/buffio.h>

#include <stencila/xml.hpp>

namespace Stencila {
namespace Html {

/**
 * @namespace Html
 *
 * Stencila's interface to HTML5
 * 
**/

typedef Xml::Attribute Attribute;
typedef Xml::AttributeList AttributeList;
typedef Xml::Node Node;
typedef Xml::Whitelist Whitelist;

/**
 * A HTML5 document
 *
 * Attempts to conform to [Polyglot markup](http://www.w3.org/TR/html-polyglot/) (is both HTML5 and XML; some people call it XHTML5)
 */
class Document : public Xml::Document {

private:

	/**
	 * Parse and tidy up a HTML string
	 */
	static std::string tidy(const std::string& html){
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
	    //	Output as XHTML
	    ok = tidyOptSetBool(document,TidyXhtmlOut,yes);
	    //	Turn off adding a html-tidy <meta name="generator".. tag for itself
	    ok = tidyOptSetBool(document,TidyMark,no);
	    // Do processing and output
	    if(ok){
	        TidyBuffer error_buffer = {0};
	        // `status` will be greater than 1 if there are errors
	        // in the `tidyParseString`
	        int status = tidySetErrorBuffer(document, &error_buffer);
	        if(status >= 0) status = tidyParseString(document,html.c_str());
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
	            return output;
	        } else {
	            STENCILA_THROW(Exception,"An error occurred");
	        }
	    }
	    STENCILA_THROW(Exception,"An error occurred");
	}    

public:

	Document(void):
		Xml::Document(){
		// Even for an initally empty document call load
		// with an empty string so that `tidy()` can create
		// the elements necessary in a HTML5 document (e.g. <body>)
		load("");
	}

	Document(const std::string& html):
		Xml::Document(){
		load(html);
	}


private:

	/**
	 * A Xml::Document traverser which ensures that the content of the document conforms to 
	 * HTML5
	 */
	struct Validator : pugi::xml_tree_walker {
	    virtual bool for_each(pugi::xml_node& node) {
	        if(node.type()==pugi::node_element){
	        	std::string name  = node.name();
	        	// Check to see if this is a "void element"
	        	bool voide = false;
	        	for(auto tag : {"area","base","br","col","embed","hr","img","input","keygen","link","meta","param","source","track","wbr"}){
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

public:

    /**
     * Load the document from a HTML string
     * 
     * @param  html A html5 string 
     */
    Document& load(const std::string& html){
        std::string tidied = tidy(html);
        // In some cases tidy is returning an empty string
        // this is a temporary kludge to avoid that.
        // FIXME
        if(tidied.length()>0) Xml::Document::load(tidied);
        else Xml::Document::load(html);
        
        // tidy-html5 does not add a DOCTYPE declaration even when `TidyXhtmlOut` is `yes`
        // So add one here..
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

        // Validate the content of the document elements
        Validator walker;
		traverse(walker);

        return *this;
    }

    /**
     * Read the document from a file
     * 
     * @param  filename Name of file to read from
     */
    Document& read(const std::string& filename){
    	std::ifstream file(filename);
    	std::stringstream html;
    	html<<file.rdbuf();
        load(html.str());
        return *this;
    }

};

}
}
