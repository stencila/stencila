//! @file html.hpp
//! @brief Classes and functions for working with HTML
//! @author Nokome Bentley

#pragma once

#include <tidy-html5/tidy.h>
#include <tidy-html5/buffio.h>

#include <stencila/xml.hpp>
#include <stencila/exception.hpp>

namespace Stencila {
namespace Html {

typedef Xml::Node Node;
typedef Xml::Attribute Attribute;

static std::string tidy(const std::string& html){
    // Create a tidyhtml5 document
    TidyDoc document = tidyCreate();
    // Set processing options.
    // For a list of options see http://w3c.github.io/tidy-html5/quickref.html
    // For C names of options see tidy-html5-master/src/config.c
    bool ok = false;
    // Do not drop attributes or elements (otherwise elems that have meaning in
    // an unrendered stencil get lost)
    ok = tidyOptSetBool(document,TidyDropPropAttrs,no);
    ok = tidyOptSetBool(document,TidyDropFontTags,no);
    ok = tidyOptSetBool(document,TidyDropEmptyElems,no);
    ok = tidyOptSetBool(document,TidyDropEmptyParas,no);
    // Ouput XHTML
    ok = tidyOptSetBool(document,TidyXhtmlOut,yes);
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
            else return output;
        } else {
            STENCILA_THROW(Exception,"An error occurred");
        }
    }
    STENCILA_THROW(Exception,"An error occurred");
}

class Document : public Xml::Document {
public:

    Document(void){
    }

    Document(const std::string& html){
        load(tidy(html));
    }
};

}
}
