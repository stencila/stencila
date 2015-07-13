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
 * Is this a "shortable" element type?
 *
 * Elements like <title>, <h1>-<h6>, <li> and <td> are block elements but their contents
 * are not usually indented when they are short.
 * 
 * @param  name Element name
 */
bool is_shortable_element(const std::string& name);

/**
 * A HTML5 document
 */
class Fragment : public Xml::Document {
public:
	/**
	 * Construct a HTML5 fragment from string
	 */
	Fragment(const std::string& html="");

	/**
	 * Construct a HTML5 fragment from a Xml::Document
	 */
	Fragment(const Xml::Document& xml);

	/**
	 * Tidy a string of HTML to ensure it can be parsed
	 * as a XML document
	 *
	 * @param	html A HTML string
	 */
	static std::string tidy(const std::string& html);

	/**
	 * Load the document from a HTML string
	 * 
	 * @param  html A HTML string 
	 */
	Fragment& load(const std::string& html,bool document=false);

	/**
	 * Dump the document to a HTML string
	 */
	std::string dump(bool pretty=true) const;

	/**
	 * Read the document from a file
	 * 
	 * @param  path File system path for file to read from
	 */
	Fragment& read(const std::string& path);

	/**
	 * Write the document to a file
	 * 
	 * @param  path File system path for file to write to
	 */
	Fragment& write(const std::string& path);
};

/**
 * A HTML5 document
 */
class Document : public Fragment {
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
	 * @param  path File system path for file to read from
	 */
	Document& read(const std::string& path);
};

}
}
