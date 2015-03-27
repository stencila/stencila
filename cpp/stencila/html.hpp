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
typedef Xml::Attributes Attributes;
typedef Xml::Node Node;
typedef Xml::Nodes Nodes;
typedef Xml::Whitelist Whitelist;

/**
 * Is this a void element type
 *
 * @param  name Element name
 */
bool is_void_element(const std::string& name);

/**
 * Is this an inline element type?
 * 
 * @param  name Element name
 */
bool is_inline_element(const std::string& name);

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
	 * Dump the document to a HTML string
	 */
	std::string dump(bool pretty=true) const;

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
