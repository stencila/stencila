#pragma once

#include <fstream>

#include <pugixml.hpp>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Utilities {
namespace Xml {

/**
 * @namespace Xml
 *
 * Stencila's XML interface currently based on [pugixml](http://pugixml.org)
 * This namespace contains utility classes for handling <a href="http://en.wikipedia.org/wiki/XML/">Extensible Markup Language (XML)</a>.

 * The Stencila library currently uses <a href="http://pugixml.org/">pugixml</a> as a backend for XML parsing and generation.
 * We chose pugixml because it is fast and has XPath support.
 * Documentation for pugixml is available <a href="http://pugixml.org/documentation/">here</a>.

 * There is a brief, but very good, guide on how to choose a XML library <a href="http://stackoverflow.com/questions/9387610/what-xml-parser-should-i-use-in-c">here</a>.
 * It summarises many of the conclusions that we came to in choosing pugixml for Stencila.
**/

class Node;

class Attribute : private pugi::xml_attribute {
private:

	friend class Node;

public:

	Attribute(const pugi::xml_attribute& attr):
		pugi::xml_attribute(attr){
	}

	operator bool(void) const {
		return not empty();
	}
	bool operator!(void) const {
		return empty();
	}

};

typedef pugi::xpath_node_set Nodes;

class Node : protected pugi::xml_node {

public:

	Node(void){

	}

	Node(const pugi::xml_node& node):
		pugi::xml_node(node){
	}

	operator bool(void) const {
		return not empty();
	}
	bool operator!(void) const {
		return empty();
	}


    std::string text(void) const {
    	return pugi::xml_node::text().get();
    }

    Node& text(const std::string& text) {
    	pugi::xml_node::text().set(text.c_str());
    	return *this;
    }


	/**
	 * @name Attributes
	 * @{
	 */

private:

	Attribute attr_get_(const std::string& name) const {
		return find_attribute([&name](Attribute attr){
        	return attr.name()==name;
    	});
	}

public:

	std::string attr(const std::string& name) const {
		Attribute attr = attr_get_(name);
		if(attr) return attr.value();
		else return "";
	}

	Node& attr(const std::string& name,const std::string& value){
        Attribute attr = attr_get_(name);
        if(attr) attr.set_value(value.c_str());
        else append_attribute(name.c_str()) = value.c_str();
		return *this;
	}

	Node& add(const std::string& name, const std::string& value){
        Attribute attr = attr_get_(name);
        if(attr){
            std::string current = attr.as_string();
            std::string future;
            if(current.length()>0) future = current + " " + value;
            else future = value;
            attr.set_value(future.c_str());
        }else {
            append_attribute(name.c_str()) = value.c_str();
        }
        return *this;
    }

	Node& erase(const std::string& name){
		Attribute attr = attr_get_(name);
        if(attr) remove_attribute(attr);
		return *this;
	}

	/**
	 * @}
	 */
	
	Node append(const Node& node) {
        return append_copy(node);
    }

	Node append(const std::string& tag) {
        return append_child(tag.c_str());
    }

    Node append(const std::string& tag, const std::string& text) {
        Node child = append(tag);
        child.append_child(pugi::node_pcdata).set_value(text.c_str());
        return child;
    }

	Node append(const std::string& tag, const std::vector<std::pair<std::string,std::string>>& attributes, const std::string& text = "") {
        Node child = append(tag);
        typedef std::pair<std::string,std::string> Attribute;
        for(Attribute attribute : attributes){
            child.append_attribute(attribute.first.c_str()) = attribute.second.c_str();
        }
        if(text.length()>0) child.append_child(pugi::node_pcdata).set_value(text.c_str());
        return child;
    }

    Node append_text(const std::string& text){
        Node child = append_child(pugi::node_pcdata);
        child.text(text);
        return child;
    }

    Node append_cdata(const std::string& text){
        Node child = append_child(pugi::node_cdata);
        child.text(text);
        return child;
    }

    Node append_comment(const std::string& text){
        Node child = append_child(pugi::node_comment);
        child.set_value(text.c_str());
        return child;
    }
    
    Node append_xml(const std::string& xml){
        pugi::xml_document doc;
        pugi::xml_parse_result result = doc.load(xml.c_str());
        if(not result){
            STENCILA_THROW(Exception,result.description());
        }
        return append(doc);
    }    

    Node& remove(const Node& child){
        remove_child(child);
        return *this;
    }
    
    Node& clear(void){
        while(first_child()) remove_child(first_child());
        return *this;
    }    


    Node find(const std::string& tag) const {
        return find_node([&tag](Node node){return node.name()==tag;});
    }

    Node find(const std::string& tag,const std::string& name,const std::string& value) const {
        return find_node([&tag,&name,&value](Node node){return node.name()==tag and node.attribute(name.c_str()).value()==value;});
    }

    /**
     * Get the XPath eqivalent of a CSS selector
     * 
     * @param  selector CSS selector string
     */
    static std::string xpath(const std::string& selector);

    Node one(const std::string& selector) const {
        std::string xpat = xpath(selector);
        try {
            return select_single_node(xpat.c_str()).node();
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }
    
    Nodes all(const std::string& selector) const {
        std::string xpat = xpath(selector);
        try {
            return select_nodes(xpat.c_str());
        } catch (const pugi::xpath_exception& e){
            STENCILA_THROW(Exception,e.what());
        }
    }


    std::string dump(bool indent=false) const {
        std::ostringstream out;
        if(!indent){
            print(out,"",pugi::format_raw);
        } else {
            print(out,"\t",pugi::format_indent);
        }
        return out.str();
    }

    void write(const std::string& filename,bool indent=false) const {
        std::ofstream out(filename);
        if(!indent){
            print(out,"",pugi::format_raw);
        } else {
            print(out,"\t",pugi::format_indent);
        }
    }

};

class Document : public Node {

protected:

	pugi::xml_document* doc_;

public:

	Document(void){
		doc_ = new pugi::xml_document;
		static_cast<pugi::xml_node&>(*this) = *doc_;
	}

    ~Document(void){
    	delete doc_;
    }

    Document& load(const std::string& xml){
        pugi::xml_parse_result result = doc_->load(xml.c_str());
        if(not result){
            STENCILA_THROW(Exception,result.description());
        }
        return *this;
    }

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
}
