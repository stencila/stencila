#pragma once

#include <stencila/xml.hpp>

namespace Stencila {
namespace Html {

/**
 * @namespace Stencila::Html
 *
 * Stencila's interface to HTML5
 * 
**/

typedef Xml::Attribute Attribute;
typedef Xml::AttributeList AttributeList;
typedef Xml::Node Node;
typedef Xml::Nodes Nodes;
typedef Xml::Whitelist Whitelist;

/**
 * A HTML5 document
 *
 * Attempts to conform to [Polyglot markup](http://www.w3.org/TR/html-polyglot/) 
 * (is both HTML5 and XML; some people call it XHTML5)
 */
class Document : public Xml::Document {
public:
	/**
	 * Construct a HTML5 document
	 */
	Document(const std::string& html="");

    /**
     * Load the document from a HTML string
     * 
     * @param  html A HTML string 
     */
    Document& load(const std::string& html);

    /**
     * Read the document from a file
     * 
     * @param  filename Name of file to read from
     */
    Document& read(const std::string& filename);

	/**
	 * Tidy a string of HTML to ensure it can be parsed
	 * as a XML document
	 *
	 * @param	html A HTML string
	 */
	static std::string tidy(const std::string& html);

	/**
	 * Validate the document to ensure it conforms to HTML5
	 */
	Document& validate(void);
};

}
}
