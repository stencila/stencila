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

#pragma once

#include <string>
#include <tuple>
#include <vector>
#include <sstream>

#include <boost/foreach.hpp>
#include <boost/xpressive/xpressive_static.hpp>
#include <boost/xpressive/regex_actions.hpp>

#include <pugixml.hpp>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Formats {
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

namespace CssToXPath {
    
/*
http://www.w3.org/TR/css3-selectors

*/
    
using namespace boost::xpressive;

/*! @{

CSS selector grammar

This is a partial implementation of the grammar described in the [W3C Recommendation](http://www.w3.org/TR/css3-selectors/#w3cselgrammar)

Some of the things that are not implemented or not fully implemented
  * identifiers and strings (unicode, escape characters etc not dealt with)
  * pseudo-element ('::') 
  * pseudo-class (':')
  * negation ('not(..)')
  * namespaces
*/

//! Defines the id of each sytax component for faster matching in translate function
#define ID(name) const void* name##_ = name.regex_id();

sregex identifier      = +(_w|'-');
    ID(identifier)
sregex string          = ('\"' >> *_w >> '\"')
                       | ('\'' >> *_w >> '\'');
    ID(string)

sregex element         = identifier|'*';
    ID(element)
    
sregex attr_value      = identifier|string;
    ID(attr_value)
sregex attr_class      = '.' >> identifier;
    ID(attr_class)
sregex attr_id         = '#' >> identifier;
    ID(attr_id)
sregex attr_exists     = ('['  >> *space >> identifier >> *space >> ']' );
    ID(attr_exists)
sregex attr_comparison = as_xpr("=") | "~=" | "|=" | "^=" | "$=" | "*=";
    ID(attr_comparison)
sregex attr_compare    = ('['  >> *space >> identifier >> *space >> attr_comparison >> *space >> attr_value >> *space >> ']');
    ID(attr_compare)
sregex attr            = attr_class | attr_id | attr_exists | attr_compare;
    ID(attr)
    
sregex selector        = (element >> *attr)| (           +attr);
    ID(selector)

sregex descendant      = +space;
    ID(descendant)
sregex child           = *space >> '>' >> *space;
    ID(child)
sregex adjacent_sibling= *space >> '+' >> *space;
    ID(adjacent_sibling)
sregex general_sibling = *space >> '~' >> *space;
    ID(general_sibling)
    
sregex selectors       = selector>>!((descendant | child | adjacent_sibling | general_sibling)>>by_ref(selectors));
    ID(selectors)
    
sregex group           = selectors >> *(*space >> ',' >> *space >> selectors);
    ID(group)
    
#undef ID
//! @}

smatch parse(const std::string& css){
    smatch tree;
    bool matched = regex_search(css,tree,group);
    //If no math, or not fully matched, then report error
    if(matched){
        std::string match = tree.str(0);
        if(match.length()!=css.length()){
            std::string error = css.substr(match.length());
            STENCILA_THROW(Exception,"syntax error in: "+error);
        }
    } else {
        STENCILA_THROW(Exception,"syntax error");
    }
    return tree;
}

/*!
A map of regex addresses to their names.

Useful for printing of syntax trees
*/
std::map<const void*,std::string> rules;

/*!
Initialises the `rules` map.

Called on each invocation of print() or translate()
*/
inline
void initialise(void) {
    static bool initialised = false;
    if(initialised) return;
    
    #define MAP(name) rules[name.regex_id()] = #name;
    MAP(identifier)
    MAP(string)
    
    MAP(element)
    
    MAP(attr_value)
    MAP(attr_class)
    MAP(attr_id)
    MAP(attr_exists)
    MAP(attr_compare)
    MAP(attr)
    
    MAP(selector)
    
    MAP(descendant)
    MAP(child)
    MAP(adjacent_sibling)
    MAP(general_sibling)
    
    MAP(selectors)
    
    MAP(group)
    #undef MAP
    initialised = true;
}

/*!
Prints a node of a CSS selector syntax tree to a stream

Not usually used directly, see std::string print(const smatch& tree) for printing entire tree.
*/
void print(const smatch& node,std::ostream& stream,std::string indent=""){
    if(node.size()>0){
        auto regex_id = node.regex_id();
        std::string rule = rules[regex_id];
        stream<<indent<<rule<<" "<<node.str(0)<<" "<<node.nested_results().size()<<"\n";
    } else {
        stream<<"?\n";
    }
    for(auto i=node.nested_results().begin();i!=node.nested_results().end();i++){
        print(*i,stream,indent+"    ");
    }
}

/*!
Prints an entire CSS selector syntax tree to a string.

Useful during devlopment of grammar.
*/
std::string print(const smatch& tree){
    //Initialise the rules map
    initialise();
    //Print the tree to stream
    std::ostringstream stream;
    print(tree,stream);
    //Return the stream string
    return stream.str();
}

/*!
Translate a single node of a CSS selector syntax tree into an XPath selector

There are several resources that describe how to convert CSS selectors to XPath selectors (
 [e.g.1](http://www.a-basketful-of-papayas.net/2010/04/css-selectors-and-xpath-expressions.html)
 [e.g.2](http://hakre.wordpress.com/2012/03/18/css-selector-to-xpath-conversion/)
 [e.g.3](http://plasmasturm.org/log/444/)
). An actively developed implementation is the [`cssselect` module for Python](http://packages.python.org/cssselect)
and that has been used here as the primary source for how to do conversion. 
In particular, the [web app of cssselect)[http://css2xpath.appspot.com/] is a useful place for checking how to do translations.

@todo Performance could be improved by writing specific translate functions
when it is know what type a child node is. This would prevent having to
navigate the if else tree below for many cases.
*/
inline
std::string translate(const smatch& node,bool adjacent=false) {
    const void* id = node.regex_id();
    if(id==attr_id_){
        std::string id = node.str(0);
        //"#id" to "id"
        id.erase(0,1);
        return "@id='" + id + "'";
    }
    else if(id==attr_class_){
        std::string klass = node.str(0);
        //".class" to "class"
        klass.erase(0,1);
        return "contains(concat(' ',normalize-space(@class),' '),' " + klass + " ')";
    }
    else if(id==attr_exists_){
        std::string attr = node.nested_results().begin()->str(0);
        return "@"+attr+"";
    }
    else if(id==attr_compare_){
        auto child = node.nested_results().begin();
        std::string name = child->str(0);
        std::string op = (++child)->str(0);
        std::string value = (++child)->str(0);
        if(op=="="){
            return "@" + name + "='" + value + "'";
        }
        else if(op=="~="){
            return "contains(concat(' ',normalize-space(@" + name + ",' '),' " + value + " ')";
        }
        else if(op=="|="){
            return "(@" + name + "='" + value + "\" or starts-with(@" + name + ",'" + value + "-'))";
        }
        else if(op=="^="){
            return "starts-with(@" + name + ",'" + value + "')";
        }
        else if(op=="$="){
            return "substring(@" + name + ",string-length(@" + name + ")-" + boost::lexical_cast<std::string>(value.length()) + ")='" + value + "'";
        }
        else if(op=="*="){
            return "contains(@" + name + ",'" + value + "')";
        }
        return "error";
    }
    else if(id==selector_){
        auto children = node.nested_results();
        auto attr = children.begin();
        int attrs = children.size();
        //Determine if first child node is universal (*) or not
        //If not then attrs start at second child
        const void* id = attr->regex_id();
        std::string name;
        if(id==element_){
            name = attr->str(0);
            ++attr;
            attrs -= 1;
        } else {
            name = "*";
        }
        //Iterate through attributes
        std::string attrs_xpath;
        int index = 1;
        for(;attr!=children.end();attr++){
            attrs_xpath += translate(*attr);
            if(index<attrs) attrs_xpath += " and ";
            index++;
        }
        
        std::string xpath;
        //If this is the child of an adjacent selectors node then
        //the generated Xpath needs to be different
        if(adjacent){
            xpath = "*[name()='" + name + "' and (position()=1)";
            if(attrs>0) xpath += " and " + attrs_xpath;
            xpath += ']';
        } else {
            xpath = name;
            if(attrs>0) xpath += '[' + attrs_xpath + ']';
        }
        return xpath;
    }
    else if(id==selectors_){
        //Determine if a relation is involved
        auto children = node.nested_results();
        if(children.size()==1){
            //No relation, just a simple selector
            return translate(*children.begin(),adjacent);
        } else {
            //Determine the type of the relation (the second child of the node)
            auto child = children.begin();
            auto left = child;
            auto relation = ++child;
            auto right = ++child;
            const void* id = relation->regex_id();
            if(id==descendant_){
                return translate(*left,adjacent)+"/descendant::"+translate(*right);
            }
            else if(id==child_){
                return translate(*left,adjacent)+"/"+translate(*right);
            }
            else if(id==adjacent_sibling_){
                return translate(*left,adjacent)+"/following-sibling::"+ translate(*right,true);
            }
            else if(id==general_sibling_){
                return translate(*left,adjacent)+"/following-sibling::"+ translate(*right);
            }
            return "error";
        }
    }
    else if(id==group_){
        //Root of sytax tree.
        std::string xpath = "descendant-or-self::";
        //Separate selectors using |
        auto children = node.nested_results();
        int n = children.size();
        int index = 1;
        for(auto i=children.begin();i!=children.end();i++){
            xpath += translate(*i);
            if(index<n) xpath += " | ";
            index++;
        }
        return xpath;
    }
    else {
        //Default is to translate each child node
        std::string xpath;
        for(auto i=node.nested_results().begin();i!=node.nested_results().end();i++){
            xpath += translate(*i);
        }
        return xpath;
    }
}

/*!
Translate a CSS selector string into an XPath selector string
*/
std::string translate(const std::string& css) {
    //Generate a CSS selector syntax tree
    smatch tree = parse(css);
    //Translate tree into an XPath selector
    std::string xpath = translate(tree);
    return xpath;
}

}

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
void NodeSetAttribute(Node node,const std::string& name){
    //Check whether attribute already exists and add it if it does not
    Attribute attr = node.find_attribute(AttributeHasName{name});
    if(not attr) node.append_attribute(name.c_str());
}

Node NodeAppend(Node node,const std::string& tag) {
    Node child = node.append_child(tag.c_str());
    return child;
}

Node NodeAppend(Node node,const std::string& tag, const std::string& text) {
    Node child = NodeAppend(node,tag);
    child.append_child(pugi::node_pcdata).set_value(text.c_str());
    return child;
}

Node NodeAppend(Node node,const std::string& tag, const std::vector<std::pair<std::string,std::string>>& attributes, const std::string& text = "") {
    Node child = NodeAppend(node,tag);
    typedef std::pair<std::string,std::string> Attribute;
    BOOST_FOREACH(Attribute attribute,attributes){
        child.append_attribute(attribute.first.c_str()) = attribute.second.c_str();
    }
    if(text.length()>0) child.append_child(pugi::node_pcdata).set_value(text.c_str());
    return child;
}

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
    
    Document& read(const char* filename){
        pugi::xml_parse_result result = pugi::xml_document::load_file(filename);
		if(not result){
			STENCILA_THROW(Exception,result.description());
		}
        return *this;
    }
    
    Node one(const std::string& css_selector){
        std::string xpath = CssToXPath::translate(css_selector);
        try {
            return select_single_node(xpath.c_str()).node();
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }
    
    Nodes all(const std::string& css_selector){
        std::string xpath = CssToXPath::translate(css_selector);
        try {
            return select_nodes(xpath.c_str());
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }
    
    Nodes operator[](const std::string& css_selector){
        return all(css_selector);
    }
    
};

}
}
}
