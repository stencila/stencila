#pragma once

#include <tidy-html5/tidy.h>
#include <tidy-html5/buffio.h>

#include <stencila/utilities/xml.hpp>

namespace Stencila {
namespace Utilities {
namespace Html {

/**
 * @namespace Html
 *
 * Stencila's interface to HTML5
 * 
**/

typedef Xml::Attribute Attribute;
typedef Xml::Node Node;

/**
 * HTML document
 *
 * Conform to [Polyglot markup](http://www.w3.org/TR/html-polyglot/) (is both HTML5 and XML; some people call it XHTML5)
 * There is a summary of what XHTML5 requires [here](http://blog.whatwg.org/xhtml5-in-a-nutshell).
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
	        int status = tidySetErrorBuffer(document, &error_buffer);
	        if(status >= 0) status = tidyParseString(document,html.c_str());
	        if(status >= 0) status = tidyCleanAndRepair(document);
	        if(status >= 0) status = tidyRunDiagnostics(document);

	        TidyBuffer output_buffer = {0};
	        if(status>=0) status = tidySaveBuffer(document, &output_buffer);
	        
	        std::stringstream output_stream;
	        output_stream<<output_buffer.bp;
	        std::string output = output_stream.str();
	        tidyBufFree(&output_buffer);
	        
	        std::stringstream error_stream;
	        error_stream<<error_buffer.bp;
	        std::string error = error_stream.str();
	        tidyBufFree(&error_buffer);
	        
	        tidyRelease(document);
	        
	        if(status>=0){
	            int errors = tidyErrorCount(document);
	            if(errors>0) {
	                STENCILA_THROW(Exception,error);
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

    /**
     * Load the document from an XML string
     * 
     * @param  xml 
     */
    Document& load(const std::string& html){
        Xml::Document::load(tidy(html));
        
        // Make changes to the document so it conforms to [polyglot markup](http://dev.w3.org/html5/html-polyglot/html-polyglot.html)...

        // tidy-html5 does not add a DOCTYPE declaration even when `TidyXhtmlOut` is `yes`
        // So add one here..
        doctype("html");

        // Add charset if it does not yet exist
        Node head = find("head");
        if(not head.find("meta","charset")) head.append("meta",{{"charset","UTF-8"}});

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
}
