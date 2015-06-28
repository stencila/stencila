#pragma once

#include <fstream>
#include <memory>
#include <vector>

#include <stencila/exception.hpp>

namespace pugi {
	class xml_attribute;
	class xml_node;
	class xml_document;
}

namespace Stencila {
namespace Xml {

/**
 * @namespace Stencila::Xml
 *
 * Stencila's interface to <a href="http://en.wikipedia.org/wiki/XML/">Extensible Markup Language (XML)</a>.
 *
 * The Stencila library currently uses <a href="http://pugixml.org/">pugixml</a> as a backend for XML parsing and generation.
 * We chose pugixml because it is fast and has XPath support.
 * Documentation for pugixml is available <a href="http://pugixml.org/documentation/">here</a>.
 *
 * There is a brief, but very good, guide on how to choose a XML library <a href="http://stackoverflow.com/questions/9387610/what-xml-parser-should-i-use-in-c">here</a>.
 * It summarises many of the conclusions that we came to in choosing pugixml for Stencila.
**/

typedef std::pair<std::string,std::string> Attribute;
typedef std::vector<Attribute> Attributes;

class Node;
typedef std::vector<Node> Nodes;

class Document;

typedef std::vector<std::pair<std::string,std::vector<std::string>>> Whitelist;

/**
 * XML node
 *
 * This class wraps `pugi::xml_node` to change the programming interace.
 * It adds convienient methods for appending elements to a node e.g. `elem.append("div",{{"class","greeting"},{"id","hello"}},"Hello world")`.
 * Where appropriate it reduces the length of method names (e.g `next_sibling()` becomes `next()`) and, to some extent, makes them consitent 
 * with the [JQuery API](https://api.jquery.com/) (e.g. `attribute("class")` becomes `attr("class")`).
 */
class Node {
public:

	Node(void);

	Node(const pugi::xml_node& node);

	Node(const Node& node);

	~Node(void);

	/**
	 * Does this Node exist in the Document?
	 */
	bool exists(void) const;

	/**
	 * Does this Node exist in the Document?
	 */
	operator bool(void) const {
		return exists();
	}

	/**
	 * Does this Node not exist in the Document?
	 */
	bool operator!(void) const {
		return not exists();
	}

	/**
	 * @name Attribute retreival and modification
	 * @{
	 */

	/**
	 * Is this a document node?
	 */
	bool is_document(void) const;
	
	/**
	 * Is this a DOCTYPE node?
	 */
	bool is_doctype(void) const;

	/**
	 * Is this an element node?
	 */
	bool is_element(void) const;

	/**
	 * Is this a text node?
	 */
	bool is_text(void) const;

	/**
	 * Is this a CDATA node?
	 */
	bool is_cdata(void) const;

	/**
	 * Get the tag name of this node e.g. 'div'
	 */
	std::string name(void) const;

	/**
	 * Has an attribute?
	 */
	bool has(const std::string& name) const;

	/**
	 * Get an attribute
	 *
	 * Returns an empty string if the attribute does not exist
	 */
	std::string attr(const std::string& name) const;

	/**
	 * Set an attribute
	 *
	 * Sets the value of the existing atribute or appends a new
	 * attribute with `value` if it does not
	 */
	Node& attr(const std::string& name,const std::string& value);

	/**
	 * Get a list of attribute names
	 */
	std::vector<std::string> attrs(void) const;

	/**
	 * Concatenate a string to an existing value (if any) of an attribute
	 *
	 * If the attribute exists, `value`, prefixed with `separator`, will be appended to the
	 * current value. If it does not then set the attribute i.e. same as `attr(name,value)`
	 * 
	 * @param  name  Name of attribute
	 * @param  value String to add
	 * @param  separator Separator between existing string and value
	 */
	Node& concat(const std::string& name, const std::string& value, const std::string& separator=" ");

	/**
	 * Remove an attribute
	 * 
	 * @param  name  Name of attribute
	 */
	Node& erase(const std::string& name);

	/**
	 * @}
	 */
	
	/**
	 * @name Text retrieval and manipulation
	 * @{
	 */

	/**
	 * Get the node's text
	 */
	std::string text(void) const;

	/**
	 * Set the node's text
	 */
	Node& text(const std::string& text);

	/**
	 * @}
	 */

	/**
	 * @name Node manipulation
	 * @{
	 */
	
	/**
	 * Append a node
	 * 
	 * @param  node A XML node
	 */
	Node append(const Node& node);

	/**
	 * Append a document
	 * 
	 * @param  doc A XML document
	 */
	Node append(const Document& doc);

	/**
	 * Append an element node
	 * 
	 * @param  tag Tag name (e.g. "div")
	 */
	Node append(const std::string& tag);

	/**
	 * Append an element node with text content
	 * 
	 * @param  tag  Tag name
	 * @param  text Text content
	 */
	Node append(const std::string& tag, const std::string& text);

	/**
	 * Append an element node having attributes and, optionally, text content
	 * 
	 * @param  tag        Tag name
	 * @param  attributes List of attributes
	 * @param  text       Text content
	 */
	Node append(const std::string& tag, const Attributes& attributes, const std::string& text = "");

	/**
	 * Append a text node
	 * 
	 * @param  text Text content
	 */
	Node append_text(const std::string& text);

	/**
	 * Append a CDATA node
	 * 
	 * @param  data Character data
	 */
	Node append_cdata(const std::string& cdata);

	/**
	 * Append a comment
	 * 
	 * @param  comment Comment
	 */
	Node append_comment(const std::string& comment);
	
	/**
	 * Append XML
	 *
	 * Parse the supplied XML and append the resulting node tree
	 * 
	 * @param xml A XML string
	 */
	Node append_xml(const std::string& xml);

	/**
	 * Append the children of another node
	 * 
	 * @param  child Child node
	 */
	Node& append_children(const Node& other);

	/**
	 * Prepend a node
	 */
	Node prepend(Node node);

	/**
	 * Prepend an element node
	 * 
	 * @param  tag Tag name (e.g. "div")
	 */
	Node prepend(const std::string& tag);

	/**
	 * Prepend an element node with text content
	 * 
	 * @param  tag  Tag name
	 * @param  text Text content
	 */
	Node prepend(const std::string& tag, const std::string& text);

	/**
	 * Prepend an element node having attributes and, optionally, text content
	 * 
	 * @param  tag        Tag name
	 * @param  attributes List of attributes
	 * @param  text       Text content
	 */
	Node prepend(const std::string& tag, const Attributes& attributes, const std::string& text = "");

	/**
	 * Prepend the children of another node
	 * 
	 * @param  other Other node
	 */
	Node& prepend_children(const Node& other);

	/**
	 * Insert a node before this one
	 */
	Node before(Node node);

	/**
	 * Insert a node after this one
	 */
	Node after(Node node);

	/**
	 * Remove a child node
	 * 
	 * @param  child Child node
	 */
	Node& remove(const Node& child);
	
	/**
	 * Clear all child nodes
	 */
	Node& clear(void);

	/**
	 * Append this node to a different parent
	 * 
	 * @param  to New parent of this node
	 */
	Node& move(Node& to);

	/**
	 * Remove this node from its parent
	 */
	void destroy(void);

	/**
	 * @}
	 */
	
	/**
	 * @name Patching
	 *
	 * Allow for changing parts of a XML tree. Uses the patch framework documented in 
	 * [An Extensible Markup Language (XML) Patch Operations Framework Utilizing
     * XML Path Language (XPath) Selectors](https://tools.ietf.org/html/rfc5261)
	 *
	 * Methods implemented in `xml-patch.cpp`
	 * 
	 * @{
	 */
		
	/**
	 * Apply a patch to this node
	 *
	 * @param patch Patch (as `Xml::Node`) to apply
	 */
	Node& patch(const Node& patch);

	/**
	 * Apply a patch to this node
	 *
	 * @param patch Patch (as `std::string`) to apply
	 */
	Node& patch(const std::string& patch);

	/**
	 * @}
	 */
	

	/**
	 * @name Parents
	 * @{
	 */
	
	/**
	 * Get the root node of the document the node belongs to
	 */
	Node root(void);

	/**
	 * Get this node's parent node
	 */
	Node parent(void) const;

	/**
	 * @}
	 */ 
	

	/**
	 * @name Child traversal
	 * @{
	 */
	
	/**
	 * Get all children
	 */
	Nodes children(void) const;

	/**
	 * Get the first child node of this node
	 */
	Node first(void) const;

	/**
	 * Get the first child  of this node that is an element
	 */
	Node first_element(void) const;

	/**
	 * Get the last child node of this node
	 */
	Node last(void) const;

	/**
	 * @}
	 */ 
	
	
	/**
	 * @name Sibling traversal
	 * @{
	 */

	/**
	 * Get the next sibling of this node
	 */
	Node next(void);

	/**
	 * Get the next sibling of this node that is an element
	 */
	Node next_element(void);

	/**
	 * Get the previous sibling of this node
	 */
	Node previous(void);

	/**
	 * @}
	 */ 
	

	/**
	 * @name Node retreival
	 * @{
	 */

	/**
	 * Find the first element with tag
	 * 
	 * @param  tag Tag name
	 */
	Node find(const std::string& tag) const;

	/**
	 * Find the first element with tag and an attribute
	 * 
	 * @param  tag    Tag name
	 * @param  name   Name of attribute
	 */
	Node find(const std::string& tag,const std::string& name) const;

	/**
	 * Find the first element with tag and attribute value
	 * 
	 * @param  tag    Tag name
	 * @param  name   Name of attribute
	 * @param  value  Value of attribute
	 */
	Node find(const std::string& tag,const std::string& name,const std::string& value) const;

	/**
	 * Get the XPath eqivalent of a CSS selector
	 * 
	 * @param  selector CSS selector string
	 */
	static std::string xpath(const std::string& selector);

	/**
	 * Get the first element which matches the selector
	 * 
	 * @param  selector Selector expression
	 * @param  type Type of seletor expression, `"css"` or `"xpath"`
	 */
	Node select(const std::string& selector,const std::string& type="css") const;

	/**
	 * Get all the element which match the selector
	 * 
	 * @param  selector Selector expression
	 * @param  type Type of seletor expression, `"css"` or `"xpath"`
	 */
	Nodes filter(const std::string& selector,const std::string& type="css") const;

	/**
	 * @}
	 */ 
	

	/**
	 * Sanitize the Node using a whitelist of tag names and attributes
	 *
	 * Only elements with names in the whitelist are allowed. Other elements are removed.
	 * Those elements can only have attributes in the whitelist of the coressponding tag name. Other attributes are erased.
	 */
	Node& sanitize(const Whitelist& whitelist);

	/**
	 * @name Serialisation
	 *
	 * See `Document` for loading and reading of XML files
	 * 
	 * @{
	 */
	
	/**
	 * Dump the node to a string
	 * 
	 * @param  indent Turn on indentation?
	 */
	std::string dump(bool indent=false) const;
	
	/**
	 * Dump the node's children to a string
	 * 
	 * @param  indent Turn on indentation?
	 */
	std::string dump_children(bool indent=false) const;

	/**
	 * Write the node to a file
	 * 
	 * @param filename Filename to write
	 * @param indent   Turn on indentation?
	 */
	void write(const std::string& filename,bool indent=false) const;

	/**
	 * @}
	 */ 

protected:

	/**
	 * Get a pugi::xml_attribute for this node
	 */
	pugi::xml_attribute attr_(const std::string& name) const;

	/**
	 * Pointer to implementation
	 *
	 * Usually std::unique_ptr would be used here. But there were errors when using that
	 * so use shared_ptr instead. See http://stackoverflow.com/questions/9020372/how-do-i-use-unique-ptr-for-pimpl
	 */
	std::shared_ptr<pugi::xml_node> pimpl_;
};


/**
 * XML document
 */
class Document : public Node {
public:

	Document(void);
	
	Document(const std::string& html);

	~Document();

	/**
	 * Prepend a document type declaration to the document
	 * 
	 * @param  type Document type
	 */
	Node doctype(const std::string& type);

	/**
	 * Load the document from an XML string
	 * 
	 * @param  xml 
	 */
	Document& load(const std::string& xml);

	/**
	 * Read the document from a file
	 * 
	 * @param  filename Name of file to read from
	 */
	Document& read(const std::string& filename);

private:
	// Pugixml does not allow for copying of `xml_document`s (presumably for efficiency).
	// To have `Document` derive from `Node` (so it inherits the public interface we define above)
	// it is necessary to store a pointer to a `pugi::xml_document`, create it and the copy the `pugi::xml_node`
	// relating to that document (`pugi::xml_node`s are handles to nodes within `pugi::xml_document`s)
	
	pugi::xml_document* doc_(void);
};

}
}
