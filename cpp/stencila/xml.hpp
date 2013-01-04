/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file xml.hpp
//! @brief Classes and functions for working with XML
//! @author Nokome Bentley

#pragma once

#include <string>
#include <tuple>
#include <vector>
#include <sstream>

#include <boost/foreach.hpp>

#include <pugixml.hpp>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Xml {

/*! 
@namespace  Xml

This namespace contains utility classes for handling <a href="http://en.wikipedia.org/wiki/XML/">Extensible Markup Language (XML)</a>.

The Stencila library currently uses <a href="http://pugixml.org/">pugixml</a> as a backend for XML parsing and generation.
We chose pugixml because it is fast and has XPath support.
Documentation for pugixml is available <a href="http://pugixml.org/documentation/">here</a>.

There is a brief, but very good, guide on how to choose a XML library <a href="http://stackoverflow.com/questions/9387610/what-xml-parser-should-i-use-in-c">here</a>.
It summarises many of the conclusions that we came to in choosing pugixml for Stencila.
*/

////////////////////////////////////////////

typedef pugi::xml_attribute Attribute;

struct AttributeHasName{
    std::string name;
    bool operator()(Attribute attr) const {
        return attr.name()==name;
    }
};

////////////////////////////////////////////

typedef pugi::xml_node Node;
typedef pugi::xpath_node_set Nodes;

/*!
Returns true is the node is an element.

Useful for find_child() and find_node() methods of Node which take a boolean predicate function
*/
inline
bool NodeIsElement(Node node){
    return node.type()==pugi::node_element;
}

inline
bool NodeHasAttribute(Node node,const std::string& name){
    return node.find_attribute(AttributeHasName{name});
}

inline
Attribute NodeGetAttribute(Node node,const std::string& name){
    return node.find_attribute(AttributeHasName{name});
}

inline
void NodeSetAttribute(Node node,const std::string& name, const std::string& value){
    // Check whether attribute already exists 
    Attribute attr = node.find_attribute(AttributeHasName{name});
    // and set its value if it does,
    if(attr){
        attr.set_value(value.c_str());
    }
    // or add attribute and set its value if it does not.
    else {
        node.append_attribute(name.c_str()) = value.c_str();
    }
}

inline
void NodeAppendAttribute(Node node,const std::string& name, const std::string& value){
    // Check whether attribute already exists 
    Attribute attr = node.find_attribute(AttributeHasName{name});
    // and append its value if it does,
    if(attr){
        std::string current = attr.as_string();
        std::string future;
        if(current.length()>0) future = current + " " + value;
        else future = value;
        attr.set_value(future.c_str());
    }
    // or add attribute and set its value if it does not.
    else {
        node.append_attribute(name.c_str()) = value.c_str();
    }
}

inline
void NodeSetAttribute(Node node,const std::string& name){
    //Check whether attribute already exists and add it if it does not
    Attribute attr = node.find_attribute(AttributeHasName{name});
    if(not attr) node.append_attribute(name.c_str());
}

inline
Node NodeAppend(Node node,const std::string& tag) {
    Node child = node.append_child(tag.c_str());
    return child;
}

inline
Node NodeAppend(Node node,const std::string& tag, const std::string& text) {
    Node child = NodeAppend(node,tag);
    child.append_child(pugi::node_pcdata).set_value(text.c_str());
    return child;
}

inline
Node NodeAppend(Node node,const std::string& tag, const std::vector<std::pair<std::string,std::string>>& attributes, const std::string& text = "") {
    Node child = NodeAppend(node,tag);
    typedef std::pair<std::string,std::string> Attribute;
    BOOST_FOREACH(Attribute attribute,attributes){
        child.append_attribute(attribute.first.c_str()) = attribute.second.c_str();
    }
    if(text.length()>0) child.append_child(pugi::node_pcdata).set_value(text.c_str());
    return child;
}

inline
Node NodeAppendText(Node node,const std::string& text){
    Node child = node.append_child(pugi::node_pcdata);
    child.text().set(text.c_str());
    return child;
}

inline
Node NodeAppendCData(Node node,const std::string& text){
    Node child = node.append_child(pugi::node_cdata);
    child.text().set(text.c_str());
    return child;
}

inline
Node NodeAppendComment(Node node,const std::string& text){
    Node child = node.append_child(pugi::node_comment);
    child.set_value(text.c_str());
    return child;
}

inline
void NodeAppendXml(Node node,const std::string& xml){
    pugi::xml_document doc;
    pugi::xml_parse_result result = doc.load(xml.c_str());
    if(not result){
        STENCILA_THROW(Exception,result.description());
    }
    //It is necessary to copy each child of the document to the node.
    //The document itself can not be copied over
    for(Node child : doc.children()){
        node.append_copy(child);
    }
}

////////////////////////////////////////////

typedef pugi::xml_tree_walker Walker;

////////////////////////////////////////////

std::string CssToXpath(const std::string& css);

class Document : public pugi::xml_document {
public:
Document(void){}

Document(const std::string& xml){
load(xml);
}

Document& load(const std::string& xml){
pugi::xml_parse_result result = pugi::xml_document::load(xml.c_str());
if(not result){
STENCILA_THROW(Exception,result.description());
}
return *this;
}

std::string dump(void) const {
std::ostringstream out;
save(out,"\t",pugi::format_raw | pugi::format_no_declaration);
return out.str();
}

std::string print(void) const {
std::ostringstream out;
save(out,"\t",pugi::format_indent | pugi::format_no_declaration);
return out.str();
}

    //! @brief 
    //! @param filename
    //! @return 
    Document& read(const std::string& filename){
pugi::xml_parse_result result = pugi::xml_document::load_file(filename.c_str());
if(not result){
STENCILA_THROW(Exception,result.description());
}
return *this;
    }
    
    //! @brief 
    //! @param selector
    //! @return 
    Node one(const std::string& css_selector){
        std::string xpath = CssToXpath(css_selector);
        try {
            return select_single_node(xpath.c_str()).node();
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }
    
    //! @brief 
    //! @param selector
    //! @return 
    Nodes all(const std::string& css_selector){
        std::string xpath = CssToXpath(css_selector);
        try {
            return select_nodes(xpath.c_str());
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }
    
    //! @brief 
    //! @param selector
    //! @return 
    Nodes operator[](const std::string& css_selector){
        return all(css_selector);
    }
    
};

}
}
