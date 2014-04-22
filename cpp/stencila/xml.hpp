#pragma once

#include <fstream>
#include <vector>

#include <pugixml.hpp>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Xml {

/**
 * @namespace Xml
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

class Node;
class Document;

/**
 * XML node attribute
 */
class Attribute : protected pugi::xml_attribute {

    // protected inheritance from pugi::xml_attribute so
    // public interface can be controlled better
    
private:

	friend class Node;

public:

	Attribute(const pugi::xml_attribute& attr):
		pugi::xml_attribute(attr){
	}

    // Logical operators used for determining if
    // an attribute is present on a node
	operator bool(void) const {
		return not empty();
	}
	bool operator!(void) const {
		return empty();
	}

};

typedef std::vector<std::pair<std::string,std::string>> AttributeList;

typedef std::vector<Node> Nodes;

typedef std::vector<std::pair<std::string,std::vector<std::string>>> Whitelist;

/**
 * XML node
 *
 * This class wraps `pugi::xml_node` to change the programming interace.
 * It adds convienient methods for appending elements to a node e.g. `elem.append("div",{{"class","greeting"},{"id","hello"}},"Hello world")`.
 * Where appropriate it reduces the length of method names (e.g `next_sibling()` becomes `next()`) and makes them consitent 
 * with the [JQuery API](https://api.jquery.com/) (e.g. `attribute("class")` becomes `attr("class")`).
 */
class Node : protected pugi::xml_node {

    // protected inheritance from pugi::xml_attribute so
    // public interface can be controlled better

public:

	Node(void){
	}

	Node(const pugi::xml_node& node):
		pugi::xml_node(node){
	}

    /**
     * Logical operators used for determining if a node is 
     * present in parent node (see `find`() method)
     */
	operator bool(void) const {
		return not empty();
	}
	bool operator!(void) const {
		return empty();
	}

	/**
	 * @name Attribute retreival and modification
	 * @{
	 */

private:

    // Private method for getting a node attribute,
    // may return an empty attribute
	Attribute attr_get_(const std::string& name) const {
		return find_attribute([&name](Attribute attr){
        	return attr.name()==name;
    	});
	}

public:

    /**
     * Is this a document node?
     */
    bool is_document(void) const {
        return type()==pugi::node_document;
    }

    /**
     * Is this an element node?
     */
    bool is_element(void) const {
        return type()==pugi::node_element;
    }

    /**
     * Is this text node?
     */
    bool is_text(void) const {
        return type()==pugi::node_pcdata;
    }

    /**
     * Get the tag name of this node e.g. 'div'
     */
    std::string name(void) const {
        return pugi::xml_node::name();
    }

    /**
     * Has an attribute?
     */
    bool has(const std::string& name) const {
        Attribute attr = attr_get_(name);
        if(attr) return true;
        else return false;
    }

    /**
     * Get an attribute
     *
     * Returns an empty string if the attribute does not exist
     */
	std::string attr(const std::string& name) const {
		Attribute attr = attr_get_(name);
		if(attr) return attr.value();
		else return "";
	}

    /**
     * Set an attribute
     *
     * Sets the value of the existing atribute or appends a new
     * attribute with `value` if it does not
     */
	Node& attr(const std::string& name,const std::string& value){
        Attribute attr = attr_get_(name);
        if(attr) attr.set_value(value.c_str());
        else append_attribute(name.c_str()) = value.c_str();
		return *this;
	}

    /**
     * Get a list of attribute names
     */
    std::vector<std::string> attrs(void) const {
        std::vector<std::string> attrs;
        for(pugi::xml_attribute attr = first_attribute(); attr; attr = attr.next_attribute()){
            attrs.push_back(attr.name());
        }
        return attrs;
    }

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
	Node& concat(const std::string& name, const std::string& value, const std::string& separator=" "){
        Attribute attr = attr_get_(name);
        if(attr){
            std::string current = attr.as_string();
            std::string future;
            if(current.length()>0) future = current + separator + value;
            else future = value;
            attr.set_value(future.c_str());
        }else {
            append_attribute(name.c_str()) = value.c_str();
        }
        return *this;
    }

    /**
     * Remove an attribute
     * 
     * @param  name  Name of attribute
     */
	Node& erase(const std::string& name){
		Attribute attr = attr_get_(name);
        if(attr) remove_attribute(attr);
		return *this;
	}

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
    std::string text(void) const {
        return pugi::xml_node::text().get();
    }

    /**
     * Set the node's text
     */
    Node& text(const std::string& text) {
        pugi::xml_node::text().set(text.c_str());
        return *this;
    }

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
	Node append(const Node& node) {
        return append_copy(node);
    }

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
	Node append(const std::string& tag) {
        return append_child(tag.c_str());
    }

    /**
     * Append an element node with text content
     * 
     * @param  tag  Tag name
     * @param  text Text content
     */
    Node append(const std::string& tag, const std::string& text) {
        Node child = append(tag);
        child.append_child(pugi::node_pcdata).set_value(text.c_str());
        return child;
    }

    /**
     * Append an element node having attributes and, optionally, text content
     * 
     * @param  tag        Tag name
     * @param  attributes List of attributes
     * @param  text       Text content
     */
	Node append(const std::string& tag, const AttributeList& attributes, const std::string& text = "") {
        Node child = append(tag);
        typedef std::pair<std::string,std::string> Attribute;
        for(Attribute attribute : attributes){
            child.append_attribute(attribute.first.c_str()) = attribute.second.c_str();
        }
        if(text.length()>0) child.append_child(pugi::node_pcdata).set_value(text.c_str());
        return child;
    }

    /**
     * Append a text node
     * 
     * @param  text Text content
     */
    Node append_text(const std::string& text){
        Node child = append_child(pugi::node_pcdata);
        child.text(text);
        return child;
    }

    /**
     * Append a CDATA node
     * 
     * @param  data Character data
     */
    Node append_cdata(const std::string& cdata){
        Node child = append_child(pugi::node_cdata);
        child.text(cdata);
        return child;
    }

    /**
     * Append a comment
     * 
     * @param  comment Comment
     */
    Node append_comment(const std::string& comment){
        Node child = append_child(pugi::node_comment);
        child.set_value(comment.c_str());
        return child;
    }
    
    /**
     * Append XML
     *
     * Parse the supplied XML and append the resulting node tree
     * 
     * @param xml A XML string
     */
    Node append_xml(const std::string& xml){
        pugi::xml_document doc;
        pugi::xml_parse_result result = doc.load(xml.c_str());
        if(not result){
            STENCILA_THROW(Exception,result.description());
        }
        // To append a pugi::xml_document it is necessary to append each of
        // it children (instead of just the document root) like this...
        for(pugi::xml_node child : doc.children()) append_copy(child);
        return doc;
    }   

    /**
     * Append the children of another node
     * 
     * @param  child Child node
     */
    Node& append_children(const Node& other){
        for(pugi::xml_node child : other.children()) append_copy(child);
        return *this;
    }

    /**
     * Prepend a node
     */
    Node prepend(Node node){
        return prepend_copy(node);
    }

    /**
     * Insert a node before this one
     */
    Node before(Node node){
        return parent().insert_copy_before(node,*this);
    }

    /**
     * Insert a node after this one
     */
    Node after(Node node){
        return parent().insert_copy_after(node,*this);
    }


    /**
     * Remove a child node
     * 
     * @param  child Child node
     */
    Node& remove(const Node& child){
        remove_child(child);
        return *this;
    }
    
    /**
     * Clear all child nodes
     */
    Node& clear(void){
        while(first_child()) remove_child(first_child());
        return *this;
    }  

    /**
     * Append this node to a different parent
     * 
     * @param  to New parent of this node
     */
    Node& move(Node& to) {
        to.append_copy(*this);
        parent().remove_child(*this);
        return *this;
    }  

    /**
     * Remove this node from its parent
     */
    void destroy(void) {
        parent().remove_child(*this);
    }  

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
    Node root(void){
        return pugi::xml_node::root();
    }

    /**
     * @}
     */ 
    

    /**
     * @name Child traversal
     * @{
     */

    using pugi::xml_node::children;

    /**
     * Get the first child node of this node
     */
    Node first(void){
        return first_child();
    }

    /**
     * Get the first child  of this node that is an element
     */
    Node first_element(void){
        return find_child([](pugi::xml_node node){
            return node.type()==pugi::node_element;
        });
    }

    /**
     * Get the last child node of this node
     */
    Node last(void){
        return last_child();
    }

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
    Node next(void){
        return next_sibling();
    }

    /**
     * Get the next sibling of this node that is an element
     */
    Node next_element(void){
        pugi::xml_node sibling = next_sibling();
        while(sibling){
            if(sibling.type()==pugi::node_element){
                return sibling;
            }
            sibling = next_sibling();
        }
        return pugi::xml_node();
    }

    /**
     * Get the previous sibling of this node
     */
    Node previous(void){
        return previous_sibling();
    }

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
    Node find(const std::string& tag) const {
        return find_node([&tag](Node node){return node.name()==tag;});
    }

    /**
     * Find the first element with tag and an attribute
     * 
     * @param  tag    Tag name
     * @param  name   Name of attribute
     */
    Node find(const std::string& tag,const std::string& name) const {
        return find_node([&tag,&name](Node node){return node.name()==tag and not node.attribute(name.c_str()).empty();});
    }

    /**
     * Find the first element with tag and attribute value
     * 
     * @param  tag    Tag name
     * @param  name   Name of attribute
     * @param  value  Value of attribute
     */
    Node find(const std::string& tag,const std::string& name,const std::string& value) const {
        return find_node([&tag,&name,&value](Node node){return node.name()==tag and node.attribute(name.c_str()).value()==value;});
    }

    /**
     * Get the XPath eqivalent of a CSS selector
     * 
     * @param  selector CSS selector string
     */
    static std::string xpath(const std::string& selector);

    /**
     * Get the first element which matches the CSS selector
     * 
     * @param  selector CSS selector expression
     */
    Node one(const std::string& selector) const {
        std::string xpat = xpath(selector);
        try {
            return select_single_node(xpat.c_str()).node();
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }
    
    /**
     * Get all the element which match the CSS selector
     * 
     * @param  selector CSS selector expression
     */
    Nodes all(const std::string& selector) const {
        std::string xpat = xpath(selector);
        try {
            // Select nodes
            pugi::xpath_node_set selected = select_nodes(xpat.c_str());
            // Construct Nodes from pugi::xpath_node_set
            Nodes nodes(selected.size());
            int index = 0;
            for (pugi::xpath_node_set::const_iterator it = selected.begin(); it != selected.end(); ++it) nodes[index++] = it->node();
            return nodes;
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }

    /**
     * @}
     */ 
    

    /**
     * Sanitize the Node using a whitelist of tag names and attributes
     *
     * Only elements with names in the whitelist are allowed. Other elements are removed.
     * Those elements can only have attributes in the whitelist of the coressponding tag name. Other attributes are erased.
     */
    Node& sanitize(const Whitelist& whitelist){
        // Element nodes get checked
        if(type()==pugi::node_element){
            // Is tag name allowed?
            bool ok = false;
            std::string tag = name();
            for(auto item : whitelist){
                if(tag==item.first){
                    ok = true;
                    // Are attribute names allowed?
                    for(auto attr : attrs()){
                        bool ok = false;
                        for(auto allowed : item.second){
                            if(attr==allowed){
                                ok = true;
                                break;
                            }
                        }
                        // Attribute is not allowed...erase it
                        if(not ok) erase(attr);
                    }
                    break;
                }
            }
            if(not ok) {
                // Tag name is not allowed... remove it from parent
                parent().remove_child(*this);
            }
            else {
                // Tag name is allowed...check children
                for(Node child : children()) child.sanitize(whitelist);
            }
        }
        // Document and text nodes are not checked, only their children (if any)
        else {
            for(Node child : children()) child.sanitize(whitelist);
        }
        return *this;
    }


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
    std::string dump(bool indent=false) const {
        std::ostringstream out;
        if(!indent){
            print(out,"",pugi::format_raw);
        } else {
            print(out,"\t",pugi::format_indent);
        }
        return out.str();
    }
    
    /**
     * Dump the node's children to a string
     * 
     * @param  indent Turn on indentation?
     */
    std::string dump_children(bool indent=false) const {
        std::ostringstream out; 
        if(!indent){
            for(auto child : children()) child.print(out,"",pugi::format_raw);
        } else {
            for(auto child : children()) child.print(out,"\t",pugi::format_indent);
        }
        return out.str();
    }

    /**
     * Write the node to a file
     * 
     * @param filename Filename to write
     * @param indent   Turn on indentation?
     */
    void write(const std::string& filename,bool indent=false) const {
        std::ofstream out(filename);
        if(!indent){
            print(out,"",pugi::format_raw);
        } else {
            print(out,"\t",pugi::format_indent);
        }
    }

    /**
     * @}
     */ 
};


/**
 * XML document
 */
class Document : public Node {

protected:

    // Pugixml does not allow for copying of `xml_document`s (presumably for efficiency).
    // To have `Document` derive from `Node` (so it inherits the public interface we define above)
    // it is necessary to store a pointer to a `pugi::xml_document`, create it and the copy the `pugi::xml_node`
    // relating to that document (`pugi::xml_node`s are handles to nodes within `pugi::xml_document`s)
	pugi::xml_document* doc_;

public:

	Document(void){
		doc_ = new pugi::xml_document;
		static_cast<pugi::xml_node&>(*this) = *doc_;
	}
    
    Document(const std::string& html){
        doc_ = new pugi::xml_document;
        static_cast<pugi::xml_node&>(*this) = *doc_;
        load(html);
    }

    ~Document(void){
    	delete doc_;
    }

    /**
     * Prepend a document type declaration to the document
     * 
     * @param  type Document type
     */
    Node doctype(const std::string& type){
        pugi::xml_node doctype = prepend_child(pugi::node_doctype);
        doctype.set_value(type.c_str());
        return doctype;
    }

    /**
     * Load the document from an XML string
     * 
     * @param  xml 
     */
    Document& load(const std::string& xml){
        pugi::xml_parse_result result = doc_->load(xml.c_str());
        if(not result){
            STENCILA_THROW(Exception,result.description());
        }
        return *this;
    }

    /**
     * Read the document from a file
     * 
     * @param  filename Name of file to read from
     */
    Document& read(const std::string& filename){
        pugi::xml_parse_result result = doc_->load_file(filename.c_str());
        if(not result){
            STENCILA_THROW(Exception,result.description());
        }
        return *this;
    }
};

}
}
