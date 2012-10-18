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
 
 There is a very guide for the choice of XML libraries <a href="http://stackoverflow.com/questions/9387610/what-xml-parser-should-i-use-in-c">here</a>.
*/

typedef pugi::xml_node Node;
typedef pugi::xml_tree_walker Walker;

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
		save(out);
		return out.str();
	}
    
    Document& read(const char* filename){
        pugi::xml_parse_result result = pugi::xml_document::load_file(filename);
		if(not result){
			STENCILA_THROW(Exception,result.description());
		}
        return *this;
    }
    
    pugi::xml_node append_to(pugi::xml_node& node,const std::string& tag) const {
        pugi::xml_node child = node.append_child(tag.c_str());
        return child;
    }
    
    pugi::xml_node append_to(pugi::xml_node& node,const std::string& tag, const std::string& text) const {
        pugi::xml_node child = append_to(node,tag);
        child.append_child(pugi::node_pcdata).set_value(text.c_str());
        return child;
    }
    
    pugi::xml_node append_to(pugi::xml_node& node,const std::string& tag, const std::vector<std::pair<std::string,std::string>>& attributes, const std::string& text = "") const {
        pugi::xml_node child = append_to(node,tag);
        typedef std::pair<std::string,std::string> Attribute;
        BOOST_FOREACH(Attribute attribute,attributes){
            child.append_attribute(attribute.first.c_str()) = attribute.second.c_str();
        }
        if(text.length()>0) child.append_child(pugi::node_pcdata).set_value(text.c_str());
        return child;
    }
    
};

}
}
}
